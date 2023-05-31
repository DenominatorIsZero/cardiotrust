use self::{
    estimation::{
        calculate_residuals, calculate_system_prediction, calculate_system_update, Estimations,
    },
    metrics::Metrics,
    refinement::derivation::Derivatives,
};

use super::{
    config::algorithm::Algorithm, data::Data, model::FunctionalDescription, results::Results,
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
            config.apply_system_update,
            epoch_index,
        );
    }
}

#[cfg(test)]
mod test {
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

        let mut functional_description =
            FunctionalDescription::new(number_of_states, number_of_sensors, number_of_steps);
        let mut estimations =
            Estimations::new(number_of_states, number_of_sensors, number_of_steps);
        let mut derivatives = Derivatives::new(number_of_states);
        let mut metrics = Metrics::new(number_of_epochs, number_of_steps);
        let data = Data::new(number_of_sensors, number_of_steps);

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
}
