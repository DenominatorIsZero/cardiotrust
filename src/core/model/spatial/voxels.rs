mod numbers;
mod positions;
#[cfg(test)]
mod tests;
mod types;

use std::{
    fs::{self, File},
    io::BufWriter,
};

use anyhow::{Context, Result};
use ndarray::arr1;
#[cfg(feature = "native")]
use ndarray_npy::WriteNpyExt;
use num_derive::FromPrimitive;
pub use numbers::VoxelNumbers;
pub use positions::VoxelPositions;
use serde::{Deserialize, Serialize};
use strum_macros::{EnumCount, EnumIter};
use tracing::{debug, trace};
pub use types::VoxelTypes;

#[cfg(feature = "native")]
use crate::core::{config::model::Model, model::spatial::nifti::load_from_nii};
#[cfg(not(feature = "native"))]
use crate::core::config::model::Model;

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
    #[tracing::instrument(level = "debug")]
    pub fn from_handcrafted_model_config(config: &Model) -> Result<Self> {
        debug!("Creating voxels from handcrafted model config");
        let types = VoxelTypes::from_handcrafted_model_config(config)?;
        let numbers = VoxelNumbers::from_voxel_types(&types);
        let positions = VoxelPositions::from_handcrafted_model_config(config, types.raw_dim());
        Ok(Self {
            size_mm: config.common.voxel_size_mm,
            types,
            numbers,
            positions_mm: positions,
        })
    }

    #[cfg(feature = "native")]
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn from_mri_model_config(config: &Model) -> anyhow::Result<Self> {
        debug!("Creating voxels from mri model config");

        let mri_config = config
            .mri
            .as_ref()
            .context("MRI configuration is required but not provided")?;
        let mri_data = load_from_nii(&mri_config.path)?;

        let positions = VoxelPositions::from_mri_model_config(config, &mri_data);
        let types = VoxelTypes::from_mri_model_config(config, &positions, &mri_data)?;
        let numbers = VoxelNumbers::from_voxel_types(&types);
        Ok(Self {
            size_mm: config.common.voxel_size_mm,
            types,
            numbers,
            positions_mm: positions,
        })
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
    /// Returns `false` if dimensions exceed conversion bounds.
    #[must_use]
    #[tracing::instrument(level = "trace")]
    pub fn is_valid_index(&self, index: [i32; 3]) -> bool {
        trace!("Checking if index is valid");
        let [x, y, z] = index;
        let [x_max, y_max, z_max] = self.count_xyz();

        // Check if dimensions can be safely converted
        let Ok(x_max_i32) = i32::try_from(x_max) else {
            return false;
        };
        let Ok(y_max_i32) = i32::try_from(y_max) else {
            return false;
        };
        let Ok(z_max_i32) = i32::try_from(z_max) else {
            return false;
        };

        // Check bounds
        if !(0 <= x && x < x_max_i32 && 0 <= y && y < y_max_i32 && 0 <= z && z < z_max_i32) {
            return false;
        }

        // Convert back to usize for array indexing
        let Ok(x_usize) = usize::try_from(x) else {
            return false;
        };
        let Ok(y_usize) = usize::try_from(y) else {
            return false;
        };
        let Ok(z_usize) = usize::try_from(z) else {
            return false;
        };

        self.types[(x_usize, y_usize, z_usize)].is_connectable()
    }

    /// Returns the index of the first voxel of type `v_type`.
    ///
    /// # Errors
    ///
    /// Returns an error if no voxel of `v_type` is present in `Voxels` or if
    /// the voxel has no assigned number.
    #[tracing::instrument(level = "trace")]
    pub fn get_first_state_of_type(&self, v_type: VoxelType) -> anyhow::Result<usize> {
        trace!("Getting first state of type {:?}", v_type);
        let query = self
            .types
            .iter()
            .zip(self.numbers.iter())
            .find(|(this_type, _)| **this_type == v_type);

        let (_, number_option) =
            query.with_context(|| format!("No voxel of type {v_type:?} found in voxels"))?;

        number_option.with_context(|| format!("Voxel of type {v_type:?} has no assigned number"))
    }

    /// Saves the voxel grid data to .npy files in the given path.
    #[cfg(feature = "native")]
    #[tracing::instrument(level = "trace")]
    pub(crate) fn save_npy(&self, path: &std::path::Path) -> anyhow::Result<()> {
        trace!("Saving voxels to npy files");
        fs::create_dir_all(path).with_context(|| {
            format!("Failed to create directory for voxels: {}", path.display())
        })?;

        let size_file_path = path.join("voxel_size_mm.npy");
        let writer = BufWriter::new(File::create(&size_file_path).with_context(|| {
            format!(
                "Failed to create voxel size file: {}",
                size_file_path.display()
            )
        })?);
        arr1(&[self.size_mm]).write_npy(writer).with_context(|| {
            format!(
                "Failed to write voxel size to: {}",
                size_file_path.display()
            )
        })?;

        self.types.save_npy(path)?;
        self.numbers.save_npy(path)?;
        self.positions_mm.save_npy(path)?;
        Ok(())
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
        VoxelType::Sinoatrial => [
            VoxelType::Atrium,
            VoxelType::Pathological,
            VoxelType::Ventricle,
        ]
        .contains(input_voxel_type),
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
