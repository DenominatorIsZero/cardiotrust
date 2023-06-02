use approx::relative_eq;
use ndarray::Array3;

use crate::core::config::{model::Model, simulation::Simulation, ModelPreset};

#[derive(Debug, PartialEq)]
pub struct Voxels {
    pub size_mm: f32,
    pub types: VoxelTypes,
}

impl Voxels {
    pub fn empty(voxels_in_dims: [usize; 3]) -> Voxels {
        Voxels {
            size_mm: 0.0,
            types: VoxelTypes::empty(voxels_in_dims),
        }
    }

    pub fn from_model_config(config: &Model) -> Voxels {
        Voxels {
            size_mm: config.voxel_size_mm,
            types: VoxelTypes::from_simulation_config(config),
        }
    }

    pub fn count(&self) -> usize {
        self.count_xyz().iter().product()
    }

    pub fn count_xyz(&self) -> &[usize] {
        self.types.values.shape()
    }

    pub fn count_states(&self) -> usize {
        self.types
            .values
            .iter()
            .filter(|voxel| **voxel != VoxelType::None)
            .count()
            * 3
    }
}

#[derive(Debug, PartialEq)]
pub struct VoxelTypes {
    pub values: Array3<VoxelType>,
}

impl VoxelTypes {
    pub fn empty(voxels_in_dims: [usize; 3]) -> VoxelTypes {
        VoxelTypes {
            values: Array3::default(voxels_in_dims),
        }
    }

    pub fn from_simulation_config(config: &Model) -> VoxelTypes {
        // Config Parameters
        let voxel_size_mm = config.voxel_size_mm;
        let heart_size_mm = config.heart_size_mm;

        assert!(relative_eq!(heart_size_mm[0] % voxel_size_mm, 0.0));
        assert!(relative_eq!(heart_size_mm[1] % voxel_size_mm, 0.0));
        assert!(relative_eq!(heart_size_mm[2] % voxel_size_mm, 0.0));

        let mut voxels_in_dims = [0, 0, 0];
        voxels_in_dims
            .iter_mut()
            .zip(heart_size_mm.iter())
            .for_each(|(number, size)| *number = (size / voxel_size_mm) as usize);
        assert!(voxels_in_dims[0] > 0);
        assert!(voxels_in_dims[1] > 0);
        assert!(voxels_in_dims[2] > 0);

        // Fixed Parameters - will add to config at later time
        let sa_x_center_percentage = 0.8;
        let sa_y_center_percentage = 0.15;
        let atrium_y_stop_percentage = 0.3;
        let av_x_center_percentage = 0.5;
        let hps_y_stop_percentage = 0.85;
        let hps_x_start_percentage = 0.2;
        let hps_x_stop_percentage = 0.8;
        let hps_y_up_percentage = 0.5;
        let pathology_x_start_percentage = 0.1;
        let pathology_x_stop_percentage = 0.3;
        let pathology_y_start_percentage = 0.7;
        let pathology_y_stop_percentage = 0.8;
        // Derived Parameters
        let sa_x_center_index = (voxels_in_dims[0] as f32 / sa_x_center_percentage) as usize;
        let sa_y_center_index = (voxels_in_dims[1] as f32 / sa_y_center_percentage) as usize;
        let atrium_y_stop_index = (voxels_in_dims[1] as f32 / atrium_y_stop_percentage) as usize;
        let av_x_center_index = (voxels_in_dims[0] as f32 / av_x_center_percentage) as usize;
        let hps_y_stop_index = (voxels_in_dims[1] as f32 / hps_y_stop_percentage) as usize;
        let hps_x_start_index = (voxels_in_dims[0] as f32 / hps_x_start_percentage) as usize;
        let hps_x_stop_index = (voxels_in_dims[0] as f32 / hps_x_stop_percentage) as usize;
        let hps_y_up_index = (voxels_in_dims[1] as f32 / hps_y_up_percentage) as usize;
        let pathology_x_start_index =
            (voxels_in_dims[0] as f32 / pathology_x_start_percentage) as usize;
        let pathology_x_stop_index =
            (voxels_in_dims[0] as f32 / pathology_x_stop_percentage) as usize;
        let pathology_y_start_index =
            (voxels_in_dims[1] as f32 / pathology_y_start_percentage) as usize;
        let pathology_y_stop_index =
            (voxels_in_dims[1] as f32 / pathology_y_stop_percentage) as usize;

        let mut voxel_types = VoxelTypes::empty(voxels_in_dims);
        voxel_types
            .values
            .indexed_iter_mut()
            .for_each(|((x, y, _z), voxel_type)| {
                if (config.pathological)
                    && (x >= pathology_x_start_index && x < pathology_x_stop_index)
                    && (pathology_y_start_index <= y && y < pathology_y_stop_index)
                {
                    *voxel_type = VoxelType::Pathological;
                    return;
                }
                if x == sa_x_center_index && y == sa_y_center_index {
                    *voxel_type = VoxelType::Sinoatrial;
                    return;
                }
                if x == av_x_center_index && y == atrium_y_stop_index {
                    *voxel_type = VoxelType::Atrioventricular;
                    return;
                }
                // HPS Downward section
                if x == av_x_center_index && y > atrium_y_stop_index && y < hps_y_stop_index {
                    *voxel_type = VoxelType::HPS;
                    return;
                }
                // HPS Across
                if x >= hps_x_start_index && x < hps_x_stop_index && y == hps_y_stop_index - 1 {
                    *voxel_type = VoxelType::HPS;
                    return;
                }
                // HPS Up
                if (x == hps_x_start_index || x == hps_x_stop_index - 1)
                    && y >= hps_y_up_index
                    && y < hps_y_stop_index
                {
                    *voxel_type = VoxelType::HPS;
                    return;
                }
                if y < atrium_y_stop_index {
                    *voxel_type = VoxelType::Atrium;
                    return;
                }
                *voxel_type = VoxelType::Ventricle;
            });
        voxel_types
    }
}

fn get_voxel_position_mm(voxel_size_mm: f32, x: usize, y: usize, z: usize) -> [f32; 3] {
    let offset = voxel_size_mm / 2.0;
    [
        voxel_size_mm * x as f32 + offset,
        voxel_size_mm * y as f32 + offset,
        voxel_size_mm * z as f32 + offset,
    ]
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
    fn count_states_none() {
        let voxels_in_dims = [1000, 1, 1];
        let voxels = Voxels::empty(voxels_in_dims);

        assert_eq!(0, voxels.count_states());
    }

    #[test]
    fn number_of_states_some() {
        let voxels_in_dims = [1000, 1, 1];
        let mut voxels = Voxels::empty(voxels_in_dims);
        voxels.types.values[(0, 0, 0)] = VoxelType::Atrioventricular;

        assert_eq!(3, voxels.count_states());
    }

    #[test]
    fn no_pathology_full_states() {
        let mut config = Model::default();
        config.heart_size_mm = [10.0, 10.0, 10.0];
        config.voxel_size_mm = 1.0;

        let voxels = Voxels::from_model_config(&config);

        assert_eq!(1000, voxels.count());
        assert_eq!(3000, voxels.count_states());
    }
}
