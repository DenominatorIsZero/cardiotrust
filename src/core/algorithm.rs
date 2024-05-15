pub mod estimation;
pub mod metrics;
pub mod refinement;

use nalgebra::{DMatrix, SVD};
use ndarray::{s, Array1, ArrayBase, Dim, ViewRepr};
use rand::seq::SliceRandom;
use rand::thread_rng;
use tracing::{debug, trace};

use crate::core::algorithm::estimation::update_kalman_gain_and_check_convergence;

use self::estimation::{
    calculate_delays_delta, calculate_gains_delta, calculate_post_update_residuals,
    calculate_residuals, calculate_system_states_delta, calculate_system_update,
    prediction::calculate_system_prediction, Estimations,
};
use super::{
    config::algorithm::Algorithm,
    data::{shapes::ArraySystemStates, Data},
    model::functional::FunctionalDescription,
    scenario::results::Results,
};

/// Calculates a pseudo inverse of the measurement matrix and estimates the system states, residuals, derivatives, and metrics.
///
/// This iterates through each time step, calculating the system state estimate, residuals, derivatives, and metrics at each step.
/// It uses SVD to calculate the pseudo inverse of the measurement matrix.
///
/// # Panics
///
/// - svd calculation fails
///
#[tracing::instrument(level = "debug", skip_all)]
pub fn calculate_pseudo_inverse(
    functional_description: &FunctionalDescription,
    results: &mut Results,
    data: &Data,
    config: &Algorithm,
) {
    debug!("Calculating pseudo inverse");
    let rows = functional_description.measurement_matrix.values.shape()[1];
    let columns = functional_description.measurement_matrix.values.shape()[2];
    let measurement_matrix = functional_description
        .measurement_matrix
        .values
        .slice(s![0, .., ..]);
    let measurement_matrix = DMatrix::from_row_slice(
        rows,
        columns,
        measurement_matrix.as_slice().expect("Slice to be some."),
    );

    let decomposition = SVD::new_unordered(measurement_matrix, true, true);

    let estimations = &mut results.estimations;
    let derivatives = &mut results.derivatives;

    for time_index in 0..estimations.system_states.num_steps() {
        let rows = data.get_measurements().num_sensors();
        let measurements = DMatrix::from_row_slice(
            rows,
            1,
            data.get_measurements()
                .slice(s![0, time_index, ..])
                .as_slice()
                .expect("Slice to be some."),
        );

        let system_states = decomposition
            .solve(&measurements, 1e-5)
            .expect("SVD to be computed.");

        let system_states = Array1::from_iter(system_states.as_slice().iter().copied());

        estimations
            .system_states
            .slice_mut(s![time_index, ..])
            .assign(&system_states);

        let measurement_matrix: ArrayBase<ViewRepr<&f32>, Dim<[usize; 2]>> = functional_description
            .measurement_matrix
            .values
            .slice(s![0, .., ..]);

        estimations
            .measurements
            .slice_mut(s![0, time_index, ..])
            .assign(&measurement_matrix.dot(&estimations.system_states.slice(s![time_index, ..])));

        calculate_residuals(
            &mut estimations.residuals,
            &estimations.measurements,
            data.get_measurements(),
            time_index,
            0,
        );

        derivatives.calculate(functional_description, estimations, config, time_index, 0);

        calculate_post_update_residuals(
            &mut estimations.post_update_residuals,
            &functional_description.measurement_matrix,
            &estimations.system_states,
            data.get_measurements(),
            time_index,
            0,
        );
        calculate_system_states_delta(
            &mut estimations.system_states_delta,
            &estimations.system_states,
            data.get_system_states(),
            time_index,
        );
        results.metrics.calculate_step(
            estimations,
            derivatives,
            config.regularization_strength,
            time_index,
        );
    }
    results.metrics.calculate_epoch(0);
}

