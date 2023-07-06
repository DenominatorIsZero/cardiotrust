use self::{
    estimation::{
        calculate_residuals, calculate_system_prediction, calculate_system_update, Estimations,
    },
    metrics::Metrics,
    refinement::derivation::Derivatives,
};

use super::{
    config::algorithm::Algorithm, data::Data, model::functional::FunctionalDescription,
    results::Results,
};

pub mod estimation;
pub mod metrics;
pub mod refinement;

fn run_epoch(
    functional_description: &mut FunctionalDescription,
    estimations: &mut Estimations,
    derivatives: &mut Derivatives,
    metrics: &mut Metrics,
    data: &Data,
    learning_rate: f32,
    apply_system_update: bool,
    epoch_index: usize,
) {
    estimations.reset();
    derivatives.reset();
    for time_index in 0..estimations.system_states.values.shape()[0] {
        calculate_system_prediction(
            &mut estimations.ap_outputs,
            &mut estimations.system_states,
            &mut estimations.measurements,
            &functional_description,
            time_index,
        );
        calculate_residuals(
            &mut estimations.residuals,
            &estimations.measurements,
            data.get_measurements(),
            time_index,
        );
        derivatives.calculate(&functional_description, &estimations, time_index);
        if apply_system_update {
            calculate_system_update(
                &mut estimations.system_states,
                &estimations.residuals,
                &functional_description.kalman_gain,
                time_index,
            )
        }
        metrics.calculate_step(&estimations.residuals, time_index, epoch_index);
    }
    functional_description
        .ap_params
        .update(&derivatives, learning_rate);
    metrics.calculate_epoch(epoch_index);
}

fn run(
    functional_description: &mut FunctionalDescription,
    results: &mut Results,
    data: &Data,
    config: &Algorithm,
) {
    for epoch_index in 0..config.epochs {
        run_epoch(
            functional_description,
            &mut results.estimations,
            &mut results.derivatives,
            &mut results.metrics,
            data,
            config.learning_rate,
            config.model.apply_system_update,
            epoch_index,
        );
    }
}

#[cfg(test)]
mod test {
    use ndarray::Dim;

    use crate::core::config::model::Model as ModelConfig;
    use crate::core::config::simulation::Simulation as SimulationConfig;
    use crate::core::model::Model;

    use super::*;

    #[test]
    fn run_epoch_no_crash() {
        let number_of_states = 3000;
        let number_of_sensors = 300;
        let number_of_steps = 3;
        let number_of_epochs = 10;
        let learning_rate = 1e3;
        let apply_system_update = true;
        let epoch_index = 3;
        let voxels_in_dims = Dim([1000, 1, 1]);

        let mut functional_description = FunctionalDescription::empty(
            number_of_states,
            number_of_sensors,
            number_of_steps,
            voxels_in_dims,
        );
        let mut estimations =
            Estimations::new(number_of_states, number_of_sensors, number_of_steps);
        let mut derivatives = Derivatives::new(number_of_states);
        let mut metrics = Metrics::new(number_of_epochs, number_of_steps);
        let data = Data::empty(
            number_of_sensors,
            number_of_states,
            number_of_steps,
            voxels_in_dims,
        );

        run_epoch(
            &mut functional_description,
            &mut estimations,
            &mut derivatives,
            &mut metrics,
            &data,
            learning_rate,
            apply_system_update,
            epoch_index,
        );
    }

    #[test]
    fn run_no_crash() {
        let number_of_states = 3000;
        let number_of_sensors = 300;
        let number_of_steps = 3;
        let voxels_in_dims = Dim([1000, 1, 1]);

        let mut config = Algorithm::default();
        config.epochs = 3;
        let mut functional_description = FunctionalDescription::empty(
            number_of_states,
            number_of_sensors,
            number_of_steps,
            voxels_in_dims,
        );
        let mut results = Results::new(
            config.epochs,
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

        run(&mut functional_description, &mut results, &data, &config);
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

        println!(
            "0: {}, 1: {}",
            results.metrics.loss_epoch.values[0], results.metrics.loss_epoch.values[1]
        );
        assert!(results.metrics.loss_epoch.values[0] == results.metrics.loss_epoch.values[1]);
    }
}
