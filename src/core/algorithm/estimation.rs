pub mod shapes;

use ndarray::{s, Array2};
use serde::{Deserialize, Serialize};

use crate::core::config::algorithm::Algorithm;
use crate::core::model::functional::allpass::from_coef_to_samples;
use crate::core::model::functional::allpass::shapes::ArrayDelays;
use crate::core::model::functional::kalman::Gain;
use crate::core::model::{
    functional::allpass::shapes::ArrayGains, functional::FunctionalDescription,
};

use crate::core::data::shapes::{ArrayMeasurements, ArraySystemStates};
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Estimations {
    pub ap_outputs: ArrayGains<f32>,
    pub system_states: ArraySystemStates,
    pub state_covariance: ArrayGains<f32>,
    pub measurements: ArrayMeasurements,
    pub residuals: ArrayMeasurements,
    pub system_states_delta: ArraySystemStates,
    pub gains_delta: ArrayGains<f32>,
    pub delays_delta: ArrayDelays<f32>,
    pub s: Array2<f32>,
    pub s_inv: Array2<f32>,
}

impl Estimations {
    #[must_use]
    pub fn empty(
        number_of_states: usize,
        number_of_sensors: usize,
        number_of_steps: usize,
    ) -> Self {
        Self {
            ap_outputs: ArrayGains::empty(number_of_states),
            system_states: ArraySystemStates::empty(number_of_steps, number_of_states),
            state_covariance: ArrayGains::empty(number_of_states),
            measurements: ArrayMeasurements::empty(number_of_steps, number_of_sensors),
            residuals: ArrayMeasurements::empty(1, number_of_sensors),
            system_states_delta: ArraySystemStates::empty(1, number_of_states),
            gains_delta: ArrayGains::empty(number_of_states),
            delays_delta: ArrayDelays::empty(number_of_states),
            s: Array2::zeros([number_of_sensors, number_of_sensors]),
            s_inv: Array2::zeros([number_of_sensors, number_of_sensors]),
        }
    }

    pub fn reset(&mut self) {
        self.ap_outputs.values.fill(0.0);
        self.system_states.values.fill(0.0);
        self.state_covariance.values.fill(0.0);
        self.measurements.values.fill(0.0);
        self.residuals.values.fill(0.0);
        self.system_states_delta.values.fill(0.0);
        self.gains_delta.values.fill(0.0);
        self.delays_delta.values.fill(0.0);
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
        &(&predicted_measurements.values.slice(s![time_index, ..])
            - &actual_measurements.values.slice(s![time_index, ..])),
    );
}

pub fn calculate_system_states_delta(
    system_states_delta: &mut ArraySystemStates,
    estimated_system_states: &ArraySystemStates,
    actual_system_states: &ArraySystemStates,
    time_index: usize,
) {
    system_states_delta.values.slice_mut(s![0, ..]).assign(
        &(&estimated_system_states.values.slice(s![time_index, ..])
            - &actual_system_states.values.slice(s![time_index, ..])),
    );
}

pub fn calculate_gains_delta(
    gains_delta: &mut ArrayGains<f32>,
    estimated_gains: &ArrayGains<f32>,
    actual_gains: &ArrayGains<f32>,
) {
    gains_delta
        .values
        .assign(&(&estimated_gains.values - &actual_gains.values));
}

pub fn calculate_delays_delta(
    delays_delta: &mut ArrayDelays<f32>,
    estimated_delays: &ArrayDelays<usize>,
    actual_delays: &ArrayDelays<usize>,
    estimated_coefs: &ArrayDelays<f32>,
    actual_coefs: &ArrayDelays<f32>,
) {
    #[allow(clippy::cast_precision_loss)]
    delays_delta
        .values
        .indexed_iter_mut()
        .for_each(|(index, delay_delta)| {
            *delay_delta = (estimated_delays.values[index] as f32
                - actual_delays.values[index] as f32)
                + (from_coef_to_samples(estimated_coefs.values[index])
                    - from_coef_to_samples(actual_coefs.values[index]));
        });
}

pub fn calculate_system_update(
    estimations: &mut Estimations,
    time_index: usize,
    functional_description: &FunctionalDescription,
    config: &Algorithm,
) {
    if config.calculate_kalman_gain {
        calculate_kalman_gain(estimations, functional_description);
    }
    let mut states = estimations
        .system_states
        .values
        .slice_mut(s![time_index, ..]);
    states.assign(
        &(&states
            + functional_description
                .kalman_gain
                .values
                .dot(&estimations.residuals.values.slice(s![0, ..]))),
    );
}

fn calculate_kalman_gain(
    estimations: &mut Estimations,
    functional_description: &FunctionalDescription,
) {
    todo!()
}

#[cfg(test)]
mod tests {
    use ndarray::{arr1, Dim};

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
        );
    }

    #[test]
    fn update_no_crash() {
        let number_of_states = 3000;
        let number_of_sensors = 300;
        let number_of_steps = 2000;
        let time_index = 333;
        let config = Algorithm::default();

        let mut estimations =
            Estimations::empty(number_of_states, number_of_sensors, number_of_steps);
        let mut functional_desrciption = FunctionalDescription::empty(
            number_of_states,
            number_of_sensors,
            number_of_steps,
            Dim([number_of_states / 3, 1, 1]),
        );

        calculate_system_update(
            &mut estimations,
            time_index,
            &functional_desrciption,
            &config,
        );
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
        );
    }
}
