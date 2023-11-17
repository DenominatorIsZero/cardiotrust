use ndarray::s;

use crate::core::model::functional::measurement::MeasurementMatrix;
use crate::core::model::{
    functional::allpass::shapes::ArrayGains, functional::FunctionalDescription,
};

use crate::core::data::shapes::{ArrayMeasurements, ArraySystemStates};

#[inline]
pub fn calculate_system_prediction(
    ap_outputs: &mut ArrayGains<f32>,
    system_states: &mut ArraySystemStates,
    measurements: &mut ArrayMeasurements,
    functional_description: &FunctionalDescription,
    time_index: usize,
) {
    innovate_system_states(
        ap_outputs,
        functional_description,
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

#[inline]
pub fn innovate_system_states(
    ap_outputs: &mut ArrayGains<f32>,
    functional_description: &FunctionalDescription,
    time_index: usize,
    system_states: &mut ArraySystemStates,
) {
    // Calculate ap outputs and system states
    ap_outputs
        .values
        .indexed_iter_mut()
        .zip(
            functional_description
                .ap_params
                .output_state_indices
                .values
                .iter(),
        )
        .filter(|((gain_index, _), output_state_index)| {
            output_state_index.is_some()
                && !(gain_index.1 == 1 && gain_index.2 == 1 && gain_index.3 == 1)
        })
        .for_each(|((gain_index, ap_output), output_state_index)| {
            let coef_index = (gain_index.0 / 3, gain_index.1, gain_index.2, gain_index.3);
            let coef = functional_description.ap_params.coefs.values[coef_index];
            let delay = functional_description.ap_params.delays.values[coef_index];
            let input = if delay <= time_index {
                system_states.values[(time_index - delay, output_state_index.unwrap_or_default())]
            } else {
                0.0
            };
            let input_delayed = if delay < time_index {
                system_states.values[(
                    time_index - delay - 1,
                    output_state_index.unwrap_or_default(),
                )]
            } else {
                0.0
            };
            *ap_output = coef.mul_add(input - *ap_output, input_delayed);
            let gain = functional_description.ap_params.gains.values[gain_index];
            system_states.values[(time_index, gain_index.0)] += gain * *ap_output;
        });
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
    system_states: &mut ArraySystemStates,
) {
    // Prediction of measurements H * x
    measurements.values.slice_mut(s![time_index, ..]).assign(
        &measurement_matrix
            .values
            .dot(&system_states.values.slice(s![time_index, ..])),
    );
}
