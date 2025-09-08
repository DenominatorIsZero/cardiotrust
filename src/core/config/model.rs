use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::core::model::spatial::voxels::VoxelType;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Model {
    pub common: Common,
    pub handcrafted: Option<Handcrafted>,
    pub mri: Option<Mri>,
}

impl Default for Model {
    #[tracing::instrument(level = "debug")]
    fn default() -> Self {
        debug!("Creating default model");
        let mut config = Self {
            common: Common::default(),
            handcrafted: Some(Handcrafted::default()),
            mri: None,
        };

        if config.handcrafted.is_some() {
            config.common.heart_offset_mm = DEFAULT_HEART_OFFSET_HANDCRAFTED;
        } else {
            config.common.heart_offset_mm = DEFAULT_HEART_OFFSET_MRI;
        }
        config
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Handcrafted {
    pub heart_size_mm: [f32; 3],
    pub sa_x_center_percentage: f32,
    pub sa_y_center_percentage: f32,
    pub atrium_y_start_percentage: f32,
    pub av_x_center_percentage: f32,
    pub hps_y_stop_percentage: f32,
    pub hps_x_start_percentage: f32,
    pub hps_x_stop_percentage: f32,
    pub hps_y_up_percentage: f32,
    pub pathology_x_start_percentage: f32,
    pub pathology_x_stop_percentage: f32,
    pub pathology_y_start_percentage: f32,
    pub pathology_y_stop_percentage: f32,
    pub include_atrium: bool,
    pub include_av: bool,
    pub include_hps: bool,
}

impl Default for Handcrafted {
    /// Returns a default Model configuration with reasonable default values.
    ///
    /// The default includes:
    /// - Default control function of Ohara
    /// - Pathological set to false  
    /// - Default sensor configuration
    /// - Default voxel and heart sizes
    /// - Default covariance values
    /// - Default propagation velocities for each voxel type
    /// - Default percentages for positioning various heart components
    ///
    /// This provides a reasonable starting point for configuring a Model.
    /// Individual properties can be overriden as needed.
    #[tracing::instrument(level = "debug")]
    fn default() -> Self {
        debug!("Creating Handcrafted model");
        Self {
            heart_size_mm: [65.0, 92.5, 2.5],
            sa_x_center_percentage: 0.2,
            sa_y_center_percentage: 0.85,
            atrium_y_start_percentage: 0.7,
            av_x_center_percentage: 0.5,
            hps_y_stop_percentage: 0.15,
            hps_x_start_percentage: 0.2,
            hps_x_stop_percentage: 0.8,
            hps_y_up_percentage: 0.5,
            pathology_x_start_percentage: 0.0,
            pathology_x_stop_percentage: 0.2,
            pathology_y_start_percentage: 0.5,
            pathology_y_stop_percentage: 0.7,
            include_atrium: true,
            include_av: true,
            include_hps: true,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Mri {
    pub path: PathBuf,
}

impl Default for Mri {
    #[tracing::instrument(level = "debug")]
    fn default() -> Self {
        debug!("Creating MriScan model");

        Self {
            path: Path::new("assets/segmentation.nii").to_path_buf(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Copy)]
pub enum ControlFunction {
    Ohara,
    Triangle,
    Ramp,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub enum SensorArrayGeometry {
    Cube,
    SparseCube,
    Cylinder,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub enum SensorArrayMotion {
    Static,
    Grid,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Common {
    pub control_function: ControlFunction,
    pub pathological: bool,
    pub sensor_array_geometry: SensorArrayGeometry,
    pub sensor_array_motion: SensorArrayMotion,
    pub three_d_sensors: bool,            // used for both kinds
    pub number_of_sensors: usize,         // used for cylinder and sparse cube
    pub sensor_array_radius_mm: f32,      // used for cylinder only
    pub sensors_per_axis: [usize; 3],     // used for cube only
    pub sensor_array_size_mm: [f32; 3],   // used for cube only
    pub sensor_array_origin_mm: [f32; 3], // used for both kinds
    pub sensor_array_motion_range_mm: [f32; 3],
    pub sensor_array_motion_steps: [usize; 3],
    pub voxel_size_mm: f32,
    pub heart_offset_mm: [f32; 3],
    pub measurement_covariance_mean: f32,
    // the measurement noise covariance matrix will be a diagonal matrix
    // if std is set to zero, every value will be set to mean
    // otherwise the elements along the main diagonal will be drawn from a
    // normal distribution
    pub measurement_covariance_std: f32,
    pub propagation_velocities_m_per_s: HashMap<VoxelType, f32>,
    pub current_factor_in_pathology: f32,
}

pub const DEFAULT_HEART_OFFSET_HANDCRAFTED: [f32; 3] = [25.0, -250.0, 150.0];
pub const DEFAULT_HEART_OFFSET_MRI: [f32; 3] = [-130.0, -300.0, -30.0];
pub const DEFAULT_SENSOR_ORIGIN_CUBE: [f32; 3] = [-50.0, -300.0, 270.0];
pub const DEFAULT_SENSOR_ORIGIN_CYLINDER: [f32; 3] = [0.0, -200.0, 100.0];

impl Default for Common {
    #[tracing::instrument(level = "debug")]
    fn default() -> Self {
        debug!("Creating default model");
        let mut propagation_velocities_m_per_s = HashMap::new();
        propagation_velocities_m_per_s.insert(VoxelType::Sinoatrial, 1.1);
        propagation_velocities_m_per_s.insert(VoxelType::Atrium, 1.1);
        propagation_velocities_m_per_s.insert(VoxelType::Atrioventricular, 0.012);
        propagation_velocities_m_per_s.insert(VoxelType::HPS, 4.5);
        propagation_velocities_m_per_s.insert(VoxelType::Ventricle, 1.1);
        propagation_velocities_m_per_s.insert(VoxelType::Pathological, 0.1);

        let mut config = Self {
            control_function: ControlFunction::Ohara,
            pathological: false,
            sensor_array_geometry: SensorArrayGeometry::Cube,
            sensor_array_motion: SensorArrayMotion::Static,
            three_d_sensors: true,
            number_of_sensors: 40,
            sensor_array_radius_mm: 400.0,
            sensors_per_axis: [4, 4, 4],
            sensor_array_size_mm: [250.0, 250.0, 100.0],
            sensor_array_origin_mm: DEFAULT_SENSOR_ORIGIN_CUBE,
            sensor_array_motion_range_mm: [100.0, 200.0, 100.0],
            sensor_array_motion_steps: [1, 2, 1],
            voxel_size_mm: 2.5,
            heart_offset_mm: [25.0, -250.0, 150.0],
            measurement_covariance_mean: 1e-3,
            measurement_covariance_std: 0.0,
            propagation_velocities_m_per_s,
            current_factor_in_pathology: 0.00,
        };
        match config.sensor_array_geometry {
            SensorArrayGeometry::Cube | SensorArrayGeometry::SparseCube => {
                config.sensor_array_origin_mm = DEFAULT_SENSOR_ORIGIN_CUBE;
            }
            SensorArrayGeometry::Cylinder => {
                config.sensor_array_origin_mm = DEFAULT_SENSOR_ORIGIN_CYLINDER;
            }
        }
        config
    }
}
