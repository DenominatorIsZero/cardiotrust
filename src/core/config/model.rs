use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::core::model::spatial::voxels::VoxelType;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Model {
    pub control_function: ControlFunction,
    pub pathological: bool,
    pub sensors_per_axis: [usize; 3],
    pub sensor_array_size_mm: [f32; 3],
    pub sensor_array_origin_mm: [f32; 3],
    pub voxel_size_mm: f32,
    pub heart_size_mm: [f32; 3],
    pub heart_origin_mm: [f32; 3],
    pub measurement_covariance_mean: f32,
    // the measurement noise covariance matrix will be a diagonal matrix
    // if std is set to zero, every value will be set to mean
    // otherwise the elements along the main diagonal will be drawn from a
    // normal distribution
    pub measurement_covariance_std: f32,
    pub process_covariance_mean: f32,
    // the covariance noise covariance matrix will be a diagonal matrix
    // if std is set to zero, every value will be set to mean
    // otherwise the elements along the main diagonal will be drawn from a
    // normal distribution
    pub process_covariance_std: f32,
    pub apply_system_update: bool,
    pub propagation_velocities_m_per_s: HashMap<VoxelType, f32>,
    pub current_factor_in_pathology: f32,
    pub sa_x_center_percentage: f32,
    pub sa_y_center_percentage: f32,
    pub atrium_y_stop_percentage: f32,
    pub av_x_center_percentage: f32,
    pub hps_y_stop_percentage: f32,
    pub hps_x_start_percentage: f32,
    pub hps_x_stop_percentage: f32,
    pub hps_y_up_percentage: f32,
    pub pathology_x_start_percentage: f32,
    pub pathology_x_stop_percentage: f32,
    pub pathology_y_start_percentage: f32,
    pub pathology_y_stop_percentage: f32,
}

impl Default for Model {
    /// Returns a default Model configuration with reasonable default values.
    ///
    /// The default includes:
    /// - Default control function of Ohara
    /// - Pathological set to false  
    /// - Default sensor configuration
    /// - Default voxel and heart sizes
    /// - Default covariance values
    /// - `apply_system_update` set to false
    /// - Default propagation velocities for each voxel type
    /// - Default percentages for positioning various heart components
    ///
    /// This provides a reasonable starting point for configuring a Model.
    /// Individual properties can be overriden as needed.
    fn default() -> Self {
        let mut propagation_velocities_m_per_s = HashMap::new();
        propagation_velocities_m_per_s.insert(VoxelType::Sinoatrial, 1.1);
        propagation_velocities_m_per_s.insert(VoxelType::Atrium, 1.1);
        propagation_velocities_m_per_s.insert(VoxelType::Atrioventricular, 0.012);
        propagation_velocities_m_per_s.insert(VoxelType::HPS, 4.5);
        propagation_velocities_m_per_s.insert(VoxelType::Ventricle, 1.1);
        propagation_velocities_m_per_s.insert(VoxelType::Pathological, 0.1);

        Self {
            control_function: ControlFunction::Ohara,
            pathological: false,
            sensors_per_axis: [4, 4, 3],
            sensor_array_size_mm: [250.0, 250.0, 100.0],
            sensor_array_origin_mm: [-125.0, -125.0, 200.0],
            voxel_size_mm: 2.5,
            heart_size_mm: [65.0, 92.5, 2.5],
            heart_origin_mm: [0.0, 0.0, 0.0],
            measurement_covariance_mean: 1e-3,
            measurement_covariance_std: 0.0,
            process_covariance_mean: 1e-5,
            process_covariance_std: 0.0,
            apply_system_update: false,
            propagation_velocities_m_per_s,
            current_factor_in_pathology: 0.00,
            sa_x_center_percentage: 0.2,
            sa_y_center_percentage: 0.15,
            atrium_y_stop_percentage: 0.3,
            av_x_center_percentage: 0.5,
            hps_y_stop_percentage: 0.85,
            hps_x_start_percentage: 0.2,
            hps_x_stop_percentage: 0.8,
            hps_y_up_percentage: 0.5,
            pathology_x_start_percentage: 0.0,
            pathology_x_stop_percentage: 0.2,
            pathology_y_start_percentage: 0.3,
            pathology_y_stop_percentage: 0.5,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub enum ControlFunction {
    Sinosodal,
    Ohara,
}
