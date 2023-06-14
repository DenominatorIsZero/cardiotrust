pub mod shapes;

use ndarray::s;

use crate::core::model::functional::kalman::KalmanGain;
use crate::core::model::{
    functional::allpass::shapes::ArrayGains, functional::FunctionalDescription,
};

use crate::core::data::shapes::{ArrayMeasurements, ArraySystemStates};
#[derive(Debug, PartialEq)]
pub struct Estimations {
    pub ap_outputs: ArrayGains<f32>,
    pub system_states: ArraySystemStates,
    pub measurements: ArrayMeasurements,
    pub residuals: ArrayMeasurements,
}

impl Estimations {
    pub fn new(
        number_of_states: usize,
        number_of_sensors: usize,
        number_of_steps: usize,
    ) -> Estimations {
        Estimations {
            ap_outputs: ArrayGains::empty(number_of_states),
            system_states: ArraySystemStates::empty(number_of_steps, number_of_states),
            measurements: ArrayMeasurements::empty(number_of_steps, number_of_sensors),
            residuals: ArrayMeasurements::empty(1, number_of_sensors),
        }
    }

    pub fn reset(&mut self) {
        self.ap_outputs.values.fill(0.0);
        self.system_states.values.fill(0.0);
        self.measurements.values.fill(0.0);
        self.residuals.values.fill(0.0);
    }
}

pub fn calculate_system_prediction(
    ap_outputs: &mut ArrayGains<f32>,
    system_states: &mut ArraySystemStates,
    measurements: &mut ArrayMeasurements,
    functional_description: &FunctionalDescription,
    time_index: usize,
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
        .filter(|(_, output_state_index)| output_state_index.is_some())
        .for_each(|((gain_index, ap_output), output_state_index)| {
            let coef_index = (gain_index.0 / 3, gain_index.1, gain_index.2, gain_index.3);
            let coef = functional_description.ap_params.coefs.values[coef_index];
            let delay = functional_description.ap_params.delays.values[coef_index];
            let input = if delay <= time_index {
                system_states.values[(time_index - delay, output_state_index.unwrap())]
            } else {
                0.0
            };
            let input_delayed = if delay < time_index {
                system_states.values[(time_index - delay - 1, output_state_index.unwrap())]
            } else {
                0.0
            };
            *ap_output = coef * (input - *ap_output) + input_delayed;
            let gain = functional_description.ap_params.gains.values[gain_index];
            system_states.values[(time_index, gain_index.0)] += gain * *ap_output;
        });
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
    // Prediction of measurements H * x
    measurements.values.slice_mut(s![time_index, ..]).assign(
        &functional_description
            .measurement_matrix
            .values
            .dot(&system_states.values.slice(s![time_index, ..])),
    );
}

pub fn calculate_residuals(
    residuals: &mut ArrayMeasurements,
    predicted_measurements: &ArrayMeasurements,
    actual_measurements: &ArrayMeasurements,
    time_index: usize,
) {
    residuals.values.slice_mut(s![0, ..]).assign(
        &(&actual_measurements.values.slice(s![time_index, ..])
            - &predicted_measurements.values.slice(s![time_index, ..])),
    )
}

pub fn calculate_system_update(
    system_states: &mut ArraySystemStates,
    residuals: &ArrayMeasurements,
    kalman_gain: &KalmanGain,
    time_index: usize,
) {
    let mut states = system_states.values.slice_mut(s![time_index, ..]);
    states.assign(&(&states + kalman_gain.values.dot(&residuals.values.slice(s![0, ..]))));
}

#[cfg(test)]
mod tests {
    use ndarray::Dim;

    use super::*;
    #[test]
    fn prediction_no_crash() {
        let number_of_states = 3000;
        let number_of_sensors = 300;
        let number_of_steps = 2000;
        let time_index = 333;
        let voxels_in_dims = Dim([1000, 1, 1]);

        let mut ap_outputs = ArrayGains::empty(number_of_states);
        let mut system_states = ArraySystemStates::empty(number_of_steps, number_of_states);
        let mut measurements = ArrayMeasurements::empty(number_of_steps, number_of_sensors);
        let functional_description = FunctionalDescription::empty(
            number_of_states,
            number_of_sensors,
            number_of_steps,
            voxels_in_dims,
        );

        calculate_system_prediction(
            &mut ap_outputs,
            &mut system_states,
            &mut measurements,
            &functional_description,
            time_index,
        )
    }

    #[test]
    fn update_no_crash() {
        let number_of_states = 3000;
        let number_of_sensors = 300;
        let number_of_steps = 2000;
        let index_time = 333;

        let mut system_states = ArraySystemStates::empty(number_of_steps, number_of_states);
        let residuals = ArrayMeasurements::empty(1, number_of_sensors);
        let kalman_gain = KalmanGain::empty(number_of_states, number_of_sensors);

        calculate_system_update(&mut system_states, &residuals, &kalman_gain, index_time);
    }

    #[test]
    fn residuals_no_crash() {
        let number_of_sensors = 300;
        let number_of_steps = 2000;
        let time_index = 333;

        let mut residuals = ArrayMeasurements::empty(1, number_of_sensors);
        let predicted_measurements = ArrayMeasurements::empty(number_of_steps, number_of_sensors);
        let actual_measurements = ArrayMeasurements::empty(number_of_steps, number_of_sensors);

        calculate_residuals(
            &mut residuals,
            &predicted_measurements,
            &actual_measurements,
            time_index,
        )
    }
}
