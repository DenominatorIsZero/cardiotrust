use approx::relative_eq;
use ndarray::{arr1, s, Array3, Array4};
use serde::{Deserialize, Serialize};

use crate::core::config::model::Model;

#[derive(Debug, PartialEq)]
pub struct Voxels {
    pub size_mm: f32,
    pub types: VoxelTypes,
    pub numbers: VoxelNumbers,
    pub positions_mm: VoxelPositions,
}

impl Voxels {
    pub fn empty(voxels_in_dims: [usize; 3]) -> Voxels {
        Voxels {
            size_mm: 0.0,
            types: VoxelTypes::empty(voxels_in_dims),
            numbers: VoxelNumbers::empty(voxels_in_dims),
            positions_mm: VoxelPositions::empty(voxels_in_dims),
        }
    }

    pub fn from_model_config(config: &Model) -> Voxels {
        let types = VoxelTypes::from_simulation_config(config);
        let numbers = VoxelNumbers::from_voxel_types(&types);
        let positions = VoxelPositions::from_model_config(config, &types);
        Voxels {
            size_mm: config.voxel_size_mm,
            types,
            numbers,
            positions_mm: positions,
        }
    }

    pub fn count(&self) -> usize {
        self.count_xyz().iter().product()
    }

    pub fn count_xyz(&self) -> [usize; 3] {
        let shape = self.types.values.raw_dim();
        [shape[0], shape[1], shape[2]]
    }

    pub fn count_states(&self) -> usize {
        self.types
            .values
            .iter()
            .filter(|voxel| **voxel != VoxelType::None)
            .count()
            * 3
    }

    pub fn is_valid_index(&self, index: [i32; 3]) -> bool {
        let [x, y, z] = index;
        let [x_max, y_max, z_max] = self.count_xyz();
        (0 <= x && x < (x_max as i32))
            && (0 <= y && y < (y_max as i32))
            && (0 < z && z < (z_max as i32))
            && (self.types.values[(x as usize, y as usize, z as usize)] != VoxelType::None)
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

        let mut voxels_in_dims = [0, 0, 0];
        voxels_in_dims
            .iter_mut()
            .zip(heart_size_mm.iter())
            .for_each(|(number, size)| *number = (size / voxel_size_mm) as usize);

        voxels_in_dims
            .iter_mut()
            .for_each(|v| *v = if *v == 0 { 1 } else { *v });

        // Fixed Parameters - will add to config at later time
        let sa_x_center_percentage = 0.2;
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
        let sa_x_center_index = (voxels_in_dims[0] as f32 * sa_x_center_percentage) as usize;
        let sa_y_center_index = (voxels_in_dims[1] as f32 * sa_y_center_percentage) as usize;
        let atrium_y_stop_index = (voxels_in_dims[1] as f32 * atrium_y_stop_percentage) as usize;
        let av_x_center_index = (voxels_in_dims[0] as f32 * av_x_center_percentage) as usize;
        let hps_y_stop_index = (voxels_in_dims[1] as f32 * hps_y_stop_percentage) as usize;
        let hps_x_start_index = (voxels_in_dims[0] as f32 * hps_x_start_percentage) as usize;
        let hps_x_stop_index = (voxels_in_dims[0] as f32 * hps_x_stop_percentage) as usize;
        let hps_y_up_index = (voxels_in_dims[1] as f32 * hps_y_up_percentage) as usize;
        let pathology_x_start_index =
            (voxels_in_dims[0] as f32 * pathology_x_start_percentage) as usize;
        let pathology_x_stop_index =
            (voxels_in_dims[0] as f32 * pathology_x_stop_percentage) as usize;
        let pathology_y_start_index =
            (voxels_in_dims[1] as f32 * pathology_y_start_percentage) as usize;
        let pathology_y_stop_index =
            (voxels_in_dims[1] as f32 * pathology_y_stop_percentage) as usize;

        let mut voxel_types = VoxelTypes::empty(voxels_in_dims);
        voxel_types
            .values
            .indexed_iter_mut()
            .for_each(|((x, y, _z), voxel_type)| {
                if (config.pathological)
                    && (x >= pathology_x_start_index && x <= pathology_x_stop_index)
                    && (pathology_y_start_index <= y && y <= pathology_y_stop_index)
                {
                    *voxel_type = VoxelType::Pathological;
                } else if (x == sa_x_center_index) && (y == sa_y_center_index) {
                    *voxel_type = VoxelType::Sinoatrial;
                } else if x == av_x_center_index && y == atrium_y_stop_index {
                    *voxel_type = VoxelType::Atrioventricular;
                }
                // HPS Downward section
                else if x == av_x_center_index && y > atrium_y_stop_index && y < hps_y_stop_index
                {
                    *voxel_type = VoxelType::HPS;
                }
                // HPS Across
                else if x >= hps_x_start_index
                    && x <= hps_x_stop_index
                    && y == hps_y_stop_index - 1
                {
                    *voxel_type = VoxelType::HPS;
                }
                // HPS Up
                else if (x == hps_x_start_index || x == hps_x_stop_index)
                    && y >= hps_y_up_index
                    && y < hps_y_stop_index
                {
                    *voxel_type = VoxelType::HPS;
                } else if y < atrium_y_stop_index {
                    *voxel_type = VoxelType::Atrium;
                } else {
                    *voxel_type = VoxelType::Ventricle;
                }
            });
        voxel_types
    }
}

