pub mod heart;
pub mod sensors;
pub mod voxels;

use crate::core::config::simulation::Simulation;

use self::voxels::{VoxelType, VoxelTypes};

#[derive(Debug, PartialEq)]
pub struct SpatialDescription {
    pub heart: Heart,
    pub voxels: Voxels,
    pub sensors: Sensors,
}

impl SpatialDescription {
    pub fn empty(number_of_sensors: usize, voxels_in_dims: [usize; 3]) -> SpatialDescription {
        SpatialDescription {
            heart: Heart::new(),
            voxels: Voxels::empty(voxels_in_dims),
            sensors: Sensors::empty(number_of_sensors),
        }
    }

    pub fn from_simulation_config(config: &Simulation) -> SpatialDescription {
        let heart = Heart::from_simulation_config(config);
        let voxels = Voxels::from_simulation_config(config);
        let sensors = Sensors::from_simulation_config(config);

        SpatialDescription {
            heart,
            voxels,
            sensors,
        }
    }

    pub fn get_number_of_states(&self) -> usize {
        self.voxel_types
            .values
            .iter()
            .filter(|voxel| **voxel != VoxelType::None)
            .count()
            * 3
    }

    pub fn get_number_of_sensors(&self) -> usize {
        self.sensor_positions.shape()[0]
    }
}

#[cfg(test)]
mod tests {
    use crate::core::model::spatial::voxels::VoxelType;

    use super::*;

    #[test]
    fn empty_no_crash() {
        let number_of_sensors = 300;
        let number_of_states = 3000;
        let voxels_in_dims = [1000, 1, 1];
        let _spatial_description =
            SpatialDescription::empty(number_of_sensors, number_of_states, voxels_in_dims);
    }

    #[test]
    fn number_of_states_none() {
        let number_of_sensors = 300;
        let number_of_states = 0;
        let voxels_in_dims = [1000, 1, 1];
        let spatial_description =
            SpatialDescription::empty(number_of_sensors, number_of_states, voxels_in_dims);

        assert_eq!(0, spatial_description.get_number_of_states());
    }

    fn number_of_states_some() {
        let number_of_sensors = 300;
        let number_of_states = 3;
        let voxels_in_dims = [1000, 1, 1];
        let mut spatial_description =
            SpatialDescription::empty(number_of_sensors, number_of_states, voxels_in_dims);
        spatial_description.voxel_types.values[(0, 0, 0)] = VoxelType::Atrioventricular;

        assert_eq!(3, spatial_description.get_number_of_states());
    }

    #[test]
    fn number_of_sensors() {
        let number_of_sensors = 300;
        let number_of_states = 3000;
        let voxels_in_dims = [1000, 1, 1];
        let spatial_description =
            SpatialDescription::empty(number_of_sensors, number_of_states, voxels_in_dims);

        assert_eq!(
            number_of_sensors,
            spatial_description.get_number_of_sensors()
        );
    }

    #[test]
    fn from_simulation_config_no_crash() {
        let config = Simulation::default();
        let _spatial_description = SpatialDescription::from_simulation_config(&config);
    }
}
