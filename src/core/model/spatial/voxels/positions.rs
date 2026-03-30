use std::{
    fs::File,
    io::BufWriter,
    ops::{Deref, DerefMut},
};

use anyhow::Context;
use ndarray::{arr1, s, Array4, Dim};
use ndarray_npy::WriteNpyExt;
use serde::{Deserialize, Serialize};

use super::{super::nifti::MriData, VoxelType};
use crate::core::config::model::Model;

#[allow(clippy::unsafe_derive_deserialize)]
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct VoxelPositions(Array4<f32>);

impl VoxelPositions {
    /// Creates a new empty `VoxelPositions` instance with the given dimensions.
    /// Initializes the position values to all zeros.
    #[must_use]
    #[tracing::instrument(level = "trace")]
    pub fn empty(voxels_in_dims: [usize; 3]) -> Self {
        tracing::trace!("Creating empty voxel positions");
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
        tracing::trace!("Creating voxel positions from handcrafted model config");
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
        tracing::trace!("Creating voxel positions from mri model config");

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
    pub(crate) fn save_npy(&self, path: &std::path::Path) -> anyhow::Result<()> {
        tracing::trace!("Saving voxel positions to npy files");
        let positions_file_path = path.join("voxel_positions_mm.npy");
        let writer = BufWriter::new(File::create(&positions_file_path).with_context(|| {
            format!(
                "Failed to create voxel positions file: {}",
                positions_file_path.display()
            )
        })?);
        self.write_npy(writer).with_context(|| {
            format!(
                "Failed to write voxel positions to: {}",
                positions_file_path.display()
            )
        })?;
        Ok(())
    }
}

impl Deref for VoxelPositions {
    type Target = Array4<f32>;

    #[tracing::instrument(level = "trace")]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for VoxelPositions {
    #[tracing::instrument(level = "trace")]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
