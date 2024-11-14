pub mod functional;
pub mod spatial;
#[cfg(test)]
mod tests;

use std::error::Error;

use ndarray::Dim;
use serde::{Deserialize, Serialize};
use tracing::{debug, trace};

use self::{functional::FunctionalDescription, spatial::SpatialDescription};
use super::config::model::Model as ModelConfig;

/// Struct representing a heart model with functional and spatial descriptions.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Model {
    pub functional_description: FunctionalDescription,
    pub spatial_description: SpatialDescription,
}

impl Model {
    /// Creates an empty `Model` with the given parameters.
    #[must_use]
    #[tracing::instrument(level = "debug")]
    pub fn empty(
        number_of_states: usize,
        number_of_sensors: usize,
        number_of_steps: usize,
        voxels_in_dims: Dim<[usize; 3]>,
        number_of_beats: usize,
    ) -> Self {
        debug!("Creating empty model");
        Self {
            functional_description: FunctionalDescription::empty(
                number_of_states,
                number_of_sensors,
                number_of_steps,
                number_of_beats,
                voxels_in_dims,
            ),
            spatial_description: SpatialDescription::empty(
                number_of_sensors,
                [voxels_in_dims[0], voxels_in_dims[1], voxels_in_dims[2]],
                number_of_beats,
            ),
        }
    }

    /// Creates a new `Model` instance from the given `ModelConfig`.
    ///
    /// This converts the high-level model configuration into a `Model` instance
    /// with populated `FunctionalDescription` and `SpatialDescription`. It handles
    /// creating the model topology and computing valid model delays.
    ///
    /// # Errors
    ///
    /// This function will return an error if the model configuration does not
    /// result in valid delays or topology.
    #[tracing::instrument(level = "debug")]
    pub fn from_model_config(
        config: &ModelConfig,
        sample_rate_hz: f32,
        duration_s: f32,
    ) -> Result<Self, Box<dyn Error>> {
        debug!("Creating model from config");
        let spatial_description = SpatialDescription::from_model_config(config);
        let functional_description = FunctionalDescription::from_model_config(
            config,
            &spatial_description,
            sample_rate_hz,
            duration_s,
        )?;
        Ok(Self {
            functional_description,
            spatial_description,
        })
    }

    /// Saves the functional and spatial descriptions of the model
    /// to .npy files at the given path.
    #[tracing::instrument(level = "trace")]
    pub fn save_npy(&self, path: &std::path::Path) {
        trace!("Saving model to npy");
        self.functional_description.save_npy(path);
        self.spatial_description.save_npy(path);
    }

    #[tracing::instrument(level = "trace", skip_all)]
    pub(crate) fn update_activation_time(
        &mut self,
        activation_times: &super::data::shapes::ActivationTimePerStateMs,
    ) {
        let target = &mut self.functional_description.ap_params.activation_time_ms;
        let numbers = &self.spatial_description.voxels.numbers;
        for x in 0..target.shape()[0] {
            for y in 0..target.shape()[1] {
                for z in 0..target.shape()[2] {
                    if let Some(voxel_number) = numbers[(x, y, z)] {
                        target[(x, y, z)] = Some(activation_times[voxel_number / 3]);
                    }
                }
            }
        }
    }
}
