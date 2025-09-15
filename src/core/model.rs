pub mod functional;
pub mod spatial;
#[cfg(test)]
mod tests;

use anyhow::{Context, Result};
use ndarray::Dim;
use serde::{Deserialize, Serialize};
use tracing::{debug, trace};

use self::{
    functional::{FunctionalDescription, FunctionalDescriptionGPU},
    spatial::SpatialDescription,
};
use super::{
    config::{model::Model as ModelConfig, simulation::Simulation},
    data::Data,
};

/// Struct representing a heart model with functional and spatial descriptions.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Model {
    pub functional_description: FunctionalDescription,
    pub spatial_description: SpatialDescription,
}

pub struct ModelGPU {
    pub functional_description: FunctionalDescriptionGPU,
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
    ) -> Result<Self> {
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

    #[tracing::instrument(level = "trace", skip_all)]
    pub fn synchronize_parameters(&mut self, data: &Data) {
        self.functional_description.measurement_matrix.assign(
            &*data
                .simulation
                .model
                .functional_description
                .measurement_matrix,
        );
        self.functional_description.measurement_covariance.assign(
            &*data
                .simulation
                .model
                .functional_description
                .measurement_covariance,
        );
        self.spatial_description.sensors.orientations_xyz.assign(
            &data
                .simulation
                .model
                .spatial_description
                .sensors
                .orientations_xyz,
        );
        self.spatial_description.sensors.positions_mm.assign(
            &data
                .simulation
                .model
                .spatial_description
                .sensors
                .positions_mm,
        );
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

    #[must_use]
    #[tracing::instrument(level = "trace", skip_all)]
    pub fn to_gpu(&self, queue: &ocl::Queue) -> Result<ModelGPU> {
        Ok(ModelGPU {
            functional_description: self.functional_description.to_gpu(queue)?,
        })
    }

    #[tracing::instrument(level = "trace", skip_all)]
    pub fn from_gpu(&mut self, model_gpu: &ModelGPU) -> Result<()> {
        self.functional_description
            .update_from_gpu(&model_gpu.functional_description)?;
        Ok(())
    }

    #[tracing::instrument(level = "trace", skip_all)]
    pub(crate) fn get_default() -> Result<Self> {
        let config = ModelConfig::default();
        let sim_config = Simulation::default();
        Self::from_model_config(&config, sim_config.sample_rate_hz, sim_config.duration_s)
    }
}
