use std::{
    fs::{self, File},
    io::BufWriter,
    ops::{Deref, DerefMut},
};

use anyhow::{Context, Result};

use approx::assert_relative_eq;
use ndarray::{Array2, Array3, Dim};
use ndarray_npy::WriteNpyExt;
use ocl::Buffer;
use serde::{Deserialize, Serialize};
use tracing::{debug, trace};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ActivationTimeMs {
    pub values: Array3<Option<f32>>,
}

impl ActivationTimeMs {
    #[must_use]
    /// Creates a new `ArrayActivationTime` with the given voxel dimensions,
    /// initializing all values to `None`.
    #[tracing::instrument(level = "debug")]
    pub fn empty(voxels_in_dims: Dim<[usize; 3]>) -> Self {
        debug!("Creating empty activation time array");
        Self {
            values: Array3::from_elem(voxels_in_dims, None),
        }
    }

    /// Saves the activation time values to a .npy file at the given path.
    /// The values are mapped from Option<f32> to f32, replacing None with -1.0.
    /// The file contains a single 3D array with the activation time values.
    ///
    /// # Errors
    ///
    /// Returns an error if the directory cannot be created or the file cannot be written.
    #[tracing::instrument(level = "trace")]
    pub(crate) fn save_npy(&self, path: &std::path::Path) -> Result<()> {
        trace!("Saving activation time to npy");
        fs::create_dir_all(path)
            .with_context(|| format!("Failed to create directory for activation time: {}", path.display()))?;

        let file_path = path.join("activation_time.npy");
        let writer = BufWriter::new(File::create(&file_path)
            .with_context(|| format!("Failed to create activation time file: {}", file_path.display()))?);

        self.map(|v| v.as_ref().map_or_else(|| -1.0, |index| *index))
            .write_npy(writer)
            .with_context(|| format!("Failed to write activation time to: {}", file_path.display()))?;

        Ok(())
    }
}

impl Deref for ActivationTimeMs {
    type Target = Array3<Option<f32>>;

    #[tracing::instrument(level = "trace")]
    fn deref(&self) -> &Self::Target {
        &self.values
    }
}

impl DerefMut for ActivationTimeMs {
    #[tracing::instrument(level = "trace")]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.values
    }
}

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Gains(Array2<f32>);

impl Gains {
    /// Creates a new `ArrayGains` with the given number of states,
    /// initializing all values to zeros.
    #[must_use]
    #[tracing::instrument(level = "trace")]
    pub fn empty(number_of_states: usize) -> Self {
        trace!("Creating empty gains array");
        Self(Array2::zeros((number_of_states, 78)))
    }

    /// Saves the array values to a .npy file at the given path with the given name.
    /// The values are written directly to the file using `write_npy`.
    ///
    /// # Errors
    ///
    /// Returns an error if the directory cannot be created or the file cannot be written.
    #[allow(dead_code)]
    #[tracing::instrument(level = "trace")]
    pub(crate) fn save_npy(&self, path: &std::path::Path, name: &str) -> Result<()> {
        trace!("Saving gains to npy");
        fs::create_dir_all(path)
            .with_context(|| format!("Failed to create directory for gains: {}", path.display()))?;

        let file_path = path.join(name);
        let writer = BufWriter::new(File::create(&file_path)
            .with_context(|| format!("Failed to create gains file: {}", file_path.display()))?);

        self.write_npy(writer)
            .with_context(|| format!("Failed to write gains to: {}", file_path.display()))?;

        Ok(())
    }

    #[tracing::instrument(level = "trace", skip_all)]
    pub(crate) fn to_gpu(&self, queue: &ocl::Queue) -> Result<ocl::Buffer<f32>> {
        let buffer = Buffer::builder()
            .queue(queue.clone())
            .len(self.len())
            .copy_host_slice(
                self.as_slice()
                    .context("Failed to get array slice for GPU copy")?,
            )
            .build()
            .context("Failed to build GPU buffer for gains")?;
        Ok(buffer)
    }

    #[tracing::instrument(level = "trace", skip_all)]
    pub(crate) fn update_from_gpu(&mut self, buffer: &Buffer<f32>) -> Result<()> {
        buffer
            .read(
                self.as_slice_mut()
                    .context("Failed to get mutable array slice for GPU read")?,
            )
            .enq()
            .context("Failed to read gains from GPU buffer")?;
        Ok(())
    }
}

impl Deref for Gains {
    type Target = Array2<f32>;

    #[tracing::instrument(level = "trace")]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Gains {
    #[tracing::instrument(level = "trace")]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Indices(Array2<Option<usize>>);

impl Indices {
    /// Creates a new `ArrayIndicesGains` with the given number of states,
    /// initializing all values to `None`.
    #[must_use]
    #[tracing::instrument(level = "trace")]
    pub fn empty(number_of_states: usize) -> Self {
        trace!("Creating empty indices gains array");
        Self(Array2::from_elem((number_of_states, 78), None))
    }

