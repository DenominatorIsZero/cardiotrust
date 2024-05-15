use ndarray::{arr1, s, Array3, Array4, Dim};
use ndarray_npy::WriteNpyExt;

use num_derive::FromPrimitive;
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::BufWriter,
    ops::{Deref, DerefMut},
};
use strum_macros::{EnumCount, EnumIter};
use tracing::{debug, trace};

use crate::core::{config::model::Model, model::spatial::nifti::load_from_nii};

use super::nifti::{determine_voxel_type, MriData};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Voxels {
    pub size_mm: f32,
    pub types: VoxelTypes,
    pub numbers: VoxelNumbers,
    pub positions_mm: VoxelPositions,
}

impl Voxels {
    /// Creates an empty Voxels struct with the given dimensions.
    #[must_use]
    #[tracing::instrument(level = "debug")]
    pub fn empty(voxels_in_dims: [usize; 3]) -> Self {
        debug!("Creating empty voxels");
        Self {
            size_mm: 0.0,
            types: VoxelTypes::empty(voxels_in_dims),
            numbers: VoxelNumbers::empty(voxels_in_dims),
            positions_mm: VoxelPositions::empty(voxels_in_dims),
        }
    }

    /// Creates a Voxels struct from the given Model config.
    #[must_use]
    #[tracing::instrument(level = "debug")]
    pub fn from_handcrafted_model_config(config: &Model) -> Self {
        debug!("Creating voxels from handcrafted model config");
        let types = VoxelTypes::from_handcrafted_model_config(config);
        let numbers = VoxelNumbers::from_voxel_types(&types);
        let positions = VoxelPositions::from_handcrafted_model_config(config, types.raw_dim());
        Self {
            size_mm: config.common.voxel_size_mm,
            types,
            numbers,
            positions_mm: positions,
        }
    }

    #[must_use]
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn from_mri_model_config(config: &Model) -> Self {
        debug!("Creating voxels from mri model config");

        let mri_data = load_from_nii(&config.mri.as_ref().unwrap().path);

        let positions = VoxelPositions::from_mri_model_config(config, &mri_data);
        let types = VoxelTypes::from_mri_model_config(config, &positions, &mri_data);
        let numbers = VoxelNumbers::from_voxel_types(&types);
        Self {
            size_mm: config.common.voxel_size_mm,
            types,
            numbers,
            positions_mm: positions,
        }
    }

    /// Returns the total number of voxels.
    ///
    /// This is calculated as the product of the x, y, and z dimensions.
    #[must_use]
    #[tracing::instrument(level = "trace")]
    pub fn count(&self) -> usize {
        trace!("Counting voxels");
        self.count_xyz().iter().product()
    }

    /// Returns the x, y, and z dimensions of the voxels as a 3-element array.
    /// This represents the shape of the voxel grid.
    #[must_use]
    #[tracing::instrument(level = "trace")]
    pub fn count_xyz(&self) -> [usize; 3] {
        trace!("Counting voxels in xyz");
        let shape = self.types.raw_dim();
        [shape[0], shape[1], shape[2]]
    }

    /// Counts the total number of states by iterating through the
    /// voxel types, filtering out voxels of type 'None', and multiplying by 3
    /// (since each voxel has an x, y, and z state).
    #[must_use]
    #[tracing::instrument(level = "trace")]
    pub fn count_states(&self) -> usize {
        trace!("Counting states");
        self.types
            .iter()
            .filter(|voxel| voxel.is_connectable())
            .count()
            * 3
    }

