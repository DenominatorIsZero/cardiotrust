use ndarray::Array2;

use crate::core::{config::model::Model, model::spatial::SpatialDescription};

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

    pub fn from_model_config(
        _config: &Model,
        _spatial_description: &SpatialDescription,
    ) -> MeasurementMatrix {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_model_config_no_crash() {
        let config = Model::default();
        let spatial_description = SpatialDescription::from_model_config(&config);

        let _measurement_matrix =
            MeasurementMatrix::from_model_config(&config, &spatial_description);
    }
}
