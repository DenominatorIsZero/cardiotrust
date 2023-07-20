pub mod heart;
pub mod sensors;
pub mod voxels;

use crate::core::config::model::Model;

use self::{heart::Heart, sensors::Sensors, voxels::Voxels};

#[derive(Debug, PartialEq, Clone)]
#[allow(clippy::module_name_repetitions)]
pub struct SpatialDescription {
    pub heart: Heart,
    pub voxels: Voxels,
    pub sensors: Sensors,
}

impl SpatialDescription {
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
