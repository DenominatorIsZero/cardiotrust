use std::fs;
use std::fs::File;
use std::io::BufWriter;

use approx::assert_relative_eq;
use ndarray::{Array3, Array4, Array5, Dim};
use ndarray_npy::WriteNpyExt;
use num_traits::Zero;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct ArrayGains<T>
where
    T: Clone + Zero + PartialEq,
{
    pub values: Array5<T>,
}

impl<T> ArrayGains<T>
where
    T: Clone + Zero + PartialEq,
{
    #[must_use]
    pub fn empty(number_of_states: usize) -> Self {
        Self {
            values: Array5::zeros((number_of_states, 3, 3, 3, 3)),
        }
    }
}
impl ArrayGains<f32> {
    pub(crate) fn save_npy(&self, path: &std::path::Path, name: &str) {
        fs::create_dir_all(path).unwrap();
        let writer = BufWriter::new(File::create(path.join(name)).unwrap());
        self.values.write_npy(writer).unwrap();
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct ArrayIndicesGains {
    pub values: Array5<Option<usize>>,
}

impl ArrayIndicesGains {
    #[must_use]
    pub fn empty(number_of_states: usize) -> Self {
        Self {
            values: Array5::from_elem((number_of_states, 3, 3, 3, 3), None),
        }
    }

    pub fn save_npy(&self, path: &std::path::Path) {
        fs::create_dir_all(path).unwrap();
        let writer = BufWriter::new(File::create(path.join("output_state_indices.npy")).unwrap());
        self.values
            .map(|v| match v {
                Some(index) => *index as i32,
                None => -1,
            })
            .write_npy(writer)
            .unwrap();
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct ArrayDelays<T>
where
    T: Clone + Zero + PartialEq,
{
    pub values: Array4<T>,
}

impl<T> ArrayDelays<T>
where
    T: Clone + Zero + PartialEq,
{
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn empty(number_of_states: usize) -> Self {
        assert_relative_eq!(number_of_states as f32 % 3.0, 0.0);
        Self {
            values: Array4::zeros((number_of_states / 3, 3, 3, 3)),
        }
    }
}

impl ArrayDelays<f32> {
    pub fn save_npy(&self, path: &std::path::Path) {
        let writer = BufWriter::new(File::create(path.join("coefs.npy")).unwrap());
        self.values.write_npy(writer).unwrap();
    }
}

impl ArrayDelays<usize> {
    pub fn save_npy(&self, path: &std::path::Path) {
        fs::create_dir_all(path).unwrap();
        let writer = BufWriter::new(File::create(path.join("delays.npy")).unwrap());
        self.values.map(|v| *v as u32).write_npy(writer).unwrap();
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ArrayActivationTime {
    pub values: Array3<Option<f32>>,
}

impl ArrayActivationTime {
    #[must_use]
    pub fn empty(voxels_in_dims: Dim<[usize; 3]>) -> Self {
        Self {
            values: Array3::from_elem(voxels_in_dims, None),
        }
    }

    pub(crate) fn save_npy(&self, path: &std::path::Path) {
        fs::create_dir_all(path).unwrap();
        let writer = BufWriter::new(File::create(path.join("activation_time.npy")).unwrap());
        self.values
            .map(|v| match v {
                Some(index) => *index,
                None => -1.0,
            })
            .write_npy(writer)
            .unwrap();
    }
}
