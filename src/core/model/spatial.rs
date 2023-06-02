pub mod voxels;
use crate::core::config::simulation::Simulation;
use ndarray::Array2;

use self::voxels::{VoxelType, VoxelTypes};

#[derive(Debug, PartialEq)]
pub struct SpatialDescription {
    pub voxel_size_mm: f32,
    pub heart_size_mm: [f32; 3],
    pub heart_origin_mm: [f32; 3],
    pub voxel_types: VoxelTypes,
    pub sensor_positions: Array2<f32>,
    pub sensors_orientations: Array2<f32>,
}

impl SpatialDescription {
    pub fn empty(
        number_of_sensors: usize,
        number_of_states: usize,
        voxels_in_dims: [usize; 3],
    ) -> SpatialDescription {
        SpatialDescription {
            voxel_size_mm: 0.0,
            heart_size_mm: [0.0, 0.0, 0.0],
            heart_origin_mm: [0.0, 0.0, 0.0],
            voxel_types: VoxelTypes::empty(voxels_in_dims),
            sensor_positions: Array2::zeros((number_of_sensors, 3)),
            sensors_orientations: Array2::zeros((number_of_sensors, 3)),
        }
    }

    pub fn from_simulation_config(config: &Simulation) -> SpatialDescription {
        let voxel_types = VoxelTypes::from_simulation_config(config);
        todo!();
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
