use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::core::model::spatial::voxels::VoxelType;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Model {
    pub control_function: ControlFunction,
    pub pathological: bool,
    pub sensors_per_axis: [usize; 3],
    pub sensor_array_size_mm: [f32; 3],   // TODO: Add to UI
    pub sensor_array_origin_mm: [f32; 3], // TODO: Add to UI
    pub voxel_size_mm: f32,               // TODO: Add to UI
    pub heart_size_mm: [f32; 3],          // TODO: Add to UI
    pub heart_origin_mm: [f32; 3],        // TODO: Add to UI
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
    pub propagation_velocities_m_per_s: HashMap<VoxelType, f32>, // TOOD: Add to UI
    pub current_factor_in_pathology: f32,
}

impl Model {
    pub fn default() -> Model {
        let mut propagation_velocities_m_per_s = HashMap::new();
        propagation_velocities_m_per_s.insert(VoxelType::Sinoatrial, 1.1);
        propagation_velocities_m_per_s.insert(VoxelType::Atrium, 1.1);
        propagation_velocities_m_per_s.insert(VoxelType::Atrioventricular, 0.012);
        propagation_velocities_m_per_s.insert(VoxelType::HPS, 4.5);
        propagation_velocities_m_per_s.insert(VoxelType::Ventricle, 1.1);
        propagation_velocities_m_per_s.insert(VoxelType::Pathological, 0.1);

        Model {
            control_function: ControlFunction::Ohara,
            pathological: false,
            sensors_per_axis: [4, 4, 2],
            sensor_array_size_mm: [250.0, 250.0, 100.0],
            sensor_array_origin_mm: [-125.0, -125.0, 200.0],
            voxel_size_mm: 2.5,
            heart_size_mm: [65.0, 92.5, 2.5],
            heart_origin_mm: [0.0, 0.0, 0.0],
            measurement_covariance_mean: 1e-30,
            measurement_covariance_std: 0.0,
            process_covariance_mean: 1e-30,
            process_covariance_std: 0.0,
            apply_system_update: true,
            propagation_velocities_m_per_s,
            current_factor_in_pathology: 0.01,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum ControlFunction {
    Sinosodal,
    Ohara,
}
