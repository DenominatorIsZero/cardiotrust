use self::estimation::{
    calculate_delays_delta, calculate_gains_delta, calculate_residuals,
    calculate_system_prediction, calculate_system_states_delta, calculate_system_update,
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

/// Runs the algorithm for one epoch.
///
/// This includes calculating the system estimates
/// and performing one gradient descent step.
pub fn run_epoch(
    functional_description: &mut FunctionalDescription,
    results: &mut Results,
    data: &Data,
    config: &Algorithm,
    epoch_index: usize,
) {
    results.estimations.reset();
    results.derivatives.reset();
    for time_index in 0..results.estimations.system_states.values.shape()[0] {
        calculate_system_prediction(
            &mut results.estimations.ap_outputs,
            &mut results.estimations.system_states,
            &mut results.estimations.measurements,
            functional_description,
            time_index,
        );
        calculate_residuals(
            &mut results.estimations.residuals,
            &results.estimations.measurements,
            data.get_measurements(),
            time_index,
        );
        if config.constrain_system_states {
            constrain_system_states(&mut results.estimations.system_states);
        }
        results.derivatives.calculate(
            functional_description,
            &results.estimations,
            config,
            time_index,
        );
        if config.model.apply_system_update {
            calculate_system_update(
                &mut results.estimations,
                time_index,
                functional_description,
                config,
            );
        }
        calculate_system_states_delta(
            &mut results.estimations.system_states_delta,
            &results.estimations.system_states,
            data.get_system_states(),
            time_index,
        );
        calculate_gains_delta(
            &mut results.estimations.gains_delta,
            &functional_description.ap_params.gains,
            data.get_gains(),
        );
        calculate_delays_delta(
            &mut results.estimations.delays_delta,
            &functional_description.ap_params.delays,
            data.get_delays(),
            &functional_description.ap_params.coefs,
            data.get_coefs(),
        );
        results.metrics.calculate_step(
            &results.estimations,
            &results.derivatives,
            config.regularization_strength,
            time_index,
            epoch_index,
        );
    }
    functional_description
        .ap_params
        .update(&results.derivatives, config);
    results.metrics.calculate_epoch(epoch_index);
}

fn constrain_system_states(system_states: &mut ArraySystemStates) {
    system_states.values.iter_mut().for_each(|v| {
        *v = v.clamp(-2.0, 2.0);
    });
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

    use ndarray::Dim;
    use ndarray_stats::QuantileExt;

    use crate::core::config::algorithm::Algorithm as AlgorithmConfig;
    use crate::core::config::simulation::Simulation as SimulationConfig;
    use crate::core::model::Model;

    use crate::vis::plotting::matrix::{plot_states_max, plot_states_over_time};
    use crate::vis::plotting::time::standard_y_plot;

    use super::*;

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
        let data = Data::from_simulation_config(&simulation_config);

        let mut algorithm_config = Algorithm::default();

        let mut model = Model::from_model_config(
            &algorithm_config.model,
            simulation_config.sample_rate_hz,
            simulation_config.duration_s,
        )
        .unwrap();
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
        let data = Data::from_simulation_config(&simulation_config);

        let mut algorithm_config = Algorithm::default();

        let mut model = Model::from_model_config(
            &algorithm_config.model,
            simulation_config.sample_rate_hz,
            simulation_config.duration_s,
        )
        .unwrap();
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
        let data = Data::from_simulation_config(&simulation_config);

        let mut algorithm_config = Algorithm {
            calculate_kalman_gain: true,
            ..Default::default()
        };

        let mut model = Model::from_model_config(
            &algorithm_config.model,
            simulation_config.sample_rate_hz,
            simulation_config.duration_s,
        )
        .unwrap();
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
        let data = Data::from_simulation_config(&simulation_config);

        let mut algorithm_config = Algorithm::default();

        let mut model = Model::from_model_config(
            &algorithm_config.model,
            simulation_config.sample_rate_hz,
            simulation_config.duration_s,
        )
        .unwrap();
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
        let data = Data::from_simulation_config(&simulation_config);

        let mut algorithm_config = Algorithm::default();

        let mut model = Model::from_model_config(
            &algorithm_config.model,
            simulation_config.sample_rate_hz,
            simulation_config.duration_s,
        )
        .unwrap();
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
        let data = Data::from_simulation_config(&simulation_config);

        let mut algorithm_config = Algorithm::default();

        let mut model = Model::from_model_config(
            &algorithm_config.model,
            simulation_config.sample_rate_hz,
            simulation_config.duration_s,
        )
        .unwrap();
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
        let data = Data::from_simulation_config(&simulation_config);

        let mut algorithm_config = Algorithm::default();

        let mut model = Model::from_model_config(
            &algorithm_config.model,
            simulation_config.sample_rate_hz,
            simulation_config.duration_s,
        )
        .unwrap();
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

        assert!(*results.estimations.system_states.values.max().unwrap() > 2.0);
    }
}
