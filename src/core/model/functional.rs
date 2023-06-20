pub mod allpass;
pub mod control;
pub mod kalman;
pub mod measurement;

use std::error::Error;

use ndarray::Dim;

use self::{
    allpass::APParameters,
    control::{ControlFunction, ControlMatrix},
    kalman::KalmanGain,
    measurement::MeasurementMatrix,
};

use super::spatial::SpatialDescription;
use crate::core::config::model::Model;

#[derive(Debug, PartialEq)]
pub struct FunctionalDescription {
    pub ap_params: APParameters,
    pub measurement_matrix: MeasurementMatrix,
    pub control_matrix: ControlMatrix,
    pub kalman_gain: KalmanGain,
    pub control_function_values: ControlFunction,
}

impl FunctionalDescription {
    pub fn empty(
        number_of_states: usize,
        number_of_sensors: usize,
        number_of_steps: usize,
        voxels_in_dims: Dim<[usize; 3]>,
    ) -> FunctionalDescription {
        FunctionalDescription {
            ap_params: APParameters::empty(number_of_states, voxels_in_dims),
            measurement_matrix: MeasurementMatrix::empty(number_of_states, number_of_sensors),
            control_matrix: ControlMatrix::empty(number_of_states),
            kalman_gain: KalmanGain::empty(number_of_states, number_of_sensors),
            control_function_values: ControlFunction::empty(number_of_steps),
        }
    }
    pub fn from_model_config(
        config: &Model,
        spatial_description: &SpatialDescription,
        sample_rate_hz: f32,
        duration_s: f32,
    ) -> Result<FunctionalDescription, Box<dyn Error>> {
        let ap_params =
            APParameters::from_model_config(config, spatial_description, sample_rate_hz)?;
        let measurement_matrix = MeasurementMatrix::from_model_config(config, spatial_description);
        let control_matrix = ControlMatrix::from_model_config(config, spatial_description);
        let kalman_gain = KalmanGain::from_model_config(config, &measurement_matrix);
        let control_function_values =
            ControlFunction::from_model_config(config, sample_rate_hz, duration_s);

        Ok(FunctionalDescription {
            ap_params,
            measurement_matrix,
            control_matrix,
            kalman_gain,
            control_function_values,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::core::config::model::Model;

    use super::*;

    #[test]
    fn ap_empty() {
        let number_of_states = 3000;
        let voxels_in_dims = Dim([1000, 1, 1]);

        let _ap_params = APParameters::empty(number_of_states, voxels_in_dims);
    }

    #[test]
    fn funcional_empty() {
        let number_of_states = 3000;
        let number_of_sensors = 300;
        let number_of_steps = 2000;
        let voxels_in_dims = Dim([1000, 1, 1]);

        let _functional_description = FunctionalDescription::empty(
            number_of_states,
            number_of_sensors,
            number_of_steps,
            voxels_in_dims,
        );
    }

    #[test]
    fn from_model_config_no_crash() {
        let config = Model::default();
        let spatial_description = SpatialDescription::from_model_config(&config);
        let sample_rate_hz = 2000.0;
        let duration_s = 2.0;
        let _functional_description = FunctionalDescription::from_model_config(
            &config,
            &spatial_description,
            sample_rate_hz,
            duration_s,
        )
        .unwrap();
    }
}
