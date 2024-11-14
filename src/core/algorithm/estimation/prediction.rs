use tracing::trace;

use super::Estimations;
use crate::core::model::functional::FunctionalDescription;

/// Calculates the system prediction by innovating the system states,
/// adding the control function, and predicting measurements.
///
/// # Panics
///
/// Panics if `ap_params_flat` is not set.
#[allow(clippy::module_name_repetitions)]
#[tracing::instrument(level = "trace", skip_all)]
pub fn calculate_system_prediction(
    estimations: &mut Estimations,
    functional_description: &FunctionalDescription,
    beat: usize,
    step: usize,
) {
    trace!("Calculating system prediction");
    innovate_system_states_v1(estimations, functional_description, step);
    add_control_function(estimations, functional_description, step);
    predict_measurements(estimations, functional_description, beat, step);
}

/// Innovates the system states by calculating the all-pass filter outputs,
/// multiplying by the gains, and adding to the appropriate system states.
///
/// It calculates the all-pass outputs based on previous states and coefficients.
/// The outputs are multiplied by the gains and added to the system states.
/// Uses unsafe indexing to avoid bounds checks.
///
/// # Panics
///
/// Panics if output state indices are not initialized corrrectly.
#[inline]
#[tracing::instrument(level = "trace")]
pub fn innovate_system_states_v1(
    estimations: &mut Estimations,
    functional_description: &FunctionalDescription,
    step: usize,
) {
    trace!("Innovating system states");
    // Calculate ap outputs and system states
    let ap_params = &functional_description.ap_params;
    let ap_outputs = &mut estimations.ap_outputs;
    let system_states = &mut estimations.system_states;

    let output_state_indices = &ap_params.output_state_indices;
    for index_state in 0..ap_outputs.shape()[0] {
        for index_offset in 0..ap_outputs.shape()[1] {
            let output_state_index =
                unsafe { output_state_indices.uget((index_state, index_offset)) };
            if output_state_index.is_none() {
                continue;
            }
            let output_state_index = output_state_index.expect("Output state index to be some");
            let coef_index = (index_state / 3, index_offset / 3);
            let coef = unsafe { *ap_params.coefs.uget(coef_index) };
            let delay = unsafe { *ap_params.delays.uget(coef_index) };
            let input = if delay <= step {
                unsafe { *system_states.uget((step - delay, output_state_index)) }
            } else {
                0.0
            };
            let input_delayed = if delay < step {
                *unsafe { system_states.uget((step - delay - 1, output_state_index)) }
            } else {
                0.0
            };
            let ap_output = unsafe { ap_outputs.uget_mut((index_state, index_offset)) };
            *ap_output = coef.mul_add(input - *ap_output, input_delayed);
            let gain = unsafe { *ap_params.gains.uget((index_state, index_offset)) };
            unsafe {
                *system_states.uget_mut((step, index_state)) += gain * *ap_output;
            };
        }
    }
}

/// Adds a control function value multiplied by the control matrix to the
/// system states for the given time index. This allows an external control
/// signal to be injected into the system states.
#[inline]
#[tracing::instrument(level = "trace", skip_all)]
pub fn add_control_function(
    estimations: &mut Estimations,
    functional_description: &FunctionalDescription,
    step: usize,
) {
    trace!("Adding control function");
    // Add control function
    estimations.system_states.at_step_mut(step).scaled_add(
        functional_description.control_function_values[step],
        &*functional_description.control_matrix,
    );
}

/// Predicts the measurements by multiplying the measurement matrix with the
/// system states for the given time index. This computes the model predicted
/// measurements to compare against the actual measurements.
#[inline]
#[tracing::instrument(level = "trace", skip_all)]
pub fn predict_measurements(
    estimations: &mut Estimations,
    functional_description: &FunctionalDescription,
    beat: usize,
    step: usize,
) {
    trace!("Predicting measurements");
    // Prediction of measurements H * x
    estimations
        .measurements
        .at_beat_mut(beat)
        .at_step_mut(step)
        .assign(
            &functional_description
                .measurement_matrix
                .at_beat(beat)
                .dot(&*estimations.system_states.at_step(step)),
        );
}
