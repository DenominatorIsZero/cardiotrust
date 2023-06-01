use crate::core::config::simulation::Simulation;
use ndarray::{Array1, Array2};

#[derive(Debug, PartialEq)]
pub struct SpatialDescription {
    pub voxel_size_mm: f32,
    pub heart_size_mm: [f32; 3],
    pub heart_origin_mm: [f32; 3],
    pub voxel_types: Array1<VoxelType>,
    pub sensor_positions: Array2<f32>,
    pub sensors_orientations: Array2<f32>,
}

impl SpatialDescription {
    pub fn empty(number_of_sensors: usize, number_of_states: usize) -> SpatialDescription {
        SpatialDescription {
            voxel_size_mm: 0.0,
            heart_size_mm: [0.0, 0.0, 0.0],
            heart_origin_mm: [0.0, 0.0, 0.0],
            voxel_types: Array1::default(number_of_states / 3),
            sensor_positions: Array2::zeros((number_of_sensors, 3)),
            sensors_orientations: Array2::zeros((number_of_sensors, 3)),
        }
    }

    pub fn from_simulation_config(config: &Simulation) -> SpatialDescription {
        todo!();
    }

    pub fn get_number_of_states(&self) -> usize {
        todo!();
    }

    pub fn get_number_of_sensors(&self) -> usize {
        todo!();
    }
}

#[derive(Default, Debug, PartialEq)]
pub enum VoxelType {
    #[default]
    None,
    Sinoatrial,
    Atrium,
    Atrioventricular,
    HPS,
    Ventricle,
    Pathological,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_no_crash() {
        let number_of_sensors = 300;
        let number_of_states = 3000;
        let _spatial_description = SpatialDescription::empty(number_of_sensors, number_of_states);
    }

    #[test]
    fn number_of_states() {
        let number_of_sensors = 300;
        let number_of_states = 3000;
        let spatial_description = SpatialDescription::empty(number_of_sensors, number_of_states);

        assert_eq!(number_of_states, spatial_description.get_number_of_states());
    }

    #[test]
    fn number_of_sensors() {
        let number_of_sensors = 300;
        let number_of_states = 3000;
        let spatial_description = SpatialDescription::empty(number_of_sensors, number_of_states);

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