#[derive(Debug, PartialEq)]
pub struct VoxelNumbers {
    pub values: Array3<Option<usize>>,
}

impl VoxelNumbers {
    pub fn empty(voxels_in_dims: [usize; 3]) -> VoxelNumbers {
        VoxelNumbers {
            values: Array3::default(voxels_in_dims),
        }
    }

    pub fn from_voxel_types(types: &VoxelTypes) -> VoxelNumbers {
        let mut numbers = VoxelNumbers {
            values: Array3::default(types.values.raw_dim()),
        };

        let mut current_number = 0;
        numbers
            .values
            .iter_mut()
            .zip(types.values.iter())
            .for_each(|(number, voxel_type)| {
                if *voxel_type != VoxelType::None {
                    *number = Some(current_number);
                    current_number += 3;
                } else {
                    *number = None;
                }
            });
        numbers
    }
}

#[derive(Debug, PartialEq)]
pub struct VoxelPositions {
    pub values: Array4<f32>,
}

impl VoxelPositions {
    pub fn empty(voxels_in_dims: [usize; 3]) -> VoxelPositions {
        VoxelPositions {
            values: Array4::zeros((voxels_in_dims[0], voxels_in_dims[1], voxels_in_dims[2], 3)),
        }
    }

    pub fn from_model_config(config: &Model, types: &VoxelTypes) -> VoxelPositions {
        let shape = types.values.raw_dim();
        let mut positions = VoxelPositions::empty([shape[0], shape[1], shape[2]]);
        let offset = config.voxel_size_mm / 2.0;

        types.values.indexed_iter().for_each(|((x, y, z), _)| {
            let position = arr1(&[
                config.voxel_size_mm * x as f32 + offset,
                config.voxel_size_mm * y as f32 + offset,
                config.voxel_size_mm * z as f32 + offset,
            ]);
            positions
                .values
                .slice_mut(s![x, y, z, ..])
                .assign(&position);
        });
        positions
    }
}

#[derive(Default, Debug, PartialEq, Eq, Hash, Deserialize, Serialize, Copy, Clone)]
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

