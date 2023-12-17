use nalgebra::{DMatrix, SVD};
use ndarray::{s, Array1};

use self::estimation::{
    calculate_delays_delta_flat, calculate_delays_delta_normal, calculate_gains_delta_flat,
    calculate_gains_delta_normal, calculate_post_update_residuals, calculate_residuals,
    calculate_system_states_delta, calculate_system_update_flat, calculate_system_update_normal,
    prediction::{calculate_system_prediction_flat, calculate_system_prediction_normal},
};

use super::{
    config::algorithm::Algorithm,
    data::{shapes::ArraySystemStates, Data},
    model::functional::FunctionalDescription,
    scenario::results::Results,
};

pub mod estimation;
pub mod metrics;
pub mod refinement;

#[allow(clippy::missing_panics_doc)]
pub fn calculate_pseudo_inverse(
    functional_description: &FunctionalDescription,
    results: &mut Results,
    data: &Data,
    config: &Algorithm,
) {
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

    let estimations_normal = results
        .estimations_normal
        .as_mut()
        .expect("Estimations normal to be some.");
    let derivatives_normal = results
        .derivatives_normal
        .as_mut()
        .expect("Derivatives normal to be some.");

    for time_index in 0..estimations_normal.system_states.values.shape()[0] {
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

        estimations_normal
            .system_states
            .values
            .slice_mut(s![time_index, ..])
            .assign(&system_states);

        estimations_normal
            .measurements
            .values
            .slice_mut(s![time_index, ..])
            .assign(
                &functional_description.measurement_matrix.values.dot(
                    &estimations_normal
                        .system_states
                        .values
                        .slice(s![time_index, ..]),
                ),
            );

        calculate_residuals(
            &mut estimations_normal.residuals,
            &estimations_normal.measurements,
            data.get_measurements(),
            time_index,
        );

        derivatives_normal.calculate(
            functional_description,
            estimations_normal,
            config,
            time_index,
        );

        calculate_post_update_residuals(
            &mut estimations_normal.post_update_residuals,
            &functional_description.measurement_matrix,
            &estimations_normal.system_states,
            data.get_measurements(),
            time_index,
        );
        calculate_system_states_delta(
            &mut estimations_normal.system_states_delta,
            &estimations_normal.system_states,
            data.get_system_states(),
            time_index,
        );
        results.metrics.calculate_step_normal(
            estimations_normal,
            derivatives_normal,
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
///
/// # Panics
/// If `ap_params_normal` is none
#[allow(clippy::too_many_lines)]
pub fn run_epoch(
    functional_description: &mut FunctionalDescription,
    results: &mut Results,
    data: &Data,
    config: &Algorithm,
    epoch_index: usize,
) {
    let num_steps = if functional_description.ap_params_normal.is_some() {
        results
            .estimations_normal
            .as_mut()
            .expect("Estimation normal to be some.")
            .reset();
        results
            .derivatives_normal
            .as_mut()
            .expect("Derivatives normal to be some.")
            .reset();
        results
            .estimations_normal
            .as_mut()
            .expect("Estimations normal to be some.")
            .system_states
            .values
            .shape()[0]
    } else {
        results
            .estimations_flat
            .as_mut()
            .expect("Estimation flat to be some.")
            .reset();
        results
            .derivatives_flat
            .as_mut()
            .expect("Derivatives flat to be some.")
            .reset();
        results
            .estimations_flat
            .as_ref()
            .expect("Estimations flat to be some.")
            .system_states
            .values
            .shape()[0]
    };
    let mut batch = match config.batch_size {
        0 => None,
        _ => Some((epoch_index * num_steps) % config.batch_size),
    };

    for time_index in 0..num_steps {
        if functional_description.ap_params_normal.is_some() {
            let estimations_normal = results
                .estimations_normal
                .as_mut()
                .expect("Estimations normal to be some");
            let derivatives_normal = results
                .derivatives_normal
                .as_mut()
                .expect("Derivatives normal to be some");
            calculate_system_prediction_normal(
                &mut estimations_normal.ap_outputs,
                &mut estimations_normal.system_states,
                &mut estimations_normal.measurements,
                functional_description,
                time_index,
            );
            calculate_residuals(
                &mut estimations_normal.residuals,
                &estimations_normal.measurements,
                data.get_measurements(),
                time_index,
            );
            if config.constrain_system_states {
                constrain_system_states(
                    &mut estimations_normal.system_states,
                    time_index,
                    config.state_clamping_threshold,
                );
            }

            derivatives_normal.calculate(
                functional_description,
                estimations_normal,
                config,
                time_index,
            );

            if config.model.apply_system_update {
                calculate_system_update_normal(
                    estimations_normal,
                    time_index,
                    functional_description,
                    config,
                );
            }

            calculate_post_update_residuals(
                &mut estimations_normal.post_update_residuals,
                &functional_description.measurement_matrix,
                &estimations_normal.system_states,
                data.get_measurements(),
                time_index,
            );
            calculate_system_states_delta(
                &mut estimations_normal.system_states_delta,
                &estimations_normal.system_states,
                data.get_system_states(),
                time_index,
            );
            calculate_gains_delta_normal(
                &mut estimations_normal.gains_delta,
                &functional_description
                    .ap_params_normal
                    .as_ref()
                    .expect("AP Params to be some.")
                    .gains,
                data.get_gains_normal(),
            );
            calculate_delays_delta_normal(
                &mut estimations_normal.delays_delta,
                &functional_description
                    .ap_params_normal
                    .as_ref()
                    .expect("Ap parms to be some.")
                    .delays,
                data.get_delays_normal(),
                &functional_description
                    .ap_params_normal
                    .as_ref()
                    .expect("Ap params to be some.")
                    .coefs,
                data.get_coefs_normal(),
            );
            results.metrics.calculate_step_normal(
                estimations_normal,
                derivatives_normal,
                config.regularization_strength,
                time_index,
            );
            if let Some(n) = batch.as_mut() {
                *n += 1;
                if *n == config.batch_size {
                    functional_description
                        .ap_params_normal
                        .as_mut()
                        .expect("AP params normal to be some.")
                        .update(
                            derivatives_normal,
                            config,
                            estimations_normal.system_states.values.shape()[0],
                        );
                    derivatives_normal.reset();
                    estimations_normal.kalman_gain_converged = false;
                    *n = 0;
                }
            }
        } else {
            let estimations_flat = results
                .estimations_flat
                .as_mut()
                .expect("Estimations flat to be some");
            let derivatives_flat = results
                .derivatives_flat
                .as_mut()
                .expect("Derivatives flat to be some");
            calculate_system_prediction_flat(
                &mut estimations_flat.ap_outputs,
                &mut estimations_flat.system_states,
                &mut estimations_flat.measurements,
                functional_description,
                time_index,
            );
            calculate_residuals(
                &mut estimations_flat.residuals,
                &estimations_flat.measurements,
                data.get_measurements(),
                time_index,
            );
            if config.constrain_system_states {
                constrain_system_states(
                    &mut estimations_flat.system_states,
                    time_index,
                    config.state_clamping_threshold,
                );
            }

            derivatives_flat.calculate(
                functional_description,
                estimations_flat,
                config,
                time_index,
            );

            if config.model.apply_system_update {
                calculate_system_update_flat(
                    estimations_flat,
                    time_index,
                    functional_description,
                    config,
                );
            }

            calculate_post_update_residuals(
                &mut estimations_flat.post_update_residuals,
                &functional_description.measurement_matrix,
                &estimations_flat.system_states,
                data.get_measurements(),
                time_index,
            );
            calculate_system_states_delta(
                &mut estimations_flat.system_states_delta,
                &estimations_flat.system_states,
                data.get_system_states(),
                time_index,
            );
            calculate_gains_delta_flat(
                &mut estimations_flat.gains_delta,
                &functional_description
                    .ap_params_flat
                    .as_ref()
                    .expect("AP Params to be some.")
                    .gains,
                data.get_gains_flat(),
            );
            calculate_delays_delta_flat(
                &mut estimations_flat.delays_delta,
                &functional_description
                    .ap_params_flat
                    .as_ref()
                    .expect("Ap parms flat to be some.")
                    .delays,
                data.get_delays_flat(),
                &functional_description
                    .ap_params_flat
                    .as_ref()
                    .expect("Ap params flat to be some.")
                    .coefs,
                data.get_coefs_flat(),
            );
            results.metrics.calculate_step_flat(
                estimations_flat,
                derivatives_flat,
                config.regularization_strength,
                time_index,
            );
            if let Some(n) = batch.as_mut() {
                *n += 1;
                if *n == config.batch_size {
                    functional_description
                        .ap_params_flat
                        .as_mut()
                        .expect("AP params flat to be some.")
                        .update(
                            derivatives_flat,
                            config,
                            estimations_flat.system_states.values.shape()[0],
                        );
                    derivatives_flat.reset();
                    estimations_flat.kalman_gain_converged = false;
                    *n = 0;
                }
            }
        }
    }
    if batch.is_none() {
        if functional_description.ap_params_normal.is_some() {
            functional_description
                .ap_params_normal
                .as_mut()
                .expect("AP params normal to be some.")
                .update(
                    results
                        .derivatives_normal
                        .as_ref()
                        .expect("Derivatives normal to be some"),
                    config,
                    results
                        .estimations_normal
                        .as_ref()
                        .expect("Estimations normal to be some.")
                        .system_states
                        .values
                        .shape()[0],
                );
        } else {
            functional_description
                .ap_params_flat
                .as_mut()
                .expect("AP params flat to be some.")
                .update(
                    results
                        .derivatives_flat
                        .as_ref()
                        .expect("Derivatives flat to be some"),
                    config,
                    results
                        .estimations_flat
                        .as_ref()
                        .expect("Estimations flat to be some.")
                        .system_states
                        .values
                        .shape()[0],
                );
        }
    }
    results.metrics.calculate_epoch(epoch_index);
}

fn constrain_system_states(
    system_states: &mut ArraySystemStates,
    time_index: usize,
    clamping_threshold: f32,
) {
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

#[allow(dead_code)]
fn run(
    functional_description: &mut FunctionalDescription,
    results: &mut Results,
    data: &Data,
    algorithm_config: &Algorithm,
) {
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

#[cfg(test)]
mod test {

    use approx::assert_relative_eq;
    use ndarray::Dim;
    use ndarray_stats::QuantileExt;

    use crate::core::config::algorithm::Algorithm as AlgorithmConfig;
    use crate::core::config::simulation::Simulation as SimulationConfig;
    use crate::core::model::Model;

    use crate::vis::plotting::matrix::{plot_states_max_normal, plot_states_over_time};
    use crate::vis::plotting::time::standard_y_plot;

    use super::*;

    #[test]
    fn run_epoch_no_crash() {
        let number_of_states = 3000;
        let number_of_sensors = 300;
        let number_of_steps = 3;
        let number_of_epochs = 10;
        let mut config = AlgorithmConfig::default();
        config.model.use_flat_arrays = false;
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
            config.model.use_flat_arrays,
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

        let mut algorithm_config = AlgorithmConfig {
            epochs: 3,
            ..Default::default()
        };
        algorithm_config.model.use_flat_arrays = false;
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
            algorithm_config.model.use_flat_arrays,
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

        let mut algorithm_config = Algorithm::default();
        algorithm_config.model.use_flat_arrays = false;

        let mut model = Model::from_model_config(
            &algorithm_config.model,
            simulation_config.sample_rate_hz,
            simulation_config.duration_s,
        )
        .expect("Model parameters to be valid.");
        algorithm_config.epochs = 3;
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
            algorithm_config.model.use_flat_arrays,
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
        algorithm_config.model.use_flat_arrays = false;

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
            algorithm_config.model.use_flat_arrays,
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

        plot_states_max_normal(
            &results
                .estimations_normal
                .as_ref()
                .expect("Estimations normal to be some.")
                .system_states,
            &model.spatial_description.voxels,
            "tests/algorith_states_max",
            "Maximum Estimated Current Densities",
        );

        let fps = 20;
        let playback_speed = 0.1;

        plot_states_over_time(
            &results
                .estimations_normal
                .as_ref()
                .expect("Estimations normal to be some.")
                .system_states,
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
            ..Default::default()
        };
        algorithm_config.model.use_flat_arrays = false;

        let mut model = Model::from_model_config(
            &algorithm_config.model,
            simulation_config.sample_rate_hz,
            simulation_config.duration_s,
        )
        .expect("Model parameters to be valid.");
        algorithm_config.epochs = 3;
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
            algorithm_config.model.use_flat_arrays,
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
        algorithm_config.model.use_flat_arrays = false;

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
            algorithm_config.model.use_flat_arrays,
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
        algorithm_config.model.use_flat_arrays = false;

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
            algorithm_config.model.use_flat_arrays,
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

        plot_states_max_normal(
            &results
                .estimations_normal
                .as_ref()
                .expect("Estimations normal to be some.")
                .system_states,
            &model.spatial_description.voxels,
            "tests/algorith_no_update_states_max",
            "Maximum Estimated Current Densities",
        );

        let fps = 20;
        let playback_speed = 0.1;

        plot_states_over_time(
            &results
                .estimations_normal
                .as_ref()
                .expect("Estimations normal to be some.")
                .system_states,
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
        algorithm_config.model.use_flat_arrays = false;

        let mut model = Model::from_model_config(
            &algorithm_config.model,
            simulation_config.sample_rate_hz,
            simulation_config.duration_s,
        )
        .expect("Model parameters to be valid.");
        model
            .functional_description
            .ap_params_normal
            .as_mut()
            .expect("Ap parmas to be some.")
            .gains
            .values *= 2.0;
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
            algorithm_config.model.use_flat_arrays,
        );

        run(
            &mut model.functional_description,
            &mut results,
            &data,
            &algorithm_config,
        );

        results
            .estimations_normal
            .expect("Estimations normal to be some.")
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
        algorithm_config.model.use_flat_arrays = false;

        let mut model = Model::from_model_config(
            &algorithm_config.model,
            simulation_config.sample_rate_hz,
            simulation_config.duration_s,
        )
        .expect("Model params to be valid.");
        model
            .functional_description
            .ap_params_normal
            .as_mut()
            .expect("AP params to be some.")
            .gains
            .values *= 2.0;
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
            algorithm_config.model.use_flat_arrays,
        );

        run(
            &mut model.functional_description,
            &mut results,
            &data,
            &algorithm_config,
        );

        assert!(
            *results
                .estimations_normal
                .expect("Estimations normal to be some.")
                .system_states
                .values
                .max_skipnan()
                > 2.0
        );
    }

    #[test]
    fn pseudo_inverse_success() {
        let simulation_config = SimulationConfig::default();
        let data = Data::from_simulation_config(&simulation_config)
            .expect("Model parameters to be valid.");

        let mut algorithm_config = Algorithm::default();
        algorithm_config.model.use_flat_arrays = false;

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
            algorithm_config.model.use_flat_arrays,
        );

        calculate_pseudo_inverse(
            &model.functional_description,
            &mut results,
            &data,
            &algorithm_config,
        );
    }

    #[test]
    fn loss_decreases_kalman_flat() {
        let simulation_config = SimulationConfig::default();
        let data = Data::from_simulation_config(&simulation_config)
            .expect("Model parameters to be valid.");

        let mut algorithm_config = Algorithm::default();
        algorithm_config.model.use_flat_arrays = false;

        let mut model = Model::from_model_config(
            &algorithm_config.model,
            simulation_config.sample_rate_hz,
            simulation_config.duration_s,
        )
        .expect("Model params to be valid.");
        model
            .functional_description
            .ap_params_normal
            .as_mut()
            .expect("AP params to be some.")
            .gains
            .values *= 2.0;
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
            algorithm_config.model.use_flat_arrays,
        );

        run(
            &mut model.functional_description,
            &mut results,
            &data,
            &algorithm_config,
        );
        let mut simulation_config = SimulationConfig::default();
        simulation_config.model.use_flat_arrays = true;
        simulation_config.model.pathological = true;
        let data = Data::from_simulation_config(&simulation_config)
            .expect("Model parameters to be valid.");

        let mut algorithm_config = Algorithm {
            calculate_kalman_gain: true,
            ..Default::default()
        };
        algorithm_config.model.use_flat_arrays = true;

        let mut model = Model::from_model_config(
            &algorithm_config.model,
            simulation_config.sample_rate_hz,
            simulation_config.duration_s,
        )
        .expect("Model parameters to be valid.");
        algorithm_config.epochs = 3;
        algorithm_config.model.apply_system_update = true;
        algorithm_config.calculate_kalman_gain = true;

        let mut results = Results::new(
            algorithm_config.epochs,
            model
                .functional_description
                .control_function_values
                .values
                .shape()[0],
            model.spatial_description.sensors.count(),
            model.spatial_description.voxels.count_states(),
            algorithm_config.model.use_flat_arrays,
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
    fn full_algo_equivalent() {
        //normal
        let mut simulation_config_normal = SimulationConfig::default();
        simulation_config_normal.model.pathological = true;
        let data_normal = Data::from_simulation_config(&simulation_config_normal)
            .expect("Model parameters to be valid.");

        let mut algorithm_config_normal = Algorithm {
            calculate_kalman_gain: true,
            ..Default::default()
        };
        algorithm_config_normal.model.use_flat_arrays = false;

        let mut model_normal = Model::from_model_config(
            &algorithm_config_normal.model,
            simulation_config_normal.sample_rate_hz,
            simulation_config_normal.duration_s,
        )
        .expect("Model parameters to be valid.");
        algorithm_config_normal.epochs = 5;
        algorithm_config_normal.model.apply_system_update = true;
        algorithm_config_normal.learning_rate = 20.0;

        let mut results_normal = Results::new(
            algorithm_config_normal.epochs,
            model_normal
                .functional_description
                .control_function_values
                .values
                .shape()[0],
            model_normal.spatial_description.sensors.count(),
            model_normal.spatial_description.voxels.count_states(),
            algorithm_config_normal.model.use_flat_arrays,
        );

        run(
            &mut model_normal.functional_description,
            &mut results_normal,
            &data_normal,
            &algorithm_config_normal,
        );

        (0..algorithm_config_normal.epochs - 1).for_each(|i| {
            assert!(
                results_normal.metrics.loss_epoch.values[i]
                    > results_normal.metrics.loss_epoch.values[i + 1]
            );
        });

        // flat
        let mut simulation_config_flat = SimulationConfig::default();
        simulation_config_flat.model.use_flat_arrays = true;
        simulation_config_flat.model.pathological = true;
        let data_flat = Data::from_simulation_config(&simulation_config_flat)
            .expect("Model parameters to be valid.");

        let mut algorithm_config_flat = Algorithm {
            calculate_kalman_gain: true,
            ..Default::default()
        };
        algorithm_config_flat.model.use_flat_arrays = true;

        let mut model_flat = Model::from_model_config(
            &algorithm_config_flat.model,
            simulation_config_flat.sample_rate_hz,
            simulation_config_flat.duration_s,
        )
        .expect("Model parameters to be valid.");
        algorithm_config_flat.epochs = 5;
        algorithm_config_flat.model.apply_system_update = true;
        algorithm_config_flat.learning_rate = 20.0;

        let mut results_flat = Results::new(
            algorithm_config_flat.epochs,
            model_flat
                .functional_description
                .control_function_values
                .values
                .shape()[0],
            model_flat.spatial_description.sensors.count(),
            model_flat.spatial_description.voxels.count_states(),
            algorithm_config_flat.model.use_flat_arrays,
        );

        run(
            &mut model_flat.functional_description,
            &mut results_flat,
            &data_flat,
            &algorithm_config_flat,
        );
        let loss_flat = &results_flat.metrics.loss_epoch.values;
        let loss_normal = &results_normal.metrics.loss_epoch.values;
        println!("flat: {loss_flat:?}");
        println!("normal: {loss_normal:?}");

        (0..algorithm_config_flat.epochs - 1).for_each(|i| {
            assert_relative_eq!(
                results_flat.metrics.loss_epoch.values[i],
                results_normal.metrics.loss_epoch.values[i]
            );
        });
    }
}
