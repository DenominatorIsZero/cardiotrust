pub mod functional;
pub mod spatial;

use ndarray::Dim;
use serde::{Deserialize, Serialize};
use std::error::Error;

use self::{functional::FunctionalDescription, spatial::SpatialDescription};
use super::config::model::Model as ModelConfig;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Model {
    pub functional_description: FunctionalDescription,
    pub spatial_description: SpatialDescription,
}

impl Model {
    #[must_use]
    pub fn empty(
        number_of_states: usize,
        number_of_sensors: usize,
        number_of_steps: usize,
        voxels_in_dims: Dim<[usize; 3]>,
    ) -> Self {
        Self {
            functional_description: FunctionalDescription::empty(
                number_of_states,
                number_of_sensors,
                number_of_steps,
                voxels_in_dims,
            ),
            spatial_description: SpatialDescription::empty(
                number_of_sensors,
                [voxels_in_dims[0], voxels_in_dims[1], voxels_in_dims[2]],
            ),
        }
    }
    /// .
    ///
    /// # Errors
    ///
    /// This function will return an error if model parameters do not result in valid delays.
    pub fn from_model_config(
        config: &ModelConfig,
        sample_rate_hz: f32,
        duration_s: f32,
    ) -> Result<Self, Box<dyn Error>> {
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

    pub fn save_npy(&self, path: &std::path::Path) {
        self.functional_description.save_npy(path);
        self.spatial_description.save_npy(path);
    }
}