pub fn is_connection_allowed(output_voxel_type: &VoxelType, input_voxel_type: &VoxelType) -> bool {
    match output_voxel_type {
        VoxelType::None => false,
        VoxelType::Sinoatrial => [VoxelType::Atrium].contains(input_voxel_type),
        VoxelType::Atrium => [
            VoxelType::Sinoatrial,
            VoxelType::Atrium,
            VoxelType::Atrioventricular,
        ]
        .contains(input_voxel_type),
        VoxelType::Atrioventricular => {
            [VoxelType::Atrium, VoxelType::HPS].contains(input_voxel_type)
        }
        VoxelType::HPS => [
            VoxelType::HPS,
            VoxelType::Atrioventricular,
            VoxelType::Ventricle,
        ]
        .contains(input_voxel_type),
        VoxelType::Ventricle => [VoxelType::Ventricle, VoxelType::HPS].contains(input_voxel_type),
        VoxelType::Pathological => true,
    }
}

#[cfg(test)]
mod tests {

    use crate::vis::plotting::plot_voxel_types;

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

    #[test]
    fn is_connection_allowed_true() {
        let output_voxel_type = VoxelType::HPS;
        let input_voxel_type = VoxelType::Ventricle;

        let allowed = is_connection_allowed(&output_voxel_type, &input_voxel_type);

        assert_eq!(allowed, true);
    }

    #[test]
    fn is_connection_allowed_false() {
        let output_voxel_type = VoxelType::Atrium;
        let input_voxel_type = VoxelType::Ventricle;

        let allowed = is_connection_allowed(&output_voxel_type, &input_voxel_type);

        assert_eq!(allowed, false);
    }

    #[test]
    fn some_voxel_types_default() {
        let config = Model::default();
        let types = VoxelTypes::from_simulation_config(&config);

        let num_sa = types
            .values
            .iter()
            .filter(|v_type| **v_type == VoxelType::Sinoatrial)
            .count();

        assert_eq!(num_sa, 1);

        let num_atrium = types
            .values
            .iter()
            .filter(|v_type| **v_type == VoxelType::Atrium)
            .count();

        assert!(num_atrium > 0);

        let num_avn = types
            .values
            .iter()
            .filter(|v_type| **v_type == VoxelType::Atrioventricular)
            .count();

        assert_eq!(num_avn, 1);

        let num_ventricle = types
            .values
            .iter()
            .filter(|v_type| **v_type == VoxelType::Ventricle)
            .count();

        assert!(num_ventricle > 0);

        let num_hps = types
            .values
            .iter()
            .filter(|v_type| **v_type == VoxelType::HPS)
            .count();

        assert!(num_hps > 0);

        let num_pathological = types
            .values
            .iter()
            .filter(|v_type| **v_type == VoxelType::Pathological)
            .count();

        assert_eq!(num_pathological, 0);

        plot_voxel_types(&types.values, "tests/voxel_types_default", "Voxel Types")
    }

    #[test]
    fn some_voxel_types_pathological() {
        let mut config = Model::default();
        config.pathological = true;
        let types = VoxelTypes::from_simulation_config(&config);

        let num_sa = types
            .values
            .iter()
            .filter(|v_type| **v_type == VoxelType::Sinoatrial)
            .count();

        assert_eq!(num_sa, 1);

        let num_atrium = types
            .values
            .iter()
            .filter(|v_type| **v_type == VoxelType::Atrium)
            .count();

        assert!(num_atrium > 0);

        let num_avn = types
            .values
            .iter()
            .filter(|v_type| **v_type == VoxelType::Atrioventricular)
            .count();

        assert_eq!(num_avn, 1);

        let num_ventricle = types
            .values
            .iter()
            .filter(|v_type| **v_type == VoxelType::Ventricle)
            .count();

        assert!(num_ventricle > 0);

        let num_hps = types
            .values
            .iter()
            .filter(|v_type| **v_type == VoxelType::HPS)
            .count();

        assert!(num_hps > 0);

        let num_pathological = types
            .values
            .iter()
            .filter(|v_type| **v_type == VoxelType::Pathological)
            .count();

        assert!(num_pathological > 0);

        plot_voxel_types(
            &types.values,
            "tests/voxel_types_pathological",
            "Voxel Types",
        )
    }
}
