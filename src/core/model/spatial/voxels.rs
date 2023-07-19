use ndarray::{arr1, s, Array3, Array4};
use serde::{Deserialize, Serialize};

use crate::core::config::model::Model;

#[derive(Debug, PartialEq, Clone)]
pub struct Voxels {
    pub size_mm: f32,
    pub types: VoxelTypes,
    pub numbers: VoxelNumbers,
    pub positions_mm: VoxelPositions,
}

impl Voxels {
    #[must_use] pub fn empty(voxels_in_dims: [usize; 3]) -> Self {
        Self {
            size_mm: 0.0,
            types: VoxelTypes::empty(voxels_in_dims),
            numbers: VoxelNumbers::empty(voxels_in_dims),
            positions_mm: VoxelPositions::empty(voxels_in_dims),
        }
    }

    #[must_use] pub fn from_model_config(config: &Model) -> Self {
        let types = VoxelTypes::from_simulation_config(config);
        let numbers = VoxelNumbers::from_voxel_types(&types);
        let positions = VoxelPositions::from_model_config(config, &types);
        Self {
            size_mm: config.voxel_size_mm,
            types,
            numbers,
            positions_mm: positions,
        }
    }

    #[must_use] pub fn count(&self) -> usize {
        self.count_xyz().iter().product()
    }

    #[must_use] pub fn count_xyz(&self) -> [usize; 3] {
        let shape = self.types.values.raw_dim();
        [shape[0], shape[1], shape[2]]
    }

    #[must_use] pub fn count_states(&self) -> usize {
        self.types
            .values
            .iter()
            .filter(|voxel| **voxel != VoxelType::None)
            .count()
            * 3
    }

    /// .
    ///
    /// # Panics
    ///
    /// Panics if number of voxels in any direction
    /// exceed `i32::MAX`.
    #[must_use] pub fn is_valid_index(&self, index: [i32; 3]) -> bool {
        let [x, y, z] = index;
        let [x_max, y_max, z_max] = self.count_xyz();
        (0 <= x && x < (i32::try_from(x_max).unwrap()))
            && (0 <= y && y < (i32::try_from(y_max).unwrap()))
            && (0 <= z && z < (i32::try_from(z_max).unwrap()))
            && (self.types.values[(usize::try_from(x).unwrap(), usize::try_from(y).unwrap(), usize::try_from(z).unwrap())] != VoxelType::None)
    }

    /// .
    ///
    /// # Panics
    ///
    /// Panics if no voxel of `v_type` is present in `Voxels`.
    #[must_use] pub fn get_first_state_of_type(&self, v_type: VoxelType) -> usize {
        let query = self
            .types
            .values
            .iter()
            .zip(self.numbers.values.iter())
            .find(|(this_type, _)| **this_type == v_type);
        query.unwrap().1.unwrap()
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct VoxelTypes {
    pub values: Array3<VoxelType>,
}

