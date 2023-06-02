use ndarray::Array2;

use crate::core::config::{model::Model, simulation::Simulation};

use super::measurement::MeasurementMatrix;

#[derive(Debug, PartialEq)]
pub struct KalmanGain {
    pub values: Array2<f32>,
}

impl KalmanGain {
    pub fn empty(number_of_states: usize, number_of_sensors: usize) -> KalmanGain {
        KalmanGain {
            values: Array2::zeros((number_of_states, number_of_sensors)),
        }
    }

    pub fn from_model_config(config: &Model, measurement_matrix: &MeasurementMatrix) -> KalmanGain {
        todo!()
    }
}