/// Runs the algorithm for one epoch.
///
/// This includes calculating the system estimates
/// and performing one gradient descent step.
#[tracing::instrument(skip_all, level = "debug")]
pub fn run_epoch(
    functional_description: &mut FunctionalDescription,
    results: &mut Results,
    data: &Data,
    config: &Algorithm,
    epoch_index: usize,
) {
    debug!("Running epoch {}", epoch_index);
    results.derivatives.reset();
    let num_steps = results.estimations.system_states.num_steps();
    let num_beats = data.get_measurements().num_beats();

    let mut batch = match config.batch_size {
        0 => None,
        _ => Some((epoch_index * num_beats) % config.batch_size),
    };

    let mut beat_indices: Vec<usize> = (0..num_beats).collect();
    let mut rng = thread_rng();
    beat_indices.shuffle(&mut rng);

    for beat_index in beat_indices {
        results.estimations.reset();
        results.estimations.kalman_gain_converged = false;
        let estimations = &mut results.estimations;
        let derivatives = &mut results.derivatives;
        for time_index in 0..num_steps {
            calculate_system_prediction(
                &mut estimations.ap_outputs,
                &mut estimations.system_states,
                &mut estimations.measurements,
                functional_description,
                time_index,
                beat_index,
            );
            calculate_residuals(
                &mut estimations.residuals,
                &estimations.measurements,
                data.get_measurements(),
                time_index,
                beat_index,
            );

            derivatives.calculate(
                functional_description,
                estimations,
                config,
                time_index,
                beat_index,
            );

            if config.model.common.apply_system_update {
                if config.update_kalman_gain {
                    update_kalman_gain_and_check_convergence(
                        estimations,
                        functional_description,
                        beat_index,
                    );
                }
                calculate_system_update(estimations, time_index, functional_description, config);
            }

            calculate_deltas(
                estimations,
                functional_description,
                data,
                time_index,
                beat_index,
            );

            results.metrics.calculate_step(
                estimations,
                derivatives,
                config.regularization_strength,
                time_index,
            );
        }
        if let Some(n) = batch.as_mut() {
            *n += 1;
            if *n == config.batch_size {
                functional_description
                    .ap_params
                    .update(derivatives, config, num_steps, num_beats);
                derivatives.reset();
                estimations.kalman_gain_converged = false;
                *n = 0;
            }
        }
    }
    if batch.is_none() {
        functional_description.ap_params.update(
            &mut results.derivatives,
            config,
            num_steps,
            num_beats,
        );
        results.derivatives.reset();
        results.estimations.kalman_gain_converged = false;
    }
    results.metrics.calculate_epoch(epoch_index);
}

#[tracing::instrument(level = "trace")]
pub fn calculate_deltas(
    estimations: &mut Estimations,
    functional_description: &FunctionalDescription,
    data: &Data,
    time_index: usize,
    beat_index: usize,
) {
    trace!("Calculating deltas");
    calculate_post_update_residuals(
        &mut estimations.post_update_residuals,
        &functional_description.measurement_matrix,
        &estimations.system_states,
        data.get_measurements(),
        time_index,
        beat_index,
    );
    calculate_system_states_delta(
        &mut estimations.system_states_delta,
        &estimations.system_states,
        data.get_system_states(),
        time_index,
    );
    calculate_gains_delta(
        &mut estimations.gains_delta,
        &functional_description.ap_params.gains,
        data.get_gains(),
    );
    calculate_delays_delta(
        &mut estimations.delays_delta,
        &functional_description.ap_params.delays,
        data.get_delays(),
        &functional_description.ap_params.coefs,
        data.get_coefs(),
    );
}

#[tracing::instrument(level = "trace")]
pub fn constrain_system_states(
    system_states: &mut ArraySystemStates,
    time_index: usize,
    clamping_threshold: f32,
) {
    trace!("Constraining system states");
    for state_index in (0..system_states.num_states()).step_by(3) {
        let sum = system_states[[time_index, state_index]].abs()
            + system_states[[time_index, state_index + 1]].abs()
            + system_states[[time_index, state_index + 2]].abs();
        if sum > clamping_threshold {
            let factor = clamping_threshold / sum;
            system_states[[time_index, state_index]] *= factor;
            system_states[[time_index, state_index + 1]] *= factor;
            system_states[[time_index, state_index + 2]] *= factor;
        }
    }
}

