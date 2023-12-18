use std::{
    fs::{self, File},
    io::BufWriter,
};

use ndarray::{Array3, Dim};
use ndarray_npy::WriteNpyExt;
use serde::{Deserialize, Serialize};

pub mod flat;

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

    ///
    /// # Panics
    ///
    /// Panics if file or directory can't be written to.
    pub(crate) fn save_npy(&self, path: &std::path::Path) {
        fs::create_dir_all(path).unwrap();
        let writer = BufWriter::new(File::create(path.join("activation_time.npy")).unwrap());
        self.values
            .map(|v| v.as_ref().map_or_else(|| -1.0, |index| *index))
            .write_npy(writer)
            .unwrap();
    }
}
