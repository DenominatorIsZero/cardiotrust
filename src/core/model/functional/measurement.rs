use ndarray::Array2;

#[derive(Debug, PartialEq)]
pub struct MeasurementMatrix {
    pub values: Array2<f32>,
}

impl MeasurementMatrix {
    pub fn empty(number_of_states: usize, number_of_sensors: usize) -> MeasurementMatrix {
        MeasurementMatrix {
            values: Array2::zeros((number_of_sensors, number_of_states)),
        }
    }
}