#[cfg(test)]
mod test {

    use std::path::Path;

    use bevy::utils::petgraph::algo;
    use ndarray::Dim;
    use tracing::info;

    use crate::core::config::algorithm::Algorithm as AlgorithmConfig;
    use crate::core::config::model::{SensorArrayGeometry, SensorArrayMotion};
    use crate::core::config::simulation::Simulation as SimulationConfig;
    use crate::core::model::Model;

    use crate::vis::plotting::gif::states::states_spherical_plot_over_time;
    use crate::vis::plotting::png::line::standard_y_plot;
    use crate::vis::plotting::png::states::states_spherical_plot;
    use crate::vis::plotting::{PlotSlice, StateSphericalPlotMode};

    use super::*;

    const COMMON_PATH: &str = "tests/core/algorithm";

    #[tracing::instrument(level = "trace")]
    fn setup(folder: Option<&str>) {
        let path = folder.map_or_else(
            || Path::new(COMMON_PATH).to_path_buf(),
            |folder| Path::new(COMMON_PATH).join(folder),
        );

        if !path.exists() {
            std::fs::create_dir_all(path).unwrap();
        }
    }

    #[tracing::instrument(level = "info", skip_all)]
    fn run(
        functional_description: &mut FunctionalDescription,
        results: &mut Results,
        data: &Data,
        algorithm_config: &Algorithm,
    ) {
        info!("Running optimization.");
        for epoch_index in 0..algorithm_config.epochs {
            run_epoch(
                functional_description,
                results,
                data,
                algorithm_config,
                epoch_index,
            );
        }
        results
            .estimations
            .system_states_spherical
            .calculate(&results.estimations.system_states);
        results
            .estimations
            .system_states_spherical_max
            .calculate(&results.estimations.system_states_spherical);
    }

    #[test]
    fn run_epoch_no_crash() {
        let number_of_states = 3000;
        let number_of_sensors = 300;
        let number_of_steps = 3;
        let number_of_epochs = 10;
        let config = AlgorithmConfig::default();
        let epoch_index = 3;
        let voxels_in_dims = Dim([1000, 1, 1]);
        let number_of_beats = 10;

        let mut functional_description = FunctionalDescription::empty(
            number_of_states,
            number_of_sensors,
            number_of_steps,
            number_of_beats,
            voxels_in_dims,
        );
        let mut results = Results::new(
            number_of_epochs,
            number_of_steps,
            number_of_sensors,
            number_of_states,
            number_of_beats,
            config.optimizer,
        );
        let data = Data::empty(
            number_of_sensors,
            number_of_states,
            number_of_steps,
            voxels_in_dims,
            number_of_beats,
        );

        run_epoch(
            &mut functional_description,
            &mut results,
            &data,
            &config,
            epoch_index,
        );
    }

    #[test]
    fn run_no_crash() {
        let number_of_states = 3000;
        let number_of_sensors = 300;
        let number_of_steps = 3;
        let number_of_beats = 7;
        let voxels_in_dims = Dim([1000, 1, 1]);

        let algorithm_config = AlgorithmConfig {
            epochs: 3,
            ..Default::default()
        };
        let mut functional_description = FunctionalDescription::empty(
            number_of_states,
            number_of_sensors,
            number_of_steps,
            number_of_beats,
            voxels_in_dims,
        );
        let mut results = Results::new(
            algorithm_config.epochs,
            number_of_steps,
            number_of_sensors,
            number_of_states,
            number_of_beats,
            algorithm_config.optimizer,
        );
        let data = Data::empty(
            number_of_sensors,
            number_of_states,
            number_of_steps,
            voxels_in_dims,
            number_of_beats,
        );

        run(
            &mut functional_description,
            &mut results,
            &data,
            &algorithm_config,
        );
    }

