use ndarray::s;

use crate::core::{
    data::shapes::{ArrayMeasurements, ArraySystemStates},
    model::functional::{
        allpass::{shapes::ArrayGains, APParameters},
        measurement::MeasurementMatrix,
        FunctionalDescription,
    },
};

/// .
///
/// # Panics
///
/// Panics if `ap_params_flat` is not set.
#[allow(clippy::module_name_repetitions)]
pub fn calculate_system_prediction(
    ap_outputs: &mut ArrayGains<f32>,
    system_states: &mut ArraySystemStates,
    measurements: &mut ArrayMeasurements,
    functional_description: &FunctionalDescription,
    time_index: usize,
) {
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
        &functional_description.measurement_matrix,
        system_states,
    );
}

/// Uses unsafe get operations.
///
/// # Panics
///
/// Panics if output state indices are not initialized corrrectly.
#[inline]
pub fn innovate_system_states_v1(
    ap_outputs: &mut ArrayGains<f32>,
    ap_params: &APParameters,
    time_index: usize,
    system_states: &mut ArraySystemStates,
) {
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
                unsafe {
                    *system_states
                        .values
                        .uget((time_index - delay, output_state_index))
                }
            } else {
                0.0
            };
            let input_delayed = if delay < time_index {
                *unsafe {
                    system_states
                        .values
                        .uget((time_index - delay - 1, output_state_index))
                }
            } else {
                0.0
            };
            let ap_output = unsafe { ap_outputs.values.uget_mut((index_state, index_offset)) };
            *ap_output = coef.mul_add(input - *ap_output, input_delayed);
            let gain = unsafe { *ap_params.gains.values.uget((index_state, index_offset)) };
            unsafe {
                *system_states.values.uget_mut((time_index, index_state)) +=
                    gain * ap_outputs.values.uget((index_state, index_offset));
            };
        }
    }
}

#[inline]
pub fn add_control_function(
    functional_description: &FunctionalDescription,
    time_index: usize,
    system_states: &mut ArraySystemStates,
) {
    // Add control function
    let control_function_value = functional_description.control_function_values.values[time_index];
    system_states
        .values
        .slice_mut(s![time_index, ..])
        .iter_mut()
        .zip(functional_description.control_matrix.values.iter())
        .for_each(|(system_state, coef)| {
            *system_state += coef * control_function_value;
        });
}

#[inline]
pub fn predict_measurements(
    measurements: &mut ArrayMeasurements,
    time_index: usize,
    measurement_matrix: &MeasurementMatrix,
    system_states: &ArraySystemStates,
) {
    // Prediction of measurements H * x
    measurements.values.slice_mut(s![time_index, ..]).assign(
        &measurement_matrix
            .values
            .dot(&system_states.values.slice(s![time_index, ..])),
    );
}