    /// Saves the array indices values to a .npy file at the given path.
    /// The indices are converted to i32 and any None values are converted to -1.
    /// The values are then written directly to the file using `write_npy`.
    ///
    /// # Errors
    ///
    /// Returns an error if the directory cannot be created or the file cannot be written.
    #[tracing::instrument(level = "trace")]
    pub fn save_npy(&self, path: &std::path::Path) -> Result<()> {
        trace!("Saving indices gains to npy");
        fs::create_dir_all(path)
            .with_context(|| format!("Failed to create directory for indices: {}", path.display()))?;

        let file_path = path.join("output_state_indices.npy");
        let writer = BufWriter::new(File::create(&file_path)
            .with_context(|| format!("Failed to create indices file: {}", file_path.display()))?);

        let converted_indices = self.map(|v| {
            v.as_ref().map_or(-1, |index| {
                i32::try_from(*index)
                    .unwrap_or_else(|_| {
                        tracing::warn!("Index {} exceeds i32::MAX, using -1", index);
                        -1
                    })
            })
        });

        converted_indices.write_npy(writer)
            .with_context(|| format!("Failed to write indices to: {}", file_path.display()))?;

        Ok(())
    }
}

impl Deref for Indices {
    type Target = Array2<Option<usize>>;

    #[tracing::instrument(level = "trace")]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Indices {
    #[tracing::instrument(level = "trace")]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Coefs(Array2<f32>);

impl Coefs {
    /// Creates a new `ArrayDelays` with the given number of states,
    /// initializing all values to 0. The number of states must be divisible by 3.
    ///
    /// # Panics
    ///
    /// Panics if `number_of_states` is not divisible by 3.
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    #[tracing::instrument(level = "trace")]
    pub fn empty(number_of_states: usize) -> Self {
        trace!("Creating empty delays array");
        assert_relative_eq!(number_of_states as f32 % 3.0, 0.0);
        Self(Array2::zeros((number_of_states / 3, 26)))
    }

    /// Saves the values in this `ArrayDelays` to a .npy file at the given path.
    ///
    /// The .npy file will be named `coefs.npy`.
    ///
    /// # Errors
    ///
    /// Returns an error if the directory cannot be created or the file cannot be written.
    #[tracing::instrument(level = "trace")]
    pub fn save_npy(&self, path: &std::path::Path) -> Result<()> {
        trace!("Saving delays to npy");
        fs::create_dir_all(path)
            .with_context(|| format!("Failed to create directory for coefs: {}", path.display()))?;

        let file_path = path.join("coefs.npy");
        let writer = BufWriter::new(File::create(&file_path)
            .with_context(|| format!("Failed to create coefs file: {}", file_path.display()))?);

        self.write_npy(writer)
            .with_context(|| format!("Failed to write coefs to: {}", file_path.display()))?;

        Ok(())
    }

    #[tracing::instrument(level = "trace", skip_all)]
    pub(crate) fn to_gpu(&self, queue: &ocl::Queue) -> Result<Buffer<f32>> {
        let buffer = Buffer::builder()
            .queue(queue.clone())
            .len(self.len())
            .copy_host_slice(
                self.as_slice()
                    .context("Failed to get array slice for GPU copy")?,
            )
            .build()
            .context("Failed to build GPU buffer for coefs")?;
        Ok(buffer)
    }

    #[tracing::instrument(level = "trace", skip_all)]
    pub(crate) fn update_from_gpu(&mut self, coefs: &Buffer<f32>) -> Result<()> {
        coefs
            .read(
                self.as_slice_mut()
                    .context("Failed to get mutable array slice for GPU read")?,
            )
            .enq()
            .context("Failed to read coefs from GPU buffer")?;
        Ok(())
    }
}

impl Deref for Coefs {
    type Target = Array2<f32>;

    #[tracing::instrument(level = "trace")]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Coefs {
    #[tracing::instrument(level = "trace")]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct UnitDelays(Array2<usize>);

impl UnitDelays {
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    #[tracing::instrument(level = "trace")]
    pub fn empty(number_of_states: usize) -> Self {
        trace!("Creating empty delays array");
        assert_relative_eq!(number_of_states as f32 % 3.0, 0.0);
        Self(Array2::zeros((number_of_states / 3, 26)))
    }
    /// Saves the delay line values in this `ArrayDelays` to a .npy file at the given path.
    ///
    /// Casts the `usize` values to `u32` before writing to satisfy `.npy` format limitations.
    ///
    /// # Errors
    ///
    /// Returns an error if the directory cannot be created or the file cannot be written.
    #[tracing::instrument(level = "trace")]
    pub fn save_npy(&self, path: &std::path::Path) -> Result<()> {
        trace!("Saving delays to npy");
        fs::create_dir_all(path)
            .with_context(|| format!("Failed to create directory for delays: {}", path.display()))?;

        let file_path = path.join("delays.npy");
        let writer = BufWriter::new(File::create(&file_path)
            .with_context(|| format!("Failed to create delays file: {}", file_path.display()))?);

        let converted_delays = self.map(|v| {
            u32::try_from(*v)
                .unwrap_or_else(|_| {
                    tracing::warn!("Delay value {} exceeds u32::MAX, using u32::MAX", v);
                    u32::MAX
                })
        });

        converted_delays.write_npy(writer)
            .with_context(|| format!("Failed to write delays to: {}", file_path.display()))?;

        Ok(())
    }
}

impl Deref for UnitDelays {
    type Target = Array2<usize>;

    #[tracing::instrument(level = "trace")]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for UnitDelays {
    #[tracing::instrument(level = "trace")]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
