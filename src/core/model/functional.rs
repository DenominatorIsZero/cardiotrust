pub mod allpass;
pub mod control;
pub mod kalman;
pub mod measurement;

use std::error::Error;

use approx::relative_eq;
use ndarray::Dim;
use ocl::{Buffer, Queue};
use rand_distr::{Distribution, Normal};
use serde::{Deserialize, Serialize};
use tracing::{debug, trace};

use self::{
    allpass::{shapes::Gains, APParameters, APParametersGPU},
    control::{ControlFunction, ControlMatrix},
    kalman::KalmanGain,
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
    pub process_covariance: Gains,
    pub measurement_covariance: MeasurementCovariance,
    pub kalman_gain: KalmanGain,
    pub control_function_values: ControlFunction,
}

pub struct FunctionalDescriptionGPU {
    pub ap_params: APParametersGPU,
    pub measurement_matrix: Buffer<f32>,
    pub control_matrix: Buffer<f32>,
    pub process_covariance: Buffer<f32>,
    pub measurement_covariance: Buffer<f32>,
    pub kalman_gain: Buffer<f32>,
    pub control_function_values: Buffer<f32>,
}

impl FunctionalDescription {
    /// Creates an empty `FunctionalDescription` with the given dimensions.
    ///
    /// This initializes all internal state to empty arrays or matrices of the appropriate size.
    /// It can be used to create a blank `FunctionalDescription` before populating its fields.
    #[must_use]
    #[tracing::instrument(level = "debug")]
    pub fn empty(
        number_of_states: usize,
        number_of_sensors: usize,
        number_of_steps: usize,
        number_of_motion_steps: usize,
        voxels_in_dims: Dim<[usize; 3]>,
    ) -> Self {
        debug!("Creating empty functional description");
        Self {
            ap_params: APParameters::empty(number_of_states, voxels_in_dims),
            measurement_matrix: MeasurementMatrix::empty(
                number_of_motion_steps,
                number_of_states,
                number_of_sensors,
            ),
            control_matrix: ControlMatrix::empty(number_of_states),
            process_covariance: Gains::empty(number_of_states),
            measurement_covariance: MeasurementCovariance::empty(number_of_sensors),
            kalman_gain: KalmanGain::empty(number_of_states, number_of_sensors),
            control_function_values: ControlFunction::empty(number_of_steps),
        }
    }
    /// Constructs a `FunctionalDescription` from the given Model config, `SpatialDescription`,
    /// sample rate, and duration. This initializes the internal state like allpass filters,
    /// matrices, gains etc. based on the provided inputs.
    ///
    /// # Errors
    ///
    /// This function will return an error if the config does not
    /// result in valid delay values.
    ///
    /// # Panics
    /// If delay cant be configured with samplerate, voxelsize and propagation speed
    #[allow(clippy::useless_let_if_seq)]
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn from_model_config(
        config: &Model,
        spatial_description: &SpatialDescription,
        sample_rate_hz: f32,
        duration_s: f32,
    ) -> Result<Self, Box<dyn Error>> {
        debug!("Creating functional description from model config");
        let ap_params =
            APParameters::from_model_config(config, spatial_description, sample_rate_hz)?;
        let process_covariance =
            process_covariance_from_model_config(config, spatial_description, &ap_params);
        let measurement_matrix =
            MeasurementMatrix::from_model_spatial_description(spatial_description);
        let control_matrix = ControlMatrix::from_model_config(config, spatial_description);
        let measurement_covariance =
            MeasurementCovariance::from_model_config(config, spatial_description);
        //        let kalman_gain = Gain::from_model_config(config, &measurement_matrix);
        let kalman_gain = KalmanGain::empty(
            spatial_description.voxels.count_states(),
            spatial_description.sensors.count(),
        );
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

    /// Saves the internal state of the `FunctionalDescription` to .npy files.
    ///
    /// This exports the allpass filter parameters, process covariance,
    /// measurement matrix, control matrix, measurement covariance, Kalman
    /// gain, and control function values to .npy files in the provided path.
    #[tracing::instrument(level = "trace")]
    pub fn save_npy(&self, path: &std::path::Path) {
        trace!("Saving functional description to npy");
        let path = &path.join("functional_description");
        self.ap_params.save_npy(path);
        self.process_covariance
            .save_npy(path, "process_covariance.npy");
        self.measurement_matrix.save_npy(path);
        self.control_matrix.save_npy(path);
        self.measurement_covariance.save_npy(path);
        self.kalman_gain.save_npy(path);
        self.control_function_values.save_npy(path);
    }

    pub fn to_gpu(&self, queue: &Queue) -> FunctionalDescriptionGPU {
        FunctionalDescriptionGPU {
            ap_params: self.ap_params.to_gpu(queue),
            measurement_matrix: self.measurement_matrix.to_gpu(queue),
            control_matrix: self.control_matrix.to_gpu(queue),
            process_covariance: self.process_covariance.to_gpu(queue),
            measurement_covariance: self.measurement_covariance.to_gpu(queue),
            kalman_gain: self.kalman_gain.to_gpu(queue),
            control_function_values: self.control_function_values.to_gpu(queue),
        }
    }

    pub(crate) fn update_from_gpu(&mut self, functional_description: &FunctionalDescriptionGPU) {
        self.ap_params
            .update_from_gpu(&functional_description.ap_params);
        self.measurement_matrix
            .update_from_gpu(&functional_description.measurement_matrix);
        self.control_matrix
            .update_from_gpu(&functional_description.control_matrix);
        self.process_covariance
            .update_from_gpu(&functional_description.process_covariance);
        self.measurement_covariance
            .update_from_gpu(&functional_description.measurement_covariance);
        self.kalman_gain
            .update_from_gpu(&functional_description.kalman_gain);
        self.control_function_values
            .update_from_gpu(&functional_description.control_function_values);
    }
}

/// Generates a process covariance matrix from the model configuration,
/// spatial description, and allpass filter parameters.
///
/// Uses the `process_covariance_mean` and `process_covariance_std` from
/// the model config to sample a normal distribution for the diagonal
/// of the process covariance matrix. Filters down to only the states  
/// that correspond to AP filter outputs based on `ap_params`.
#[tracing::instrument(level = "debug", skip_all)]
fn process_covariance_from_model_config(
    config: &Model,
    spatial_description: &SpatialDescription,
    ap_params: &APParameters,
) -> Gains {
    debug!("Creating process covariance matrix from model config");
    let normal = if relative_eq!(config.common.process_covariance_std, 0.0) {
        None
    } else {
        Some(
            Normal::<f32>::new(
                config.common.process_covariance_mean,
                config.common.process_covariance_std,
            )
            .unwrap(),
        )
    };
    let mut process_covariance = Gains::empty(spatial_description.voxels.count_states());
    process_covariance
        .indexed_iter_mut()
        .zip(ap_params.output_state_indices.iter())
        .filter(|((index, _), output_state_index)| {
            output_state_index.is_some() && index.0 == output_state_index.unwrap_or(0)
        })
        .for_each(|((_, variance), _)| {
            *variance = normal.map_or(config.common.process_covariance_mean, |dist| {
                dist.sample(&mut rand::thread_rng())
            });
        });
    process_covariance
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::core::config::model::{Common, Mri};

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
        let number_of_motion_steps = 2000;
        let voxels_in_dims = Dim([1000, 1, 1]);

        let _functional_description = FunctionalDescription::empty(
            number_of_states,
            number_of_sensors,
            number_of_steps,
            number_of_motion_steps,
            voxels_in_dims,
        );
    }

    #[test]
    fn from_handcrafted_model_config_no_crash() {
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

    #[test_log::test]
    fn from_mri_model_config_no_crash() {
        let config = Model {
            common: Common::default(),
            handcrafted: None,
            mri: Some(Mri::default()),
        };
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