    /// Checks if the given voxel index is within the valid bounds of the voxel grid
    /// and that the voxel type at that index is not `VoxelType::None`.
    ///
    /// Returns `true` if the index is valid, `false` otherwise.
    ///
    /// # Panics
    ///
    /// Panics if number of voxels in any direction
    /// exceed `i32::MAX`.
    #[must_use]
    #[tracing::instrument(level = "trace")]
    pub fn is_valid_index(&self, index: [i32; 3]) -> bool {
        trace!("Checking if index is valid");
        let [x, y, z] = index;
        let [x_max, y_max, z_max] = self.count_xyz();
        (0 <= x && x < (i32::try_from(x_max).unwrap()))
            && (0 <= y && y < (i32::try_from(y_max).unwrap()))
            && (0 <= z && z < (i32::try_from(z_max).unwrap()))
            && self.types[(
                usize::try_from(x).unwrap(),
                usize::try_from(y).unwrap(),
                usize::try_from(z).unwrap(),
            )]
                .is_connectable()
    }

    /// Returns the index of the first voxel of type `v_type`.
    ///
    /// # Panics
    ///
    /// Panics if no voxel of `v_type` is present in `Voxels`.
    #[must_use]
    #[tracing::instrument(level = "trace")]
    pub fn get_first_state_of_type(&self, v_type: VoxelType) -> usize {
        trace!("Getting first state of type {:?}", v_type);
        let query = self
            .types
            .iter()
            .zip(self.numbers.iter())
            .find(|(this_type, _)| **this_type == v_type);
        query.unwrap().1.unwrap()
    }

    /// Saves the voxel grid data to .npy files in the given path.
    #[tracing::instrument(level = "trace")]
    pub(crate) fn save_npy(&self, path: &std::path::Path) {
        trace!("Saving voxels to npy files");
        fs::create_dir_all(path).unwrap();
        let writer = BufWriter::new(File::create(path.join("voxel_size_mm.npy")).unwrap());
        arr1(&[self.size_mm]).write_npy(writer).unwrap();
        self.types.save_npy(path);
        self.numbers.save_npy(path);
        self.positions_mm.save_npy(path);
    }
}

#[allow(clippy::unsafe_derive_deserialize)]
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct VoxelTypes(Array3<VoxelType>);

impl VoxelTypes {
    /// Creates an empty `VoxelTypes` with the given dimensions.
    #[must_use]
    #[tracing::instrument(level = "trace")]
    pub fn empty(voxels_in_dims: [usize; 3]) -> Self {
        trace!("Creating empty voxel types");
        Self(Array3::default(voxels_in_dims))
    }