    #[test]
    fn loss_decreases() {
        let mut simulation_config = SimulationConfig::default();
        simulation_config.model.common.pathological = true;
        simulation_config.model.common.sensor_array_geometry = SensorArrayGeometry::Cube;
        simulation_config.model.common.sensor_array_motion = SensorArrayMotion::Static;
        let data = Data::from_simulation_config(&simulation_config)
            .expect("Model parameters to be valid.");

        let mut algorithm_config = Algorithm {
            learning_rate: 1.0,
            epochs: 3,
            ..Default::default()
        };
        algorithm_config.model.common.apply_system_update = true;
        algorithm_config.model.common.sensor_array_geometry = SensorArrayGeometry::Cube;
        algorithm_config.model.common.sensor_array_motion = SensorArrayMotion::Static;

        let mut model = Model::from_model_config(
            &algorithm_config.model,
            simulation_config.sample_rate_hz,
            simulation_config.duration_s,
        )
        .expect("Model parameters to be valid.");

        let mut results = Results::new(
            algorithm_config.epochs,
            model
                .functional_description
                .control_function_values
                .values
                .shape()[0],
            model.spatial_description.sensors.count(),
            model.spatial_description.voxels.count_states(),
            simulation_config
                .model
                .common
                .sensor_array_motion_steps
                .iter()
                .product(),
            algorithm_config.optimizer,
        );

        run(
            &mut model.functional_description,
            &mut results,
            &data,
            &algorithm_config,
        );

        (0..algorithm_config.epochs - 1).for_each(|i| {
            assert!(
                results.metrics.loss_epoch.values[i] > results.metrics.loss_epoch.values[i + 1]
            );
        });
    }

    #[test]
    #[ignore]
    fn loss_decreases_and_plot() {
        setup(Some("default"));
        let mut simulation_config = SimulationConfig::default();
        simulation_config.model.common.pathological = true;
        simulation_config.model.common.sensor_array_geometry = SensorArrayGeometry::Cube;
        simulation_config.model.common.sensor_array_motion = SensorArrayMotion::Static;
        let data = Data::from_simulation_config(&simulation_config)
            .expect("Model parameters to be valid.");

        let mut algorithm_config = Algorithm::default();
        algorithm_config.model.common.sensor_array_geometry = SensorArrayGeometry::Cube;
        algorithm_config.model.common.sensor_array_motion = SensorArrayMotion::Static;

        let mut model = Model::from_model_config(
            &algorithm_config.model,
            simulation_config.sample_rate_hz,
            simulation_config.duration_s,
        )
        .expect("Model paramters to be valid");
        algorithm_config.epochs = 10;
        algorithm_config.model.common.apply_system_update = true;

        let mut results = Results::new(
            algorithm_config.epochs,
            model
                .functional_description
                .control_function_values
                .values
                .shape()[0],
            model.spatial_description.sensors.count(),
            model.spatial_description.voxels.count_states(),
            simulation_config
                .model
                .common
                .sensor_array_motion_steps
                .iter()
                .product(),
            algorithm_config.optimizer,
        );

        run(
            &mut model.functional_description,
            &mut results,
            &data,
            &algorithm_config,
        );

        let path = Path::new(COMMON_PATH).join("default").join("loss.png");
        standard_y_plot(
            &results.metrics.loss.values,
            Path::new(path.as_path()),
            "Loss",
            "Loss",
            "Step",
        )
        .unwrap();

        let path = Path::new(COMMON_PATH)
            .join("default")
            .join("loss_epoch.png");
        standard_y_plot(
            &results.metrics.loss_epoch.values,
            Path::new(path.as_path()),
            "Sum Loss Per Epoch",
            "Loss",
            "Epoch",
        )
        .unwrap();

        let path = Path::new(COMMON_PATH)
            .join("default")
            .join("states_max.png");
        states_spherical_plot(
            &results.estimations.system_states_spherical,
            &results.estimations.system_states_spherical_max,
            &model.spatial_description.voxels.positions_mm,
            model.spatial_description.voxels.size_mm,
            &model.spatial_description.voxels.numbers,
            Some(path.as_path()),
            None,
            Some(StateSphericalPlotMode::ABS),
            None,
            None,
        )
        .unwrap();

        let fps = 20;
        let playback_speed = 0.1;

        let path = Path::new(COMMON_PATH).join("default").join("states.gif");
        states_spherical_plot_over_time(
            &results.estimations.system_states_spherical,
            &results.estimations.system_states_spherical_max,
            &model.spatial_description.voxels.positions_mm,
            model.spatial_description.voxels.size_mm,
            simulation_config.sample_rate_hz,
            &model.spatial_description.voxels.numbers,
            Some(path.as_path()),
            Some(PlotSlice::Z(0)),
            Some(StateSphericalPlotMode::ABS),
            Some(playback_speed),
            Some(fps),
        )
        .unwrap();
    }

