pub mod allpass;
pub mod control;
pub mod kalman;
pub mod measurement;

use std::error::Error;

use approx::relative_eq;
use ndarray::Dim;

use rand_distr::{Distribution, Normal};
use serde::{Deserialize, Serialize};

use self::{
    allpass::{shapes::normal::ArrayGains, APParameters},
    control::{ControlFunction, ControlMatrix},
    kalman::Gain,
    measurement::{MeasurementCovariance, MeasurementMatrix},
};

use super::spatial::SpatialDescription;
use crate::core::config::model::Model;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[allow(clippy::module_name_repetitions)]
pub struct FunctionalDescription {
    pub ap_params: APParameters,
    pub measurement_matrix: MeasurementMatrix,
    pub control_matrix: ControlMatrix,
    pub process_covariance: ArrayGains<f32>,
    pub measurement_covariance: MeasurementCovariance,
    pub kalman_gain: Gain,
    pub control_function_values: ControlFunction,
}

impl FunctionalDescription {
    #[must_use]
    pub fn empty(
        number_of_states: usize,
        number_of_sensors: usize,
        number_of_steps: usize,
        voxels_in_dims: Dim<[usize; 3]>,
    ) -> Self {
        Self {
            ap_params: APParameters::empty(number_of_states, voxels_in_dims),
            measurement_matrix: MeasurementMatrix::empty(number_of_states, number_of_sensors),
            control_matrix: ControlMatrix::empty(number_of_states),
            process_covariance: ArrayGains::empty(number_of_states),
            measurement_covariance: MeasurementCovariance::empty(number_of_sensors),
            kalman_gain: Gain::empty(number_of_states, number_of_sensors),
            control_function_values: ControlFunction::empty(number_of_steps),
        }
    }
    /// .
    ///
    /// # Errors
    ///
    /// This function will return an error if the config does not
    /// result in valid delay values.
    pub fn from_model_config(
        config: &Model,
        spatial_description: &SpatialDescription,
        sample_rate_hz: f32,
        duration_s: f32,
    ) -> Result<Self, Box<dyn Error>> {
        let ap_params =
            APParameters::from_model_config(config, spatial_description, sample_rate_hz)?;
        let measurement_matrix = MeasurementMatrix::from_model_config(config, spatial_description);
        let control_matrix = ControlMatrix::from_model_config(config, spatial_description);
        let process_covariance =
            process_covariance_from_model_config(config, spatial_description, &ap_params);
        let measurement_covariance =
            MeasurementCovariance::from_model_config(config, spatial_description);
        let kalman_gain = Gain::from_model_config(config, &measurement_matrix);
        let control_function_values =
            ControlFunction::from_model_config(config, sample_rate_hz, duration_s);

        Ok(Self {
            ap_params,
            measurement_matrix,
            control_matrix,
            process_covariance,
            measurement_covariance,
            kalman_gain,
            control_function_values,
        })
    }

    pub fn save_npy(&self, path: &std::path::Path) {
        let path = &path.join("functional_description");
        self.ap_params.save_npy(path);
        self.measurement_matrix.save_npy(path);
        self.control_matrix.save_npy(path);
        self.process_covariance
            .save_npy(path, "process_covariance.npy");
        self.measurement_covariance.save_npy(path);
        self.kalman_gain.save_npy(path);
        self.control_function_values.save_npy(path);
    }
}

fn process_covariance_from_model_config(
    config: &Model,
    spatial_description: &SpatialDescription,
    ap_params: &APParameters,
) -> ArrayGains<f32> {
    let normal = if relative_eq!(config.process_covariance_std, 0.0) {
        None
    } else {
        Some(
            Normal::<f32>::new(
                config.process_covariance_mean,
                config.process_covariance_std,
            )
            .unwrap(),
        )
    };
    let mut process_covariance = ArrayGains::empty(spatial_description.voxels.count_states());
    process_covariance
        .values
        .indexed_iter_mut()
        .zip(ap_params.output_state_indices.values.iter())
        .filter(|((index, _), output_state_index)| {
            output_state_index.is_some() && index.0 == output_state_index.unwrap_or(0)
        })
        .for_each(|((_, variance), _)| {
            *variance = normal.map_or(config.process_covariance_mean, |dist| {
                dist.sample(&mut rand::thread_rng())
            });
        });
    process_covariance
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
