use std::{
    fs::{self, File},
    io::BufWriter,
    ops::{Deref, DerefMut},
};

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
    /// # Panics
    ///
    /// Panics if the values cannot be written to the file.
    #[tracing::instrument(level = "trace")]
    pub(crate) fn save_npy(&self, path: &std::path::Path) {
        trace!("Saving activation time to npy");
        fs::create_dir_all(path).unwrap();
        let writer = BufWriter::new(File::create(path.join("activation_time.npy")).unwrap());
        self.map(|v| v.as_ref().map_or_else(|| -1.0, |index| *index))
            .write_npy(writer)
            .unwrap();
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
    /// # Panics
    ///
    /// Panics if the file cannot be created or written to.
    #[allow(dead_code)]
    #[tracing::instrument(level = "trace")]
    pub(crate) fn save_npy(&self, path: &std::path::Path, name: &str) {
        trace!("Saving gains to npy");
        fs::create_dir_all(path).unwrap();
        let writer = BufWriter::new(File::create(path.join(name)).unwrap());
        self.write_npy(writer).unwrap();
    }

    #[tracing::instrument(level = "trace", skip_all)]
    pub(crate) fn to_gpu(&self, queue: &ocl::Queue) -> ocl::Buffer<f32> {
        Buffer::builder()
            .queue(queue.clone())
            .len(self.len())
            .copy_host_slice(self.as_slice().unwrap())
            .build()
            .unwrap()
    }

    #[tracing::instrument(level = "trace", skip_all)]
    pub(crate) fn update_from_gpu(&mut self, buffer: &Buffer<f32>) {
        buffer.read(self.as_slice_mut().unwrap()).enq().unwrap();
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
    /// # Panics
    ///
    /// Panics if the file cannot be created or written to.
    #[tracing::instrument(level = "trace")]
    pub fn save_npy(&self, path: &std::path::Path) {
        trace!("Saving indices gains to npy");
        fs::create_dir_all(path).unwrap();
        let writer = BufWriter::new(File::create(path.join("output_state_indices.npy")).unwrap());
        self.map(|v| {
            v.as_ref()
                .map_or(-1, |index| i32::try_from(*index).unwrap())
        })
        .write_npy(writer)
        .unwrap();
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
    /// # Panics
    ///
    /// Panics if the file cannot be created or written to.
    #[tracing::instrument(level = "trace")]
    pub fn save_npy(&self, path: &std::path::Path) {
        trace!("Saving delays to npy");
        fs::create_dir_all(path).unwrap();
        let writer = BufWriter::new(File::create(path.join("coefs.npy")).unwrap());
        self.write_npy(writer).unwrap();
    }

    #[tracing::instrument(level = "trace", skip_all)]
    pub(crate) fn to_gpu(&self, queue: &ocl::Queue) -> Buffer<f32> {
        Buffer::builder()
            .queue(queue.clone())
            .len(self.len())
            .copy_host_slice(self.as_slice().unwrap())
            .build()
            .unwrap()
    }

    #[tracing::instrument(level = "trace", skip_all)]
    pub(crate) fn update_from_gpu(&mut self, coefs: &Buffer<f32>) {
        coefs.read(self.as_slice_mut().unwrap()).enq().unwrap();
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
    /// # Panics
    ///
    /// If the target directory cannot be created, the file cannot be opened,
    /// or if a delay value cannot be converted to `u32`.
    #[tracing::instrument(level = "trace")]
    pub fn save_npy(&self, path: &std::path::Path) {
        trace!("Saving delays to npy");
        fs::create_dir_all(path).unwrap();
        let writer = BufWriter::new(File::create(path.join("delays.npy")).unwrap());
        self.map(|v| u32::try_from(*v).unwrap())
            .write_npy(writer)
            .unwrap();
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