    #[test]
    fn loss_decreases_kalman() {
        let mut simulation_config = SimulationConfig::default();
        simulation_config.model.common.pathological = true;
        simulation_config.model.common.sensor_array_geometry = SensorArrayGeometry::Cube;
        simulation_config.model.common.sensor_array_motion = SensorArrayMotion::Static;
        let data = Data::from_simulation_config(&simulation_config)
            .expect("Model parameters to be valid.");

        let mut algorithm_config = Algorithm {
            update_kalman_gain: true,
            learning_rate: 1.0,
            epochs: 3,
            ..Default::default()
        };
        algorithm_config.model.common.apply_system_update = true;
        algorithm_config.model.common.sensor_array_geometry = SensorArrayGeometry::Cube;
        algorithm_config.model.common.sensor_array_motion = SensorArrayMotion::Static;

        let mut model = Model::from_model_config(
            &algorithm_config.model,
            simulation_config.sample_rate_hz,
            simulation_config.duration_s,
        )
        .expect("Model parameters to be valid.");

        let mut results = Results::new(
            algorithm_config.epochs,
            model
                .functional_description
                .control_function_values
                .values
                .shape()[0],
            model.spatial_description.sensors.count(),
            model.spatial_description.voxels.count_states(),
            simulation_config
                .model
                .common
                .sensor_array_motion_steps
                .iter()
                .product(),
            algorithm_config.optimizer,
        );

        run(
            &mut model.functional_description,
            &mut results,
            &data,
            &algorithm_config,
        );

        println!("{:?}", results.metrics.loss_mse_epoch.values);

        (0..algorithm_config.epochs - 1).for_each(|i| {
            assert!(
                results.metrics.loss_mse_epoch.values[i]
                    > results.metrics.loss_mse_epoch.values[i + 1]
            );
        });
    }

    #[test]
    fn loss_decreases_no_kalman() {
        let mut simulation_config = SimulationConfig::default();
        simulation_config.model.common.pathological = true;
        simulation_config.model.common.sensor_array_geometry = SensorArrayGeometry::Cube;
        simulation_config.model.common.sensor_array_motion = SensorArrayMotion::Static;
        let data = Data::from_simulation_config(&simulation_config)
            .expect("Model parameters to be valid.");

        let mut algorithm_config = Algorithm::default();
        algorithm_config.model.common.sensor_array_geometry = SensorArrayGeometry::Cube;
        algorithm_config.model.common.sensor_array_motion = SensorArrayMotion::Static;

        let mut model = Model::from_model_config(
            &algorithm_config.model,
            simulation_config.sample_rate_hz,
            simulation_config.duration_s,
        )
        .expect("Model parameters to be valid.");
        algorithm_config.epochs = 5;
        algorithm_config.learning_rate = 1.0;
        algorithm_config.model.common.apply_system_update = false;

        let mut results = Results::new(
            algorithm_config.epochs,
            model
                .functional_description
                .control_function_values
                .values
                .shape()[0],
            model.spatial_description.sensors.count(),
            model.spatial_description.voxels.count_states(),
            simulation_config
                .model
                .common
                .sensor_array_motion_steps
                .iter()
                .product(),
            algorithm_config.optimizer,
        );

        run(
            &mut model.functional_description,
            &mut results,
            &data,
            &algorithm_config,
        );

        println!("{:?}", results.metrics.loss_epoch.values);
        (0..algorithm_config.epochs - 1).for_each(|i| {
            assert!(
                results.metrics.loss_mse_epoch.values[i]
                    > results.metrics.loss_mse_epoch.values[i + 1]
            );
        });
    }

