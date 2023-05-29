use ndarray::s;

use crate::core::model::{
    shapes::{ArrayGains, ArrayKalmanGain},
    FunctionalDescription,
};

use self::shapes::{ArrayMeasurements, ArraySystemStates};

pub mod shapes;

pub struct Estimations {
    pub ap_outputs: ArrayGains<f32>,
    pub system_states: ArraySystemStates,
    pub residuals: ArrayMeasurements,
}

impl Estimations {
    pub fn new(
        number_of_states: usize,
        number_of_sensors: usize,
        number_of_steps: usize,
    ) -> Estimations {
        Estimations {
            ap_outputs: ArrayGains::new(number_of_states),
            system_states: ArraySystemStates::new(number_of_steps, number_of_states),
            residuals: ArrayMeasurements::new(1, number_of_sensors),
        }
    }
}

pub fn calculate_system_prediction(
    ap_outputs: &mut ArrayGains<f32>,
    system_states: &mut ArraySystemStates,
    measurements: &mut ArrayMeasurements,
    functional_description: &FunctionalDescription,
    control_function_value: f32,
    index_time: usize,
) {
    measurements.values.slice_mut(s![index_time, ..]).assign(
        &functional_description
            .measurement_matrix
            .values
            .dot(&system_states.values.slice(s![index_time, ..])),
    );
}

pub fn calculate_system_update(
    system_states: &mut ArraySystemStates,
    predicted_measurements: &mut ArrayMeasurements,
    actual_measurements: &ArrayMeasurements,
    kalman_gain: &ArrayKalmanGain,
    index_time: usize,
) {
    todo!();
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn prediction_no_crash() {
        let number_of_states = 3000;
        let number_of_sensors = 300;
        let number_of_steps = 2000;
        let control_function_value = 5.0;
        let index_time = 333;

        let mut ap_outputs = ArrayGains::new(number_of_states);
        let mut system_states = ArraySystemStates::new(number_of_steps, number_of_states);
        let mut measurements = ArrayMeasurements::new(number_of_steps, number_of_sensors);
        let functional_description =
            FunctionalDescription::new(number_of_states, number_of_sensors);

        calculate_system_prediction(
            &mut ap_outputs,
            &mut system_states,
            &mut measurements,
            &functional_description,
            control_function_value,
            index_time,
        )
    }

    #[test]
    fn update_no_crash() {
        let number_of_states = 3000;
        let number_of_sensors = 300;
        let number_of_steps = 2000;
        let index_time = 333;

        let mut system_states = ArraySystemStates::new(number_of_steps, number_of_states);
        let mut predicted_measurements = ArrayMeasurements::new(number_of_steps, number_of_sensors);
        let actual_measurements = ArrayMeasurements::new(number_of_steps, number_of_sensors);
        let kalman_gain = ArrayKalmanGain::new(number_of_states, number_of_sensors);

        calculate_system_update(
            &mut system_states,
            &mut predicted_measurements,
            &actual_measurements,
            &kalman_gain,
            index_time,
        );
    }
}
