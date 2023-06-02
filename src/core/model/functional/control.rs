use ndarray::Array1;

#[derive(Debug, PartialEq)]
pub struct ControlMatrix {
    pub values: Array1<f32>,
}

impl ControlMatrix {
    pub fn empty(number_of_states: usize) -> ControlMatrix {
        ControlMatrix {
            values: Array1::zeros(number_of_states),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct ControlFunction {
    pub values: Array1<f32>,
}

impl ControlFunction {
    pub fn empty(number_of_steps: usize) -> ControlFunction {
        ControlFunction {
            values: Array1::zeros(number_of_steps),
        }
    }
}
