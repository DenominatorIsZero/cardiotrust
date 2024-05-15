use ndarray::s;
use tracing::trace;

use crate::core::{
    data::shapes::{Measurements, SystemStates},
    model::functional::{
        allpass::{shapes::Gains, APParameters},
        control::{ControlFunction, ControlMatrix},
        measurement::MeasurementMatrixAtBeat,
    },
};

/// Calculates the system prediction by innovating the system states,
/// adding the control function, and predicting measurements.
///
/// # Panics
///
/// Panics if `ap_params_flat` is not set.
#[allow(clippy::module_name_repetitions)]
#[tracing::instrument(level = "trace", skip_all)]
pub fn calculate_system_prediction(
    ap_outputs: &mut Gains,
    system_states: &mut SystemStates,
    measurements: &mut Measurements,
    ap_params: &APParameters,
    measurement_matrix: &MeasurementMatrixAtBeat,
    control_function: &ControlFunction,
    control_matrix: &ControlMatrix,
    time_index: usize,
    beat_index: usize,
) {
    trace!("Calculating system prediction");
    innovate_system_states_v1(ap_outputs, ap_params, time_index, system_states);
    add_control_function(system_states, control_function, control_matrix, time_index);
    predict_measurements(
        measurements,
        time_index,
        beat_index,
        measurement_matrix,
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
    ap_outputs: &mut Gains,
    ap_params: &APParameters,
    time_index: usize,
    system_states: &mut SystemStates,
) {
    trace!("Innovating system states");
    // Calculate ap outputs and system states
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
            let ap_output = unsafe { ap_outputs.uget_mut((index_state, index_offset)) };
            *ap_output = coef.mul_add(input - *ap_output, input_delayed);
            let gain = unsafe { *ap_params.gains.uget((index_state, index_offset)) };
            unsafe {
                *system_states.uget_mut((time_index, index_state)) +=
                    gain * ap_outputs.uget((index_state, index_offset));
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
    system_states: &mut SystemStates,
    control_function: &ControlFunction,
    control_matrix: &ControlMatrix,
    time_index: usize,
) {
    trace!("Adding control function");
    // Add control function
    let control_function_value = control_function[time_index];
    system_states
        .slice_mut(s![time_index, ..])
        .iter_mut()
        .zip(control_matrix.iter())
        .for_each(|(system_state, coef)| {
            *system_state += coef * control_function_value;
        });
}

/// Predicts the measurements by multiplying the measurement matrix with the
/// system states for the given time index. This computes the model predicted
/// measurements to compare against the actual measurements.
#[inline]
#[tracing::instrument(level = "trace", skip_all)]
pub fn predict_measurements(
    measurements: &mut Measurements,
    time_index: usize,
    beat_index: usize,
    measurement_matrix: &MeasurementMatrixAtBeat,
    system_states: &SystemStates,
) {
    trace!("Predicting measurements");
    // Prediction of measurements H * x
    measurements
        .slice_mut(s![beat_index, time_index, ..])
        .assign(&measurement_matrix.dot(&system_states.slice(s![time_index, ..])));
}
