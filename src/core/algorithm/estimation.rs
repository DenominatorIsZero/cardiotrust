use ndarray::{Array1, Array2};

use crate::core::model::shapes::ArrayGains;

use self::shapes::{ArrayMeasurement, ArraySystemStates};

pub mod shapes;

pub struct Estimations {
    pub ap_outputs: ArrayGains<f32>,
    pub system_states: ArraySystemStates,
    pub residuals: ArrayMeasurement,
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
            residuals: ArrayMeasurement::new(number_of_sensors),
        }
    }
}
