#[cfg(feature = "native")]
use std::{
    fs::File,
    io::BufWriter,
};
use std::{
    ops::{Deref, DerefMut},
};

use anyhow::{Context, Result};
#[cfg(feature = "native")]
use ndarray::{s, Array3};
#[cfg(not(feature = "native"))]
use ndarray::Array3;
#[cfg(feature = "native")]
use ndarray_npy::WriteNpyExt;
use serde::{Deserialize, Serialize};
use super::VoxelType;
#[cfg(feature = "native")]
use super::VoxelPositions;
#[cfg(feature = "native")]
use super::super::nifti::{determine_voxel_type, MriData};
use crate::core::config::model::Model;

#[allow(clippy::unsafe_derive_deserialize)]
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct VoxelTypes(Array3<VoxelType>);

impl VoxelTypes {
    /// Creates an empty `VoxelTypes` with the given dimensions.
    #[must_use]
    #[tracing::instrument(level = "trace")]
    pub fn empty(voxels_in_dims: [usize; 3]) -> Self {
        tracing::trace!("Creating empty voxel types");
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
    #[tracing::instrument(level = "trace")]
    pub fn from_handcrafted_model_config(config: &Model) -> Result<Self> {
        tracing::trace!("Creating voxel types from simulation config");
        let handcrafted = config
            .handcrafted
            .as_ref()
            .context("Handcrafted config is required for from_handcrafted_model_config")?;
        // Config Parameters
        let voxel_size_mm = config.common.voxel_size_mm;
        let heart_size_mm = handcrafted.heart_size_mm;

        let mut voxels_in_dims = [0, 0, 0];
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        voxels_in_dims
            .iter_mut()
            .zip(heart_size_mm.iter())
            .for_each(|(number, size)| *number = (size / voxel_size_mm) as usize);

        for v in &mut voxels_in_dims {
            *v = if *v == 0 { 1 } else { *v };
        }

        // Derived Parameters
        let sa_x_center_index = ((voxels_in_dims[0] as f32 * handcrafted.sa_x_center_percentage)
            as usize)
            .min(voxels_in_dims[0] - 1);
        let sa_y_center_index = ((voxels_in_dims[1] as f32 * handcrafted.sa_y_center_percentage)
            as usize)
            .min(voxels_in_dims[1] - 1);
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
                if (x == sa_x_center_index) && (y == sa_y_center_index) {
                    *voxel_type = VoxelType::Sinoatrial;
                } else if (config.common.pathological)
                    && (x >= pathology_x_start_index && x <= pathology_x_stop_index)
                    && (pathology_y_start_index <= y && y <= pathology_y_stop_index)
                {
                    *voxel_type = VoxelType::Pathological;
                } else if x == av_x_center_index
                    && y == atrium_y_start_index
                    && handcrafted.include_av
                {
                    *voxel_type = VoxelType::Atrioventricular;
                } else if ((x == av_x_center_index
                    && y < atrium_y_start_index
                    && y >= hps_y_stop_index)
                    || (x >= hps_x_start_index && x <= hps_x_stop_index && y == hps_y_stop_index)
                    || ((x == hps_x_start_index || x == hps_x_stop_index)
                        && y < hps_y_up_index
                        && y >= hps_y_stop_index))
                    && handcrafted.include_hps
                {
                    *voxel_type = VoxelType::HPS;
                } else if y > atrium_y_start_index && handcrafted.include_atrium {
                    *voxel_type = VoxelType::Atrium;
                } else {
                    *voxel_type = VoxelType::Ventricle;
                }
            });
        Ok(voxel_types)
    }

    #[cfg(feature = "native")]
    #[tracing::instrument(level = "trace")]
    pub(crate) fn save_npy(&self, path: &std::path::Path) -> anyhow::Result<()> {
        tracing::trace!("Saving voxel types to npy files");
        let types_file_path = path.join("voxel_types.npy");
        let writer = BufWriter::new(File::create(&types_file_path).with_context(|| {
            format!(
                "Failed to create voxel types file: {}",
                types_file_path.display()
            )
        })?);
        self.map(|v| *v as u32).write_npy(writer).with_context(|| {
            format!(
                "Failed to write voxel types to: {}",
                types_file_path.display()
            )
        })?;
        Ok(())
    }

    #[cfg(feature = "native")]
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn from_mri_model_config(
        config: &Model,
        positions: &VoxelPositions,
        mri_data: &MriData,
    ) -> anyhow::Result<Self> {
        let mut voxel_types = Self::empty([
            positions.raw_dim()[0],
            positions.raw_dim()[1],
            positions.raw_dim()[2],
        ]);

        let mut sinoatrial_placed = false;

        for (index, voxel_type) in voxel_types.indexed_iter_mut() {
            let (x, y, z) = index;
            let position = positions.slice(s![x, y, z, ..]);

            *voxel_type = determine_voxel_type(config, position, mri_data, sinoatrial_placed)
                .with_context(|| {
                    format!("Failed to determine voxel type at position ({x}, {y}, {z})")
                })?;
            if *voxel_type == VoxelType::Sinoatrial {
                sinoatrial_placed = true;
            }
        }

        Ok(voxel_types)
    }
}

impl Deref for VoxelTypes {
    type Target = Array3<VoxelType>;

    #[tracing::instrument(level = "trace")]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for VoxelTypes {
    #[tracing::instrument(level = "trace")]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
