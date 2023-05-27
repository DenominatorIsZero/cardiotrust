use ndarray::Array2;

use crate::core::model::shapes::ArrayGains;

/// Shape for the estimated system states
///
/// Has dimensions (number_of_steps x number_of_states)
pub struct ArraySystemStates {
    pub values: Array2<f32>,
}

impl ArraySystemStates {
    pub fn new(number_of_steps: usize, number_of_states: usize) -> ArraySystemStates {
        ArraySystemStates {
            values: Array2::zeros((number_of_steps, number_of_states)),
        }
    }
}

pub struct Estimations {
    pub ap_outputs: ArrayGains<f32>,
    pub system_states: ArraySystemStates,
}
