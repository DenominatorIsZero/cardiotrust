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
        // Config Parameters
        let voxel_size_mm = config.voxel_size_mm;
        let heart_size_mm = config.heart_size_mm;
        let heart_origin_mm = config.heart_origin_mm;
        // Fixed Parameters - will add to config at later time
        let atrium_y_stop_percentage = 0.3;
        let av_x_center_percentage = 0.5;
        let hps_y_stop_percentage = 0.85;
        let hps_x_left_percentage = 0.2;
        let hps_x_right_percentage = 0.8;
        let hps_y_up_percentage = 0.5;
        let pathology_x_start_percentage = 0.1;
        let pathology_x_stop_percentage = 0.3;
        let pathology_y_start_percentage = 0.7;
        let pathology_y_stop_percentage = 0.8;
        // Derived Parameters
        let atrium_y_stop_mm = heart_size_mm[1] * atrium_y_stop_percentage;
        let av_x_center_mm = heart_size_mm[0] * av_x_center_percentage;
        let hps_y_stop_mm = heart_size_mm[0] * hps_y_stop_percentage;
        let hps_x_left_mm = heart_size_mm[0] * hps_x_left_percentage;
        let hps_x_right_mm = heart_size_mm[0] * 0.8;
        let hps_y_up_mm = heart_size_mm[1] * 0.5;
        let pathology_x_start_mm = heart_size_mm[0] * pathology_x_start_percentage;
        let pathology_x_stop_mm = heart_size_mm[0] * pathology_x_stop_percentage;
        let pathology_y_start_mm = heart_size_mm[1] * pathology_y_start_percentage;
        let pathology_y_stop_mm = heart_size_mm[1] * pathology_y_stop_percentage;
    }

    pub fn get_number_of_states(&self) -> usize {
        self.voxel_types
            .iter()
            .filter(|voxel| **voxel != VoxelType::None)
            .count()
            * 3
    }

    pub fn get_number_of_sensors(&self) -> usize {
        self.sensor_positions.shape()[0]
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
    fn number_of_states_none() {
        let number_of_sensors = 300;
        let number_of_states = 3000;
        let spatial_description = SpatialDescription::empty(number_of_sensors, number_of_states);

        assert_eq!(0, spatial_description.get_number_of_states());
    }

    fn number_of_states_some() {
        let number_of_sensors = 300;
        let number_of_states = 3000;
        let mut spatial_description =
            SpatialDescription::empty(number_of_sensors, number_of_states);
        spatial_description.voxel_types[0] = VoxelType::Atrioventricular;

        assert_eq!(3, spatial_description.get_number_of_states());
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
