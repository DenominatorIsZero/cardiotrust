pub mod estimation;
pub mod gpu;
pub mod metrics;
pub mod refinement;
#[cfg(test)]
mod tests;

use nalgebra::{DMatrix, SVD};
use ndarray::{s, Array1};
use rand::{rng, seq::SliceRandom};
use refinement::derivation::{calculate_average_delays, calculate_batch_derivatives};
use tracing::{debug, trace};

use self::estimation::{calculate_residuals, prediction::calculate_system_prediction};
use super::{
    config::algorithm::Algorithm,
    data::{shapes::SystemStates, Data},
    model::functional::FunctionalDescription,
    scenario::results::Results,
};
use crate::core::algorithm::refinement::derivation::calculate_step_derivatives;

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
    let rows = functional_description.measurement_matrix.shape()[1];
    let columns = functional_description.measurement_matrix.shape()[2];
    let measurement_matrix = functional_description
        .measurement_matrix
        .slice(s![0, .., ..]);
    let measurement_matrix = DMatrix::from_row_slice(
        rows,
        columns,
        measurement_matrix.as_slice().expect("Slice to be some."),
    );

    let decomposition = SVD::new_unordered(measurement_matrix, true, true);

    let num_sensors = data.simulation.measurements.num_sensors();

    let estimations = &mut results.estimations;
    let derivatives = &mut results.derivatives;

    for step in 0..estimations.system_states.num_steps() {
        let mut estimated_measurements = estimations.measurements.at_beat_mut(0);
        let actual_measurements = data.simulation.measurements.at_beat(0);
        let mut estimated_system_states = estimations.system_states.at_step_mut(step);
        let mut estimated_measurements = estimated_measurements.at_step_mut(step);
        let actual_measurements = actual_measurements.at_step(step);

        let rows = data.simulation.measurements.num_sensors();
        let measurements =
            DMatrix::from_row_slice(rows, 1, actual_measurements.as_slice().unwrap());

        let system_states = decomposition
            .solve(&measurements, 1e-5)
            .expect("SVD to be computed.");

        let system_states = Array1::from_iter(system_states.as_slice().iter().copied());

        estimated_system_states.assign(&system_states);

        let measurement_matrix = functional_description.measurement_matrix.at_beat(0);

        estimated_measurements.assign(&measurement_matrix.dot(&*estimated_system_states));

        calculate_residuals(estimations, data, 0, step);

        calculate_step_derivatives(
            derivatives,
            estimations,
            functional_description,
            config,
            step,
            0,
            num_sensors,
        );

        metrics::calculate_step(
            &mut results.metrics,
            estimations,
            derivatives.maximum_regularization_sum,
            config.maximum_regularization_strength,
            step,
        );
    }
    metrics::calculate_batch(&mut results.metrics, 0);
}

/// Runs the algorithm for one epoch.
///
/// This includes calculating the system estimates
/// and performing one gradient descent step.
#[tracing::instrument(skip_all, level = "debug")]
pub fn run_epoch(results: &mut Results, batch_index: &mut usize, data: &Data, config: &Algorithm) {
    results.derivatives.reset();
    let num_steps = results.estimations.system_states.num_steps();
    let num_beats = data.simulation.measurements.num_beats();

    let mut batch = match config.batch_size {
        0 => None,
        _ => Some(0),
    };

    let mut beat_indices: Vec<usize> = (0..num_beats).collect();
    let mut rng = rng();
    beat_indices.shuffle(&mut rng);

    let estimations = &mut results.estimations;
    let derivatives = &mut results.derivatives;

    let num_sensors = data.simulation.measurements.num_sensors();

    for beat in beat_indices {
        estimations.reset();

        for step in 0..num_steps {
            calculate_system_prediction(
                estimations,
                &results.model.as_mut().unwrap().functional_description,
                beat,
                step,
            );

            calculate_residuals(estimations, data, beat, step);

            calculate_step_derivatives(
                derivatives,
                estimations,
                &results.model.as_mut().unwrap().functional_description,
                config,
                step,
                beat,
                num_sensors,
            );

            metrics::calculate_step(
                &mut results.metrics,
                estimations,
                derivatives.maximum_regularization_sum,
                config.maximum_regularization_strength,
                step,
            );
        }
        if let Some(n) = batch.as_mut() {
            *n += 1;
            if *n == config.batch_size {
                calculate_average_delays(
                    &mut estimations.average_delays,
                    &results
                        .model
                        .as_ref()
                        .unwrap()
                        .functional_description
                        .ap_params,
                );
                calculate_batch_derivatives(
                    derivatives,
                    estimations,
                    &results.model.as_ref().unwrap().functional_description,
                    config,
                );
                results
                    .model
                    .as_mut()
                    .unwrap()
                    .functional_description
                    .ap_params
                    .update(derivatives, config, num_steps, *n);
                derivatives.reset();
                *n = 0;
                metrics::calculate_batch(&mut results.metrics, *batch_index);
                *batch_index += 1;
            }
        }
    }
    if let Some(n) = batch {
        if n > 0 {
            calculate_average_delays(
                &mut estimations.average_delays,
                &results
                    .model
                    .as_ref()
                    .unwrap()
                    .functional_description
                    .ap_params,
            );
            calculate_batch_derivatives(
                derivatives,
                estimations,
                &results.model.as_ref().unwrap().functional_description,
                config,
            );
            results
                .model
                .as_mut()
                .unwrap()
                .functional_description
                .ap_params
                .update(&mut results.derivatives, config, num_steps, n);
            metrics::calculate_batch(&mut results.metrics, *batch_index);
            *batch_index += 1;
        }
    } else {
        calculate_average_delays(
            &mut estimations.average_delays,
            &results
                .model
                .as_ref()
                .unwrap()
                .functional_description
                .ap_params,
        );
        calculate_batch_derivatives(
            derivatives,
            estimations,
            &results.model.as_ref().unwrap().functional_description,
            config,
        );
        results
            .model
            .as_mut()
            .unwrap()
            .functional_description
            .ap_params
            .update(&mut results.derivatives, config, num_steps, num_beats);
        metrics::calculate_batch(&mut results.metrics, *batch_index);
        *batch_index += 1;
    }
}

#[tracing::instrument(level = "trace")]
pub fn constrain_system_states(
    system_states: &mut SystemStates,
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
