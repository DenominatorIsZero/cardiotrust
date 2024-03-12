use approx::assert_relative_eq;
use ndarray::{Array2, Array3, Dim};
use ndarray_npy::WriteNpyExt;
use num_traits::Zero;
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::BufWriter,
};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ArrayActivationTime {
    pub values: Array3<Option<f32>>,
}

impl ArrayActivationTime {
    #[must_use]
    /// Creates a new `ArrayActivationTime` with the given voxel dimensions,
    /// initializing all values to `None`.
    pub fn empty(voxels_in_dims: Dim<[usize; 3]>) -> Self {
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
    pub(crate) fn save_npy(&self, path: &std::path::Path) {
        fs::create_dir_all(path).unwrap();
        let writer = BufWriter::new(File::create(path.join("activation_time.npy")).unwrap());
        self.values
            .map(|v| v.as_ref().map_or_else(|| -1.0, |index| *index))
            .write_npy(writer)
            .unwrap();
    }
}

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct ArrayGains<T>
where
    T: Clone + Zero + PartialEq,
{
    pub values: Array2<T>,
}

impl<T> ArrayGains<T>
where
    T: Clone + Zero + PartialEq,
{
    /// Creates a new `ArrayGains` with the given number of states,
    /// initializing all values to zeros.
    #[must_use]
    pub fn empty(number_of_states: usize) -> Self {
        Self {
            values: Array2::zeros((number_of_states, 78)),
        }
    }
}
impl ArrayGains<f32> {
    /// Saves the array values to a .npy file at the given path with the given name.
    /// The values are written directly to the file using `write_npy`.
    ///
    /// # Panics
    ///
    /// Panics if the file cannot be created or written to.
    #[allow(dead_code)]
    pub(crate) fn save_npy(&self, path: &std::path::Path, name: &str) {
        fs::create_dir_all(path).unwrap();
        let writer = BufWriter::new(File::create(path.join(name)).unwrap());
        self.values.write_npy(writer).unwrap();
    }
}

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct ArrayIndicesGains {
    pub values: Array2<Option<usize>>,
}

impl ArrayIndicesGains {
    /// Creates a new `ArrayIndicesGains` with the given number of states,
    /// initializing all values to `None`.
    #[must_use]
    pub fn empty(number_of_states: usize) -> Self {
        Self {
            values: Array2::from_elem((number_of_states, 78), None),
        }
    }

    /// Saves the array indices values to a .npy file at the given path.  
    /// The indices are converted to i32 and any None values are converted to -1.
    /// The values are then written directly to the file using `write_npy`.
    ///
    /// # Panics
    ///
    /// Panics if the file cannot be created or written to.
    pub fn save_npy(&self, path: &std::path::Path) {
        fs::create_dir_all(path).unwrap();
        let writer = BufWriter::new(File::create(path.join("output_state_indices.npy")).unwrap());
        self.values
            .map(|v| {
                v.as_ref()
                    .map_or(-1, |index| i32::try_from(*index).unwrap())
            })
            .write_npy(writer)
            .unwrap();
    }
}

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct ArrayDelays<T>
where
    T: Clone + Zero + PartialEq,
{
    pub values: Array2<T>,
}

impl<T> ArrayDelays<T>
where
    T: Clone + Zero + PartialEq,
{
    /// Creates a new `ArrayDelays` with the given number of states,
    /// initializing all values to 0. The number of states must be divisible by 3.
    ///
    /// # Panics
    ///
    /// Panics if `number_of_states` is not divisible by 3.
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn empty(number_of_states: usize) -> Self {
        assert_relative_eq!(number_of_states as f32 % 3.0, 0.0);
        Self {
            values: Array2::zeros((number_of_states / 3, 26)),
        }
    }
}

impl ArrayDelays<f32> {
    /// Saves the values in this `ArrayDelays` to a .npy file at the given path.
    ///
    /// The .npy file will be named `coefs.npy`.
    ///
    /// # Panics
    ///
    /// Panics if the file cannot be created or written to.
    pub fn save_npy(&self, path: &std::path::Path) {
        fs::create_dir_all(path).unwrap();
        let writer = BufWriter::new(File::create(path.join("coefs.npy")).unwrap());
        self.values.write_npy(writer).unwrap();
    }
}

impl ArrayDelays<usize> {
    /// Saves the delay line values in this `ArrayDelays` to a .npy file at the given path.
    ///
    /// Casts the `usize` values to `u32` before writing to satisfy `.npy` format limitations.
    ///
    /// # Panics
    ///
    /// If the target directory cannot be created, the file cannot be opened,
    /// or if a delay value cannot be converted to `u32`.
    pub fn save_npy(&self, path: &std::path::Path) {
        fs::create_dir_all(path).unwrap();
        let writer = BufWriter::new(File::create(path.join("delays.npy")).unwrap());
        self.values
            .map(|v| u32::try_from(*v).unwrap())
            .write_npy(writer)
            .unwrap();
    }
}
