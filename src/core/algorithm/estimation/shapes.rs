use ndarray::{Array1, Array2};

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

pub struct ArrayMeasurement {
    pub values: Array1<f32>,
}

impl ArrayMeasurement {
    pub fn new(number_of_sensors: usize) -> ArrayMeasurement {
        ArrayMeasurement {
            values: Array1::zeros(number_of_sensors),
        }
    }
}
