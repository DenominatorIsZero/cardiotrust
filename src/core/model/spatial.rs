pub mod heart;
pub mod sensors;
pub mod voxels;

use crate::core::config::model::Model;

use self::{heart::Heart, sensors::Sensors, voxels::Voxels};

use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[allow(clippy::module_name_repetitions)]
pub struct VoxelTypes {
    pub heart: Heart,
    pub voxels: Voxels,
    pub sensors: Sensors,
}

impl VoxelTypes {
    #[must_use]
    pub fn empty(number_of_sensors: usize, voxels_in_dims: [usize; 3]) -> Self {
        Self {
            heart: Heart::empty(),
            voxels: Voxels::empty(voxels_in_dims),
            sensors: Sensors::empty(number_of_sensors),
        }
    }

    #[must_use]
    pub fn from_model_config(config: &Model) -> Self {
        let heart = Heart::from_model_config(config);
        let voxels = Voxels::from_model_config(config);
        let sensors = Sensors::from_model_config(config);

        Self {
            heart,
            voxels,
            sensors,
        }
    }

    pub(crate) fn save_npy(&self, path: &std::path::Path) {
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
        let _spatial_description = VoxelTypes::empty(number_of_sensors, voxels_in_dims);
    }

    #[test]
    fn from_simulation_config_no_crash() {
        let config = Model::default();
        let _spatial_description = VoxelTypes::from_model_config(&config);
    }
}