    /// Creates a `VoxelTypes` struct initialized with voxel types according
    /// to the provided Model configuration. Voxel types are assigned based
    /// on the Model's parameters that define different anatomical regions.
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_precision_loss,
        clippy::similar_names
    )]
    #[must_use]
    #[tracing::instrument(level = "trace")]
    pub fn from_handcrafted_model_config(config: &Model) -> Self {
        trace!("Creating voxel types from simulation config");
        let handcrafted = config.handcrafted.as_ref().unwrap();
        // Config Parameters
        let voxel_size_mm = config.common.voxel_size_mm;
        let heart_size_mm = handcrafted.heart_size_mm;

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
        let sa_x_center_index =
            (voxels_in_dims[0] as f32 * handcrafted.sa_x_center_percentage) as usize;
        let sa_y_center_index =
            (voxels_in_dims[1] as f32 * handcrafted.sa_y_center_percentage) as usize;
        let atrium_y_start_index =
            (voxels_in_dims[1] as f32 * handcrafted.atrium_y_start_percentage) as usize;
        let av_x_center_index =
            (voxels_in_dims[0] as f32 * handcrafted.av_x_center_percentage) as usize;
        let hps_y_stop_index =
            (voxels_in_dims[1] as f32 * handcrafted.hps_y_stop_percentage) as usize;
        let hps_x_start_index =
            (voxels_in_dims[0] as f32 * handcrafted.hps_x_start_percentage) as usize;
        let hps_x_stop_index =
            (voxels_in_dims[0] as f32 * handcrafted.hps_x_stop_percentage) as usize;
        let hps_y_up_index = (voxels_in_dims[1] as f32 * handcrafted.hps_y_up_percentage) as usize;
        let pathology_x_start_index =
            (voxels_in_dims[0] as f32 * handcrafted.pathology_x_start_percentage) as usize;
        let pathology_x_stop_index =
            (voxels_in_dims[0] as f32 * handcrafted.pathology_x_stop_percentage) as usize;
        let pathology_y_start_index =
            (voxels_in_dims[1] as f32 * handcrafted.pathology_y_start_percentage) as usize;
        let pathology_y_stop_index =
            (voxels_in_dims[1] as f32 * handcrafted.pathology_y_stop_percentage) as usize;

        let mut voxel_types = Self::empty(voxels_in_dims);
        voxel_types
            .indexed_iter_mut()
            .for_each(|((x, y, _z), voxel_type)| {
                if (config.common.pathological)
                    && (x >= pathology_x_start_index && x <= pathology_x_stop_index)
                    && (pathology_y_start_index <= y && y <= pathology_y_stop_index)
                {
                    *voxel_type = VoxelType::Pathological;
                } else if (x == sa_x_center_index) && (y == sa_y_center_index) {
                    *voxel_type = VoxelType::Sinoatrial;
                } else if x == av_x_center_index && y == atrium_y_start_index {
                    *voxel_type = VoxelType::Atrioventricular;
                } else if (x == av_x_center_index
                    && y < atrium_y_start_index
                    && y >= hps_y_stop_index)
                    || (x >= hps_x_start_index && x <= hps_x_stop_index && y == hps_y_stop_index)
                    || ((x == hps_x_start_index || x == hps_x_stop_index)
                        && y < hps_y_up_index
                        && y >= hps_y_stop_index)
                {
                    *voxel_type = VoxelType::HPS;
                } else if y > atrium_y_start_index {
                    *voxel_type = VoxelType::Atrium;
                } else {
                    *voxel_type = VoxelType::Ventricle;
                }
            });
        voxel_types
    }

    #[tracing::instrument(level = "trace")]
    fn save_npy(&self, path: &std::path::Path) {
        trace!("Saving voxel types to npy files");
        let writer = BufWriter::new(File::create(path.join("voxel_types.npy")).unwrap());
        self.map(|v| *v as u32).write_npy(writer).unwrap();
    }

    #[must_use]
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn from_mri_model_config(
        config: &Model,
        positions: &VoxelPositions,
        mri_data: &MriData,
    ) -> Self {
        let mut voxel_types = Self::empty([
            positions.raw_dim()[0],
            positions.raw_dim()[1],
            positions.raw_dim()[2],
        ]);

        let mut sinoatrial_placed = false;

        voxel_types
            .indexed_iter_mut()
            .for_each(|(index, voxel_type)| {
                let (x, y, z) = index;
                let position = positions.slice(s![x, y, z, ..]);

                *voxel_type = determine_voxel_type(config, position, mri_data, sinoatrial_placed);
                if *voxel_type == VoxelType::Sinoatrial {
                    sinoatrial_placed = true;
                }
            });

        voxel_types
    }
}

