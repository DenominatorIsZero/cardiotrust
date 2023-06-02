pub mod heart;
pub mod sensors;
pub mod voxels;

use crate::core::config::{model::Model, simulation::Simulation};

use self::{heart::Heart, sensors::Sensors, voxels::Voxels};

#[derive(Debug, PartialEq)]
pub struct SpatialDescription {
    pub heart: Heart,
    pub voxels: Voxels,
    pub sensors: Sensors,
}

impl SpatialDescription {
    pub fn empty(number_of_sensors: usize, voxels_in_dims: [usize; 3]) -> SpatialDescription {
        SpatialDescription {
            heart: Heart::empty(),
            voxels: Voxels::empty(voxels_in_dims),
            sensors: Sensors::empty(number_of_sensors),
        }
    }

    pub fn from_model_config(config: &Model) -> SpatialDescription {
        let heart = Heart::from_model_config(config);
        let voxels = Voxels::from_model_config(config);
        let sensors = Sensors::from_model_config(config);

        SpatialDescription {
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
