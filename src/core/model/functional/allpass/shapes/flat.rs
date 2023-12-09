use std::fs;
use std::fs::File;
use std::io::BufWriter;

use approx::assert_relative_eq;
use ndarray::Array2;
use ndarray_npy::WriteNpyExt;
use num_traits::Zero;
use serde::Deserialize;
use serde::Serialize;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct ArrayGainsFlat<T>
where
    T: Clone + Zero + PartialEq,
{
    pub values: Array2<T>,
}

impl<T> ArrayGainsFlat<T>
where
    T: Clone + Zero + PartialEq,
{
    #[must_use]
    pub fn empty(number_of_states: usize) -> Self {
        Self {
            values: Array2::zeros((number_of_states, 78)),
        }
    }
}
impl ArrayGainsFlat<f32> {
    #[allow(dead_code)]
    pub(crate) fn save_npy(&self, path: &std::path::Path, name: &str) {
        fs::create_dir_all(path).unwrap();
        let writer = BufWriter::new(File::create(path.join(name)).unwrap());
        self.values.write_npy(writer).unwrap();
    }
}

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct ArrayIndicesGainsFlat {
    pub values: Array2<Option<usize>>,
}

impl ArrayIndicesGainsFlat {
    #[must_use]
    pub fn empty(number_of_states: usize) -> Self {
        Self {
            values: Array2::from_elem((number_of_states, 78), None),
        }
    }

    ///
    /// # Panics
    ///
    /// Panics if files or directories can't be written or if indices don't fit into i32s.
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
pub struct ArrayDelaysFlat<T>
where
    T: Clone + Zero + PartialEq,
{
    pub values: Array2<T>,
}

impl<T> ArrayDelaysFlat<T>
where
    T: Clone + Zero + PartialEq,
{
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn empty(number_of_states: usize) -> Self {
        assert_relative_eq!(number_of_states as f32 % 3.0, 0.0);
        Self {
            values: Array2::zeros((number_of_states / 3, 26)),
        }
    }
}

impl ArrayDelaysFlat<f32> {
    ///
    /// # Panics
    ///
    /// Panics if file or directory can't be written to.
    pub fn save_npy(&self, path: &std::path::Path) {
        fs::create_dir_all(path).unwrap();
        let writer = BufWriter::new(File::create(path.join("coefs.npy")).unwrap());
        self.values.write_npy(writer).unwrap();
    }
}

impl ArrayDelaysFlat<usize> {
    ///
    /// # Panics
    ///
    /// Panics if file or directory can't be written to or delays don't fit into u32.
    pub fn save_npy(&self, path: &std::path::Path) {
        fs::create_dir_all(path).unwrap();
        let writer = BufWriter::new(File::create(path.join("delays.npy")).unwrap());
        self.values
            .map(|v| u32::try_from(*v).unwrap())
            .write_npy(writer)
            .unwrap();
    }
}