    #[test]
    #[ignore]
    fn loss_decreases_no_kalman_and_plot() {
        setup(Some("no_kalman"));
        let mut simulation_config = SimulationConfig::default();
        simulation_config.model.common.pathological = true;
        simulation_config.model.common.sensor_array_geometry = SensorArrayGeometry::Cube;
        simulation_config.model.common.sensor_array_motion = SensorArrayMotion::Static;
        let data = Data::from_simulation_config(&simulation_config)
            .expect("Model parameters to be valid.");

        let mut algorithm_config = Algorithm::default();
        algorithm_config.model.common.sensor_array_geometry = SensorArrayGeometry::Cube;
        algorithm_config.model.common.sensor_array_motion = SensorArrayMotion::Static;

        let mut model = Model::from_model_config(
            &algorithm_config.model,
            simulation_config.sample_rate_hz,
            simulation_config.duration_s,
        )
        .expect("Model parameters to be valid.");
        algorithm_config.epochs = 10;
        algorithm_config.model.common.apply_system_update = false;

        let mut results = Results::new(
            algorithm_config.epochs,
            model
                .functional_description
                .control_function_values
                .values
                .shape()[0],
            model.spatial_description.sensors.count(),
            model.spatial_description.voxels.count_states(),
            simulation_config
                .model
                .common
                .sensor_array_motion_steps
                .iter()
                .product(),
            algorithm_config.optimizer,
        );

        run(
            &mut model.functional_description,
            &mut results,
            &data,
            &algorithm_config,
        );

        let path = Path::new(COMMON_PATH).join("no_kalman").join("loss.png");
        standard_y_plot(
            &results.metrics.loss.values,
            Path::new(path.as_path()),
            "Loss",
            "Loss",
            "Step",
        )
        .unwrap();
        let path = Path::new(COMMON_PATH)
            .join("no_kalman")
            .join("loss_epoch.png");
        standard_y_plot(
            &results.metrics.loss_epoch.values,
            Path::new(path.as_path()),
            "Sum Loss Per Epoch",
            "Loss",
            "Epoch",
        )
        .unwrap();

        let path = Path::new(COMMON_PATH)
            .join("no_kalman")
            .join("states_max.png");
        states_spherical_plot(
            &results.estimations.system_states_spherical,
            &results.estimations.system_states_spherical_max,
            &model.spatial_description.voxels.positions_mm,
            model.spatial_description.voxels.size_mm,
            &model.spatial_description.voxels.numbers,
            Some(path.as_path()),
            None,
            Some(StateSphericalPlotMode::ABS),
            None,
            None,
        )
        .unwrap();

        let fps = 20;
        let playback_speed = 0.1;

        let path = Path::new(COMMON_PATH).join("no_kalman").join("states.gif");
        states_spherical_plot_over_time(
            &results.estimations.system_states_spherical,
            &results.estimations.system_states_spherical_max,
            &model.spatial_description.voxels.positions_mm,
            model.spatial_description.voxels.size_mm,
            simulation_config.sample_rate_hz,
            &model.spatial_description.voxels.numbers,
            Some(path.as_path()),
            Some(PlotSlice::Z(0)),
            Some(StateSphericalPlotMode::ABS),
            Some(playback_speed),
            Some(fps),
        )
        .unwrap();
    }

