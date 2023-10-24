use std::{
    fs::{self, File},
    io::BufWriter,
};

use ndarray::{arr1, Array1};
use ndarray_npy::WriteNpyExt;
use serde::{Deserialize, Serialize};

use crate::core::config::model::Model;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Heart {
    pub origin_mm: Array1<f32>,
    pub size_mm: Array1<f32>,
}

impl Heart {
    #[must_use]
    pub fn empty() -> Self {
        Self {
            origin_mm: Array1::zeros(3),
            size_mm: Array1::zeros(3),
        }
    }

    #[must_use]
    pub fn from_model_config(config: &Model) -> Self {
        Self {
            origin_mm: arr1(&config.heart_origin_mm),
            size_mm: arr1(&config.heart_size_mm),
        }
    }

    pub(crate) fn save_npy(&self, path: std::path::PathBuf) {
        fs::create_dir_all(path.clone()).unwrap();
        let writer = BufWriter::new(File::create(path.join("heart_origin_mm.npy")).unwrap());
        self.origin_mm.write_npy(writer).unwrap();
        let writer = BufWriter::new(File::create(path.join("heart_size_mm.npy")).unwrap());
        self.size_mm.write_npy(writer).unwrap();
    }
}
