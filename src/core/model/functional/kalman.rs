use ndarray::Array2;

use crate::core::config::model::Model;

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

    pub fn from_model_config(
        _config: &Model,
        _measurement_matrix: &MeasurementMatrix,
    ) -> KalmanGain {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use crate::core::model::spatial::SpatialDescription;

    use super::*;

    #[test]
    fn from_model_config_no_crash() {
        let config = Model::default();
        let spatial_description = SpatialDescription::from_model_config(&config);
        let measurement_matrix =
            MeasurementMatrix::from_model_config(&config, &spatial_description);

        let _kalman_gain = KalmanGain::from_model_config(&config, &measurement_matrix);
    }
}
