use ndarray::Array2;
use serde::{Deserialize, Serialize};

/// Shape for the simulated/estimated system states
///
/// Has dimensions (`number_of_steps` `number_of_states`)
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ArraySystemStates {
    pub values: Array2<f32>,
}

impl ArraySystemStates {
    #[must_use]
    pub fn empty(number_of_steps: usize, number_of_states: usize) -> Self {
        Self {
            values: Array2::zeros((number_of_steps, number_of_states)),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ArrayMeasurements {
    pub values: Array2<f32>,
}

impl ArrayMeasurements {
    #[must_use]
    pub fn empty(number_of_steps: usize, number_of_sensors: usize) -> Self {
        Self {
            values: Array2::zeros((number_of_steps, number_of_sensors)),
        }
    }
}
