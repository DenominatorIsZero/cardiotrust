pub mod heart;
pub mod nifti;
pub mod sensors;
pub mod voxels;

use serde::{Deserialize, Serialize};
use tracing::{debug, trace};

use self::{heart::Heart, sensors::Sensors, voxels::Voxels};
use crate::core::config::model::Model;

/// Struct containing fields for the heart,
/// voxels and sensors spatial model components.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[allow(clippy::module_name_repetitions)]
pub struct SpatialDescription {
    pub heart: Heart,
    pub voxels: Voxels,
    pub sensors: Sensors,
}

impl SpatialDescription {
    /// Creates an empty `SpatialDescription` struct with the given number of
    /// sensors and voxel dimensions.
    #[must_use]
    #[tracing::instrument(level = "debug")]
    pub fn empty(number_of_sensors: usize, voxels_in_dims: [usize; 3]) -> Self {
        debug!("Creating empty spatial description");
        Self {
            heart: Heart::empty(),
            voxels: Voxels::empty(voxels_in_dims),
            sensors: Sensors::empty(number_of_sensors),
        }
    }

    /// Creates a `SpatialDescription` from the given [`Model`] configuration.
    ///
    /// Constructs the `heart`, `voxels`, and `sensors` fields by calling their
    /// respective `from_model_config()` methods.
    #[must_use]
    #[tracing::instrument(level = "debug")]
    pub fn from_model_config(config: &Model) -> Self {
        debug!("Creating spatial description from model config");
        let (heart, voxels) = if config.handcrafted.is_some() {
            let heart = Heart::from_handcrafted_model_config(config);
            let voxels = Voxels::from_handcrafted_model_config(config);
            (heart, voxels)
        } else {
            let heart = Heart::from_mri_model_config(config);
            let voxels = Voxels::from_mri_model_config(config);
            (heart, voxels)
        };

        let sensors = Sensors::from_model_config(&config.common);

        Self {
            heart,
            voxels,
            sensors,
        }
    }

    /// Saves the spatial description components to .npy files.
    ///
    /// Saves the `heart`, `voxels`, and `sensors` fields to .npy files
    /// in the given `path`.
    #[tracing::instrument(level = "trace")]
    pub(crate) fn save_npy(&self, path: &std::path::Path) {
        trace!("Saving spatial description to npy");
        let path = &path.join("spatial_description");
        self.heart.save_npy(path);
        self.voxels.save_npy(path);
        self.sensors.save_npy(path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_no_crash() {
        let number_of_sensors = 300;
        let voxels_in_dims = [1000, 1, 1];
        let _spatial_description = SpatialDescription::empty(number_of_sensors, voxels_in_dims);
    }

    #[test]
    fn from_simulation_config_no_crash() {
        let config = Model::default();
        let _spatial_description = SpatialDescription::from_model_config(&config);
    }
}
