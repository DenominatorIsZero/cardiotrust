use ndarray::s;
use tracing::trace;

use crate::core::{
    data::shapes::{ArraySystemStates, Measurements},
    model::functional::{
        allpass::{shapes::ArrayGains, APParameters},
        measurement::MeasurementMatrix,
        FunctionalDescription,
    },
};

/// Calculates the system prediction by innovating the system states,
/// adding the control function, and predicting measurements.
///
/// # Panics
///
/// Panics if `ap_params_flat` is not set.
#[allow(clippy::module_name_repetitions)]
#[tracing::instrument(level = "trace")]
pub fn calculate_system_prediction(
    ap_outputs: &mut ArrayGains<f32>,
    system_states: &mut ArraySystemStates,
    measurements: &mut Measurements,
    functional_description: &FunctionalDescription,
    time_index: usize,
    beat_index: usize,
) {
    trace!("Calculating system prediction");
    innovate_system_states_v1(
        ap_outputs,
        &functional_description.ap_params,
        time_index,
        system_states,
    );
    add_control_function(functional_description, time_index, system_states);
    predict_measurements(
        measurements,
        time_index,
        beat_index,
        &functional_description.measurement_matrix,
        system_states,
    );
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
    ap_outputs: &mut ArrayGains<f32>,
    ap_params: &APParameters,
    time_index: usize,
    system_states: &mut ArraySystemStates,
) {
    trace!("Innovating system states");
    // Calculate ap outputs and system states
    let output_state_indices = &ap_params.output_state_indices.values;
    for index_state in 0..ap_outputs.values.shape()[0] {
        for index_offset in 0..ap_outputs.values.shape()[1] {
            let output_state_index =
                unsafe { output_state_indices.uget((index_state, index_offset)) };
            if output_state_index.is_none() {
                continue;
            }
            let output_state_index = output_state_index.expect("Output state index to be some");
            let coef_index = (index_state / 3, index_offset / 3);
            let coef = unsafe { *ap_params.coefs.values.uget(coef_index) };
            let delay = unsafe { *ap_params.delays.values.uget(coef_index) };
            let input = if delay <= time_index {
                unsafe { *system_states.uget((time_index - delay, output_state_index)) }
            } else {
                0.0
            };
            let input_delayed = if delay < time_index {
                *unsafe { system_states.uget((time_index - delay - 1, output_state_index)) }
            } else {
                0.0
            };
            let ap_output = unsafe { ap_outputs.values.uget_mut((index_state, index_offset)) };
            *ap_output = coef.mul_add(input - *ap_output, input_delayed);
            let gain = unsafe { *ap_params.gains.values.uget((index_state, index_offset)) };
            unsafe {
                *system_states.uget_mut((time_index, index_state)) +=
                    gain * ap_outputs.values.uget((index_state, index_offset));
            };
        }
    }
}

/// Adds a control function value multiplied by the control matrix to the
/// system states for the given time index. This allows an external control
/// signal to be injected into the system states.
#[inline]
#[tracing::instrument(level = "trace")]
pub fn add_control_function(
    functional_description: &FunctionalDescription,
    time_index: usize,
    system_states: &mut ArraySystemStates,
) {
    trace!("Adding control function");
    // Add control function
    let control_function_value = functional_description.control_function_values.values[time_index];
    system_states
        .slice_mut(s![time_index, ..])
        .iter_mut()
        .zip(functional_description.control_matrix.values.iter())
        .for_each(|(system_state, coef)| {
            *system_state += coef * control_function_value;
        });
}

/// Predicts the measurements by multiplying the measurement matrix with the
/// system states for the given time index. This computes the model predicted
/// measurements to compare against the actual measurements.
#[inline]
#[tracing::instrument(level = "trace")]
pub fn predict_measurements(
    measurements: &mut Measurements,
    time_index: usize,
    beat_index: usize,
    measurement_matrix: &MeasurementMatrix,
    system_states: &ArraySystemStates,
) {
    trace!("Predicting measurements");
    // Prediction of measurements H * x
    let measurement_matrix = measurement_matrix.values.slice(s![beat_index, .., ..]);
    measurements
        .slice_mut(s![beat_index, time_index, ..])
        .assign(&measurement_matrix.dot(&system_states.slice(s![time_index, ..])));
}