impl Deref for VoxelTypes {
    type Target = Array3<VoxelType>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for VoxelTypes {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Wrapper around a 3d array that contains the state-indices
/// of each voxel.
///
/// If the value is none it means that there is no voxel in this position.
/// In this case, the voxel type at this position is also none.
///
/// Otherwise it is the first component of the current density at this
/// position. In other words the component in the x direction.
/// The next value is then the component in the y direction
/// and finally the offset-2 value is the component in the z
/// direction.
///
/// This struct is often used to iterate over the voxel-tpyes.
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct VoxelNumbers(Array3<Option<usize>>);

impl VoxelNumbers {
    /// Creates a new `VoxelNumbers` instance with the given dimensions,
    /// initializing all voxel values to None.
    #[must_use]
    #[tracing::instrument(level = "trace")]
    pub fn empty(voxels_in_dims: [usize; 3]) -> Self {
        trace!("Creating empty voxel numbers");
        Self(Array3::default(voxels_in_dims))
    }

    /// Creates a new `VoxelNumbers` instance from the given `VoxelTypes`.
    /// initializing the voxel number values based on the voxel types.
    /// Voxels with type `None` will have their number set to `None`.
    /// Other voxels will have their number set to a incrementing integer,
    /// starting from 0 and incrementing by 3 for each voxel.
    #[must_use]
    #[tracing::instrument(level = "trace", skip_all)]
    pub fn from_voxel_types(types: &VoxelTypes) -> Self {
        trace!("Creating voxel numbers from voxel types");
        let mut numbers = Self(Array3::default(types.raw_dim()));

        let mut current_number = 0;
        numbers
            .iter_mut()
            .zip(types.iter())
            .for_each(|(number, voxel_type)| {
                if voxel_type.is_connectable() {
                    *number = Some(current_number);
                    current_number += 3;
                } else {
                    *number = None;
                }
            });
        numbers
    }

    /// Saves the voxel numbers to a .npy file at the given path.
    /// The voxel numbers are converted to i32, with -1 representing None.
    /// Uses numpy's .npy format for efficient storage and loading.
    #[tracing::instrument(level = "trace")]
    fn save_npy(&self, path: &std::path::Path) {
        trace!("Saving voxel numbers to npy files");
        let writer = BufWriter::new(File::create(path.join("voxel_numbers.npy")).unwrap());
        self.map(|v| {
            v.as_ref()
                .map_or(-1, |number| i32::try_from(*number).unwrap())
        })
        .write_npy(writer)
        .unwrap();
    }
}

impl Deref for VoxelNumbers {
    type Target = Array3<Option<usize>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for VoxelNumbers {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[allow(clippy::unsafe_derive_deserialize)]
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct VoxelPositions(Array4<f32>);

impl VoxelPositions {
    /// Creates a new empty `VoxelPositions` instance with the given dimensions.
    /// Initializes the position values to all zeros.
    #[must_use]
    #[tracing::instrument(level = "trace")]
    pub fn empty(voxels_in_dims: [usize; 3]) -> Self {
        trace!("Creating empty voxel positions");
        Self(Array4::zeros((
            voxels_in_dims[0],
            voxels_in_dims[1],
            voxels_in_dims[2],
            3,
        )))
    }

    /// Creates a new `VoxelPositions` instance from the given `Model` config
    /// and `VoxelTypes`. Initializes the position values based on the voxel
    /// size and dimensions specified in the `Model`.
    #[must_use]
    #[tracing::instrument(level = "trace")]
    pub fn from_handcrafted_model_config(config: &Model, shape: Dim<[usize; 3]>) -> Self {
        trace!("Creating voxel positions from handcrafted model config");
        let mut positions = Self::empty([shape[0], shape[1], shape[2]]);
        let offset = config.common.voxel_size_mm / 2.0;

        #[allow(clippy::cast_precision_loss)]
        for x in 0..shape[0] {
            for y in 0..shape[1] {
                for z in 0..shape[2] {
                    let position = arr1(&[
                        config
                            .common
                            .voxel_size_mm
                            .mul_add(x as f32, offset + config.common.heart_offset_mm[0]),
                        config
                            .common
                            .voxel_size_mm
                            .mul_add(y as f32, offset + config.common.heart_offset_mm[1]),
                        config
                            .common
                            .voxel_size_mm
                            .mul_add(z as f32, offset + config.common.heart_offset_mm[2]),
                    ]);
                    positions.slice_mut(s![x, y, z, ..]).assign(&position);
                }
            }
        }
        positions
    }

    #[must_use]
    #[allow(
        clippy::cast_precision_loss,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss
    )]
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn from_mri_model_config(config: &Model, mri_data: &MriData) -> Self {
        trace!("Creating voxel positions from mri model config");

        let mut min_heart_x = mri_data.segmentation.shape()[0];
        let mut max_heart_x = 0;
        let mut min_heart_y = mri_data.segmentation.shape()[1];
        let mut max_heart_y = 1;
        let mut min_heart_z = mri_data.segmentation.shape()[2];
        let mut max_heart_z = 2;

        for x in 0..mri_data.segmentation.shape()[0] {
            for y in 0..mri_data.segmentation.shape()[1] {
                for z in 0..mri_data.segmentation.shape()[2] {
                    if (VoxelType::from_mri_data(mri_data.segmentation[[x, y, z]] as usize))
                        .is_connectable()
                    {
                        min_heart_x = min_heart_x.min(x);
                        max_heart_x = max_heart_x.max(x);
                        min_heart_y = min_heart_y.min(y);
                        max_heart_y = max_heart_y.max(y);
                        min_heart_z = min_heart_z.min(z);
                        max_heart_z = max_heart_z.max(z);
                    }
                }
            }
        }

        let range_heart_x = max_heart_x - min_heart_x;
        let range_heart_y = max_heart_y - min_heart_y;
        let range_heart_z = max_heart_z - min_heart_z;

        let size_mm = [
            range_heart_x as f32 * mri_data.voxel_size_mm[0],
            range_heart_y as f32 * mri_data.voxel_size_mm[1],
            range_heart_z as f32 * mri_data.voxel_size_mm[2],
        ];
        let num_voxels = [
            (size_mm[0] / config.common.voxel_size_mm) as usize,
            (size_mm[1] / config.common.voxel_size_mm) as usize,
            (size_mm[2] / config.common.voxel_size_mm) as usize,
        ];

        let mut positions = Self::empty(num_voxels);
        let offset = config.common.voxel_size_mm / 2.0;
        let offset = [
            (min_heart_x as f32).mul_add(
                mri_data.voxel_size_mm[0],
                offset + config.common.heart_offset_mm[0],
            ),
            (min_heart_y as f32).mul_add(
                mri_data.voxel_size_mm[1],
                offset + config.common.heart_offset_mm[1],
            ),
            (min_heart_z as f32).mul_add(
                mri_data.voxel_size_mm[2],
                offset + config.common.heart_offset_mm[2],
            ),
        ];

        for x in 0..num_voxels[0] {
            for y in 0..num_voxels[1] {
                for z in 0..num_voxels[2] {
                    let position = arr1(&[
                        config.common.voxel_size_mm.mul_add(x as f32, offset[0]),
                        config.common.voxel_size_mm.mul_add(y as f32, offset[1]),
                        config.common.voxel_size_mm.mul_add(z as f32, offset[2]),
                    ]);
                    positions.slice_mut(s![x, y, z, ..]).assign(&position);
                }
            }
        }
        positions
    }

    /// Saves the voxel position values to a .npy file at the given path.
    /// The position values are saved as a 4D float32 array with shape
    /// (x, y, z, 3), where the last dimension contains the x, y, z
    /// coordinates for each voxel position.
    #[tracing::instrument(level = "trace")]
    fn save_npy(&self, path: &std::path::Path) {
        trace!("Saving voxel positions to npy files");
        let writer = BufWriter::new(File::create(path.join("voxel_positions_mm.npy")).unwrap());
        self.write_npy(writer).unwrap();
    }
}

impl Deref for VoxelPositions {
    type Target = Array4<f32>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for VoxelPositions {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(
    Default,
    Debug,
    PartialEq,
    Eq,
    Hash,
    Deserialize,
    Serialize,
    Copy,
    Clone,
    EnumIter,
    EnumCount,
    FromPrimitive,
)]
pub enum VoxelType {
    #[default]
    None,
    Sinoatrial,
    Atrium,
    Atrioventricular,
    HPS,
    Ventricle,
    Pathological,
    Vessel,
    Torso,
    Chamber,
}

impl VoxelType {
    pub(crate) const fn from_mri_data(value: usize) -> Self {
        match value {
            1 => Self::Atrium,
            2 => Self::Vessel,
            3 => Self::Torso,
            5 => Self::Chamber,
            6 => Self::Sinoatrial,
            _ => Self::None,
        }
    }

    pub(crate) const fn is_connectable(self) -> bool {
        matches!(
            self,
            Self::Sinoatrial
                | Self::Atrium
                | Self::Atrioventricular
                | Self::HPS
                | Self::Ventricle
                | Self::Pathological
        )
    }
}

/// Checks if a connection between the given input and output voxel types is allowed
/// based on anatomical constraints. Returns true if allowed, false otherwise.
#[must_use]
#[tracing::instrument(level = "trace")]
pub fn is_connection_allowed(output_voxel_type: &VoxelType, input_voxel_type: &VoxelType) -> bool {
    trace!("Checking if connection is allowed");
    match output_voxel_type {
        VoxelType::None | VoxelType::Vessel | VoxelType::Torso | VoxelType::Chamber => false,
        VoxelType::Sinoatrial => {
            [VoxelType::Atrium, VoxelType::Pathological].contains(input_voxel_type)
        }
        VoxelType::Atrium => [
            VoxelType::Sinoatrial,
            VoxelType::Atrium,
            VoxelType::Atrioventricular,
            VoxelType::Pathological,
        ]
        .contains(input_voxel_type),
        VoxelType::Atrioventricular => {
            [VoxelType::Atrium, VoxelType::HPS, VoxelType::Pathological].contains(input_voxel_type)
        }
        VoxelType::HPS => [
            VoxelType::HPS,
            VoxelType::Atrioventricular,
            VoxelType::Ventricle,
            VoxelType::Pathological,
        ]
        .contains(input_voxel_type),
        VoxelType::Ventricle => [
            VoxelType::Ventricle,
            VoxelType::HPS,
            VoxelType::Pathological,
        ]
        .contains(input_voxel_type),
        VoxelType::Pathological => true,
    }
}

#[cfg(test)]
mod tests {

    use crate::core::config::model::{Common, Handcrafted};

    use super::*;

    const _COMMON_PATH: &str = "tests/core/model/spatial/voxel/";

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
        voxels.types[(0, 0, 0)] = VoxelType::Atrioventricular;

        assert_eq!(3, voxels.count_states());
    }

    #[test]
    fn no_pathology_full_states() {
        let config = Model {
            handcrafted: Some(Handcrafted {
                heart_size_mm: [10.0, 10.0, 10.0],
                ..Default::default()
            }),
            common: Common {
                voxel_size_mm: 1.0,
                ..Default::default()
            },
            ..Default::default()
        };
        let voxels = Voxels::from_handcrafted_model_config(&config);

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
        let types = VoxelTypes::from_handcrafted_model_config(&config);

        let num_sa = types
            .iter()
            .filter(|v_type| **v_type == VoxelType::Sinoatrial)
            .count();

        assert_eq!(num_sa, 1);

        let num_atrium = types
            .iter()
            .filter(|v_type| **v_type == VoxelType::Atrium)
            .count();

        assert!(num_atrium > 0);

        let num_avn = types
            .iter()
            .filter(|v_type| **v_type == VoxelType::Atrioventricular)
            .count();

        assert_eq!(num_avn, 1);

        let num_ventricle = types
            .iter()
            .filter(|v_type| **v_type == VoxelType::Ventricle)
            .count();

        assert!(num_ventricle > 0);

        let num_hps = types
            .iter()
            .filter(|v_type| **v_type == VoxelType::HPS)
            .count();

        assert!(num_hps > 0);

        let num_pathological = types
            .iter()
            .filter(|v_type| **v_type == VoxelType::Pathological)
            .count();

        assert_eq!(num_pathological, 0);
    }
}
