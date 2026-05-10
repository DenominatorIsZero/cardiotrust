use std::{
    fs::File,
    io::BufWriter,
    ops::{Deref, DerefMut},
};

use anyhow::Context;
use ndarray::Array3;
#[cfg(feature = "native")]
use ndarray_npy::WriteNpyExt;
use serde::{Deserialize, Serialize};

use super::VoxelTypes;

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
        tracing::trace!("Creating empty voxel numbers");
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
        tracing::trace!("Creating voxel numbers from voxel types");
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
    #[cfg(feature = "native")]
    #[tracing::instrument(level = "trace")]
    pub(crate) fn save_npy(&self, path: &std::path::Path) -> anyhow::Result<()> {
        tracing::trace!("Saving voxel numbers to npy files");
        let numbers_file_path = path.join("voxel_numbers.npy");
        let writer = BufWriter::new(File::create(&numbers_file_path).with_context(|| {
            format!(
                "Failed to create voxel numbers file: {}",
                numbers_file_path.display()
            )
        })?);

        let converted_numbers = self.map(|v| {
            v.as_ref().map_or(-1, |number| {
                i32::try_from(*number).unwrap_or_else(|_| {
                    tracing::warn!("Voxel number {} exceeds i32::MAX, using -1", number);
                    -1
                })
            })
        });

        converted_numbers.write_npy(writer).with_context(|| {
            format!(
                "Failed to write voxel numbers to: {}",
                numbers_file_path.display()
            )
        })?;
        Ok(())
    }
}

impl Deref for VoxelNumbers {
    type Target = Array3<Option<usize>>;

    #[tracing::instrument(level = "trace")]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for VoxelNumbers {
    #[tracing::instrument(level = "trace")]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