    #[test]
    #[ignore]
    fn loss_decreases_kalman_and_plot() {
        setup(Some("full_kalman"));
        let mut simulation_config = SimulationConfig::default();
        simulation_config.model.common.pathological = true;
        simulation_config.model.common.sensor_array_geometry = SensorArrayGeometry::Cube;
        simulation_config.model.common.sensor_array_motion = SensorArrayMotion::Static;
        let data = Data::from_simulation_config(&simulation_config)
            .expect("Model parameters to be valid.");

        let mut algorithm_config = Algorithm {
            update_kalman_gain: true,
            epochs: 10,
            ..Default::default()
        };
        algorithm_config.model.common.apply_system_update = true;
        algorithm_config.model.common.sensor_array_geometry = SensorArrayGeometry::Cube;
        algorithm_config.model.common.sensor_array_motion = SensorArrayMotion::Static;

        let mut model = Model::from_model_config(
            &algorithm_config.model,
            simulation_config.sample_rate_hz,
            simulation_config.duration_s,
        )
        .expect("Model parameters to be valid.");

        let mut results = Results::new(
            algorithm_config.epochs,
            model
                .functional_description
                .control_function_values
                .values
                .shape()[0],
            model.spatial_description.sensors.count(),
            model.spatial_description.voxels.count_states(),
            simulation_config
                .model
                .common
                .sensor_array_motion_steps
                .iter()
                .product(),
            algorithm_config.optimizer,
        );

        run(
            &mut model.functional_description,
            &mut results,
            &data,
            &algorithm_config,
        );

        let path = Path::new(COMMON_PATH).join("full_kalman").join("loss.png");
        standard_y_plot(
            &results.metrics.loss.values,
            Path::new(path.as_path()),
            "Loss",
            "Loss",
            "Step",
        )
        .unwrap();
        let path = Path::new(COMMON_PATH)
            .join("full_kalman")
            .join("loss_epoch.png");
        standard_y_plot(
            &results.metrics.loss_epoch.values,
            Path::new(path.as_path()),
            "Sum Loss Per Epoch",
            "Loss",
            "Epoch",
        )
        .unwrap();

        let path = Path::new(COMMON_PATH)
            .join("full_kalman")
            .join("states_max.png");
        states_spherical_plot(
            &results.estimations.system_states_spherical,
            &results.estimations.system_states_spherical_max,
            &model.spatial_description.voxels.positions_mm,
            model.spatial_description.voxels.size_mm,
            &model.spatial_description.voxels.numbers,
            Some(path.as_path()),
            None,
            Some(StateSphericalPlotMode::ABS),
            None,
            None,
        )
        .unwrap();

        let fps = 20;
        let playback_speed = 0.1;

        let path = Path::new(COMMON_PATH)
            .join("full_kalman")
            .join("states.gif");
        states_spherical_plot_over_time(
            &results.estimations.system_states_spherical,
            &results.estimations.system_states_spherical_max,
            &model.spatial_description.voxels.positions_mm,
            model.spatial_description.voxels.size_mm,
            simulation_config.sample_rate_hz,
            &model.spatial_description.voxels.numbers,
            Some(path.as_path()),
            Some(PlotSlice::Z(0)),
            Some(StateSphericalPlotMode::ABS),
            Some(playback_speed),
            Some(fps),
        )
        .unwrap();
    }

    #[test]
    fn pseudo_inverse_success() {
        let mut simulation_config = SimulationConfig::default();
        simulation_config.model.common.sensor_array_geometry = SensorArrayGeometry::Cube;
        simulation_config.model.common.sensor_array_motion = SensorArrayMotion::Static;
        let data = Data::from_simulation_config(&simulation_config)
            .expect("Model parameters to be valid.");

        let algorithm_config = Algorithm::default();

        let model = Model::from_model_config(
            &algorithm_config.model,
            simulation_config.sample_rate_hz,
            simulation_config.duration_s,
        )
        .expect("Model parameters to be valid.");

        let mut results = Results::new(
            algorithm_config.epochs,
            model
                .functional_description
                .control_function_values
                .values
                .shape()[0],
            model.spatial_description.sensors.count(),
            model.spatial_description.voxels.count_states(),
            simulation_config
                .model
                .common
                .sensor_array_motion_steps
                .iter()
                .product(),
            algorithm_config.optimizer,
        );

        calculate_pseudo_inverse(
            &model.functional_description,
            &mut results,
            &data,
            &algorithm_config,
        );
    }
}
