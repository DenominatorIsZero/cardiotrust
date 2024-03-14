pub mod estimation;
pub mod metrics;
pub mod refinement;

use nalgebra::{inf_sup, DMatrix, SVD};
use ndarray::{s, Array1};
use tracing::{debug, info, trace};

use self::estimation::{
    calculate_delays_delta, calculate_gains_delta, calculate_post_update_residuals,
    calculate_residuals, calculate_system_states_delta, calculate_system_update,
    prediction::calculate_system_prediction,
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
    let rows = functional_description.measurement_matrix.values.shape()[0];
    let columns = functional_description.measurement_matrix.values.shape()[1];
    let measurement_matrix = DMatrix::from_row_slice(
        rows,
        columns,
        functional_description
            .measurement_matrix
            .values
            .as_slice()
            .expect("Slice to be some"),
    );

    let decomposition = SVD::new_unordered(measurement_matrix, true, true);

    let estimations = &mut results.estimations;
    let derivatives = &mut results.derivatives;

    for time_index in 0..estimations.system_states.values.shape()[0] {
        let rows = data.get_measurements().values.shape()[1];
        let measurements = DMatrix::from_row_slice(
            rows,
            1,
            data.get_measurements()
                .values
                .slice(s![time_index, ..])
                .as_slice()
                .expect("Slice to be some."),
        );

        let system_states = decomposition
            .solve(&measurements, 1e-5)
            .expect("SVD to be computed.");

        let system_states = Array1::from_iter(system_states.as_slice().iter().copied());

        estimations
            .system_states
            .values
            .slice_mut(s![time_index, ..])
            .assign(&system_states);

        estimations
            .measurements
            .values
            .slice_mut(s![time_index, ..])
            .assign(
                &functional_description
                    .measurement_matrix
                    .values
                    .dot(&estimations.system_states.values.slice(s![time_index, ..])),
            );

        calculate_residuals(
            &mut estimations.residuals,
            &estimations.measurements,
            data.get_measurements(),
            time_index,
        );

        derivatives.calculate(functional_description, estimations, config, time_index);

        calculate_post_update_residuals(
            &mut estimations.post_update_residuals,
            &functional_description.measurement_matrix,
            &estimations.system_states,
            data.get_measurements(),
            time_index,
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
    results.estimations.reset();
    results.derivatives.reset();
    let num_steps = results.estimations.system_states.values.shape()[0];
    let mut batch = match config.batch_size {
        0 => None,
        _ => Some((epoch_index * num_steps) % config.batch_size),
    };

    for time_index in 0..num_steps {
        let estimations = &mut results.estimations;
        let derivatives = &mut results.derivatives;
        calculate_system_prediction(
            &mut estimations.ap_outputs,
            &mut estimations.system_states,
            &mut estimations.measurements,
            functional_description,
            time_index,
        );
        calculate_residuals(
            &mut estimations.residuals,
            &estimations.measurements,
            data.get_measurements(),
            time_index,
        );
        if config.constrain_system_states {
            constrain_system_states(
                &mut estimations.system_states,
                time_index,
                config.state_clamping_threshold,
            );
        }

        derivatives.calculate(functional_description, estimations, config, time_index);

        if config.model.apply_system_update {
            calculate_system_update(estimations, time_index, functional_description, config);
        }

        calculate_post_update_residuals(
            &mut estimations.post_update_residuals,
            &functional_description.measurement_matrix,
            &estimations.system_states,
            data.get_measurements(),
            time_index,
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
        results.metrics.calculate_step(
            estimations,
            derivatives,
            config.regularization_strength,
            time_index,
        );
        if let Some(n) = batch.as_mut() {
            *n += 1;
            if *n == config.batch_size {
                functional_description.ap_params.update(
                    derivatives,
                    config,
                    estimations.system_states.values.shape()[0],
                );
                derivatives.reset();
                estimations.kalman_gain_converged = false;
                *n = 0;
            }
        }
    }
    if batch.is_none() {
        functional_description.ap_params.update(
            &results.derivatives,
            config,
            results.estimations.system_states.values.shape()[0],
        );
    }
    results.metrics.calculate_epoch(epoch_index);
}

#[tracing::instrument(level = "trace")]
fn constrain_system_states(
    system_states: &mut ArraySystemStates,
    time_index: usize,
    clamping_threshold: f32,
) {
    trace!("Constraining system states");
    for state_index in (0..system_states.values.raw_dim()[1]).step_by(3) {
        let sum = system_states.values[[time_index, state_index]].abs()
            + system_states.values[[time_index, state_index + 1]].abs()
            + system_states.values[[time_index, state_index + 2]].abs();
        if sum > clamping_threshold {
            let factor = clamping_threshold / sum;
            system_states.values[[time_index, state_index]] *= factor;
            system_states.values[[time_index, state_index + 1]] *= factor;
            system_states.values[[time_index, state_index + 2]] *= factor;
        }
    }
}

#[cfg(test)]
mod test {

    use ndarray::Dim;
    use ndarray_stats::QuantileExt;

    use crate::core::config::algorithm::Algorithm as AlgorithmConfig;
    use crate::core::config::simulation::Simulation as SimulationConfig;
    use crate::core::model::Model;

    use crate::vis::plotting::matrix::{plot_states_max, plot_states_over_time};
    use crate::vis::plotting::time::standard_y_plot;

    use super::*;

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

        let mut functional_description = FunctionalDescription::empty(
            number_of_states,
            number_of_sensors,
            number_of_steps,
            voxels_in_dims,
        );
        let mut results = Results::new(
            number_of_epochs,
            number_of_steps,
            number_of_sensors,
            number_of_states,
        );
        let data = Data::empty(
            number_of_sensors,
            number_of_states,
            number_of_steps,
            voxels_in_dims,
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
        let voxels_in_dims = Dim([1000, 1, 1]);

        let algorithm_config = AlgorithmConfig {
            epochs: 3,
            ..Default::default()
        };
        let mut functional_description = FunctionalDescription::empty(
            number_of_states,
            number_of_sensors,
            number_of_steps,
            voxels_in_dims,
        );
        let mut results = Results::new(
            algorithm_config.epochs,
            number_of_steps,
            number_of_sensors,
            number_of_states,
        );
        let data = Data::empty(
            number_of_sensors,
            number_of_states,
            number_of_steps,
            voxels_in_dims,
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
        simulation_config.model.pathological = true;
        let data = Data::from_simulation_config(&simulation_config)
            .expect("Model parameters to be valid.");

        let mut algorithm_config = Algorithm {
            learning_rate: 10.0,
            epochs: 3,
            ..Default::default()
        };
        algorithm_config.model.apply_system_update = true;

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
        let mut simulation_config = SimulationConfig::default();
        simulation_config.model.pathological = true;
        let data = Data::from_simulation_config(&simulation_config)
            .expect("Model parameters to be valid.");

        let mut algorithm_config = Algorithm::default();

        let mut model = Model::from_model_config(
            &algorithm_config.model,
            simulation_config.sample_rate_hz,
            simulation_config.duration_s,
        )
        .expect("Model paramters to be valid");
        algorithm_config.epochs = 10;
        algorithm_config.model.apply_system_update = true;

        let mut results = Results::new(
            algorithm_config.epochs,
            model
                .functional_description
                .control_function_values
                .values
                .shape()[0],
            model.spatial_description.sensors.count(),
            model.spatial_description.voxels.count_states(),
        );

        run(
            &mut model.functional_description,
            &mut results,
            &data,
            &algorithm_config,
        );

        standard_y_plot(
            &results.metrics.loss.values,
            "tests/algorithm_loss",
            "Loss",
            "Loss",
            "Step",
        );
        standard_y_plot(
            &results.metrics.loss_epoch.values,
            "tests/algorithm_loss_epoch",
            "Sum Loss Per Epoch",
            "Loss",
            "Epoch",
        );

        plot_states_max(
            &results.estimations.system_states,
            &model.spatial_description.voxels,
            "tests/algorith_states_max",
            "Maximum Estimated Current Densities",
        );

        let fps = 20;
        let playback_speed = 0.1;

        plot_states_over_time(
            &results.estimations.system_states,
            &model.spatial_description.voxels,
            fps,
            playback_speed,
            "tests/algorithm_states",
            "Estimated Current Densities",
        );

        (0..algorithm_config.epochs - 1).for_each(|i| {
            assert!(
                results.metrics.loss_mse_epoch.values[i]
                    > results.metrics.loss_mse_epoch.values[i + 1]
            );
        });
    }

    #[test]
    fn loss_decreases_kalman() {
        let mut simulation_config = SimulationConfig::default();
        simulation_config.model.pathological = true;
        let data = Data::from_simulation_config(&simulation_config)
            .expect("Model parameters to be valid.");

        let mut algorithm_config = Algorithm {
            calculate_kalman_gain: true,
            learning_rate: 10.0,
            epochs: 3,
            ..Default::default()
        };
        algorithm_config.model.apply_system_update = true;

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
    fn loss_decreases_no_update() {
        let mut simulation_config = SimulationConfig::default();
        simulation_config.model.pathological = true;
        let data = Data::from_simulation_config(&simulation_config)
            .expect("Model parameters to be valid.");

        let mut algorithm_config = Algorithm::default();

        let mut model = Model::from_model_config(
            &algorithm_config.model,
            simulation_config.sample_rate_hz,
            simulation_config.duration_s,
        )
        .expect("Model parameters to be valid.");
        algorithm_config.epochs = 3;
        algorithm_config.model.apply_system_update = false;

        let mut results = Results::new(
            algorithm_config.epochs,
            model
                .functional_description
                .control_function_values
                .values
                .shape()[0],
            model.spatial_description.sensors.count(),
            model.spatial_description.voxels.count_states(),
        );

        run(
            &mut model.functional_description,
            &mut results,
            &data,
            &algorithm_config,
        );

        (0..algorithm_config.epochs - 1).for_each(|i| {
            assert!(
                results.metrics.loss_mse_epoch.values[i]
                    > results.metrics.loss_mse_epoch.values[i + 1]
            );
        });
    }

    #[test]
    #[ignore]
    fn loss_decreases_no_update_and_plot() {
        let mut simulation_config = SimulationConfig::default();
        simulation_config.model.pathological = true;
        let data = Data::from_simulation_config(&simulation_config)
            .expect("Model parameters to be valid.");

        let mut algorithm_config = Algorithm::default();

        let mut model = Model::from_model_config(
            &algorithm_config.model,
            simulation_config.sample_rate_hz,
            simulation_config.duration_s,
        )
        .expect("Model parameters to be valid.");
        algorithm_config.epochs = 5;
        algorithm_config.model.apply_system_update = false;

        let mut results = Results::new(
            algorithm_config.epochs,
            model
                .functional_description
                .control_function_values
                .values
                .shape()[0],
            model.spatial_description.sensors.count(),
            model.spatial_description.voxels.count_states(),
        );

        run(
            &mut model.functional_description,
            &mut results,
            &data,
            &algorithm_config,
        );

        standard_y_plot(
            &results.metrics.loss.values,
            "tests/algorithm_no_update_loss",
            "Loss",
            "Loss",
            "Step",
        );
        standard_y_plot(
            &results.metrics.loss_epoch.values,
            "tests/algorithm_no_update_loss_epoch",
            "Sum Loss Per Epoch",
            "Loss",
            "Epoch",
        );

        plot_states_max(
            &results.estimations.system_states,
            &model.spatial_description.voxels,
            "tests/algorith_no_update_states_max",
            "Maximum Estimated Current Densities",
        );

        let fps = 20;
        let playback_speed = 0.1;

        plot_states_over_time(
            &results.estimations.system_states,
            &model.spatial_description.voxels,
            fps,
            playback_speed,
            "tests/algorithm_no_update_states",
            "Estimated Current Densities",
        );

        (0..algorithm_config.epochs - 1).for_each(|i| {
            assert!(
                results.metrics.loss_mse_epoch.values[i]
                    > results.metrics.loss_mse_epoch.values[i + 1]
            );
        });
    }

    #[test]
    fn current_density_constrained() {
        let simulation_config = SimulationConfig::default();
        let data = Data::from_simulation_config(&simulation_config)
            .expect("Model parameters to be valid.");

        let mut algorithm_config = Algorithm::default();

        let mut model = Model::from_model_config(
            &algorithm_config.model,
            simulation_config.sample_rate_hz,
            simulation_config.duration_s,
        )
        .expect("Model parameters to be valid.");
        model.functional_description.ap_params.gains.values *= 2.0;
        algorithm_config.epochs = 1;
        algorithm_config.model.apply_system_update = false;

        let mut results = Results::new(
            algorithm_config.epochs,
            model
                .functional_description
                .control_function_values
                .values
                .shape()[0],
            model.spatial_description.sensors.count(),
            model.spatial_description.voxels.count_states(),
        );

        run(
            &mut model.functional_description,
            &mut results,
            &data,
            &algorithm_config,
        );

        results
            .estimations
            .system_states
            .values
            .for_each(|v| assert!(*v <= 2.0, "{v} was greater than 2."));
    }

    #[test]
    fn current_density_not_constrained() {
        let simulation_config = SimulationConfig::default();
        let data = Data::from_simulation_config(&simulation_config)
            .expect("Model parameters to be valid.");

        let mut algorithm_config = Algorithm::default();

        let mut model = Model::from_model_config(
            &algorithm_config.model,
            simulation_config.sample_rate_hz,
            simulation_config.duration_s,
        )
        .expect("Model params to be valid.");
        model.functional_description.ap_params.gains.values *= 2.0;
        algorithm_config.epochs = 1;
        algorithm_config.constrain_system_states = false;
        algorithm_config.model.apply_system_update = false;

        let mut results = Results::new(
            algorithm_config.epochs,
            model
                .functional_description
                .control_function_values
                .values
                .shape()[0],
            model.spatial_description.sensors.count(),
            model.spatial_description.voxels.count_states(),
        );

        run(
            &mut model.functional_description,
            &mut results,
            &data,
            &algorithm_config,
        );

        assert!(*results.estimations.system_states.values.max_skipnan() > 2.0);
    }

    #[test]
    fn pseudo_inverse_success() {
        let simulation_config = SimulationConfig::default();
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
        );

        calculate_pseudo_inverse(
            &model.functional_description,
            &mut results,
            &data,
            &algorithm_config,
        );
    }
}
