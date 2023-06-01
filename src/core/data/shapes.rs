use ndarray::Array2;

/// Shape for the simulated/estimated system states
///
/// Has dimensions (number_of_steps x number_of_states)
#[derive(Debug, PartialEq)]
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

#[derive(Debug, PartialEq)]
pub struct ArrayMeasurements {
    pub values: Array2<f32>,
}

impl ArrayMeasurements {
    pub fn new(number_of_steps: usize, number_of_sensors: usize) -> ArrayMeasurements {
        ArrayMeasurements {
            values: Array2::zeros((number_of_steps, number_of_sensors)),
        }
    }
}