impl VoxelTypes {
    #[must_use] pub fn empty(voxels_in_dims: [usize; 3]) -> Self {
        Self {
            values: Array3::default(voxels_in_dims),
        }
    }

    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss, clippy::cast_precision_loss, clippy::similar_names)]
    #[must_use] pub fn from_simulation_config(config: &Model) -> Self {
        // Config Parameters
        let voxel_size_mm = config.voxel_size_mm;
        let heart_size_mm = config.heart_size_mm;

        let mut voxels_in_dims = [0, 0, 0];
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        voxels_in_dims
            .iter_mut()
            .zip(heart_size_mm.iter())
            .for_each(|(number, size)| *number = (size / voxel_size_mm) as usize);

        voxels_in_dims
            .iter_mut()
            .for_each(|v| *v = if *v == 0 { 1 } else { *v });

        // Derived Parameters
        let sa_x_center_index = (voxels_in_dims[0] as f32 * config.sa_x_center_percentage) as usize;
        let sa_y_center_index = (voxels_in_dims[1] as f32 * config.sa_y_center_percentage) as usize;
        let atrium_y_stop_index =
            (voxels_in_dims[1] as f32 * config.atrium_y_stop_percentage) as usize;
        let av_x_center_index = (voxels_in_dims[0] as f32 * config.av_x_center_percentage) as usize;
        let hps_y_stop_index = (voxels_in_dims[1] as f32 * config.hps_y_stop_percentage) as usize;
        let hps_x_start_index = (voxels_in_dims[0] as f32 * config.hps_x_start_percentage) as usize;
        let hps_x_stop_index = (voxels_in_dims[0] as f32 * config.hps_x_stop_percentage) as usize;
        let hps_y_up_index = (voxels_in_dims[1] as f32 * config.hps_y_up_percentage) as usize;
        let pathology_x_start_index =
            (voxels_in_dims[0] as f32 * config.pathology_x_start_percentage) as usize;
        let pathology_x_stop_index =
            (voxels_in_dims[0] as f32 * config.pathology_x_stop_percentage) as usize;
        let pathology_y_start_index =
            (voxels_in_dims[1] as f32 * config.pathology_y_start_percentage) as usize;
        let pathology_y_stop_index =
            (voxels_in_dims[1] as f32 * config.pathology_y_stop_percentage) as usize;

        let mut voxel_types = Self::empty(voxels_in_dims);
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
                
                else if 
                // HPS Downward section
                (x == av_x_center_index && y > atrium_y_stop_index && y < hps_y_stop_index) || 
                // HPS Across
                (x >= hps_x_start_index
                    && x <= hps_x_stop_index
                    && y == hps_y_stop_index - 1)||
                // HPS Up
                ((x == hps_x_start_index || x == hps_x_stop_index)
                && y >= hps_y_up_index
                && y < hps_y_stop_index)
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

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct VoxelNumbers {
    pub values: Array3<Option<usize>>,
}

impl VoxelNumbers {
    #[must_use] pub fn empty(voxels_in_dims: [usize; 3]) -> Self {
        Self {
            values: Array3::default(voxels_in_dims),
        }
    }

    #[must_use] pub fn from_voxel_types(types: &VoxelTypes) -> Self {
        let mut numbers = Self {
            values: Array3::default(types.values.raw_dim()),
        };

        let mut current_number = 0;
        numbers
            .values
            .iter_mut()
            .zip(types.values.iter())
            .for_each(|(number, voxel_type)| {
                if *voxel_type == VoxelType::None {
                    *number = None;
                } else {
                    *number = Some(current_number);
                    current_number += 3;
                }
            });
        numbers
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct VoxelPositions {
    pub values: Array4<f32>,
}

impl VoxelPositions {
    #[must_use] pub fn empty(voxels_in_dims: [usize; 3]) -> Self {
        Self {
            values: Array4::zeros((voxels_in_dims[0], voxels_in_dims[1], voxels_in_dims[2], 3)),
        }
    }

    #[must_use] pub fn from_model_config(config: &Model, types: &VoxelTypes) -> Self {
        let shape = types.values.raw_dim();
        let mut positions = Self::empty([shape[0], shape[1], shape[2]]);
        let offset = config.voxel_size_mm / 2.0;

        #[allow(clippy::cast_precision_loss)]
        types.values.indexed_iter().for_each(|((x, y, z), _)| {
            let position = arr1(&[
                config.voxel_size_mm.mul_add(x as f32, offset),
                config.voxel_size_mm.mul_add(y as f32, offset),
                config.voxel_size_mm.mul_add(z as f32, offset),
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

#[must_use] pub fn is_connection_allowed(output_voxel_type: &VoxelType, input_voxel_type: &VoxelType) -> bool {
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

    use crate::vis::plotting::matrix::plot_voxel_types;

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

        assert!(allowed);
    }

    #[test]
    fn is_connection_allowed_false() {
        let output_voxel_type = VoxelType::Atrium;
        let input_voxel_type = VoxelType::Ventricle;

        let allowed = is_connection_allowed(&output_voxel_type, &input_voxel_type);

        assert!(!allowed);
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
    }

    #[test]
    #[ignore]
    fn some_voxel_types_default_and_plot() {
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

        plot_voxel_types(&types.values, "tests/voxel_types_default", "Voxel Types");
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
    }

    #[test]
    #[ignore]
    fn some_voxel_types_pathological_and_plot() {
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
        );
    }
}
