use ndarray::{arr1, Array1};
use ndarray_npy::WriteNpyExt;
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::BufWriter,
};
use tracing::{debug, trace};

use crate::core::config::model::Model;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Heart {
    pub origin_mm: Array1<f32>,
    pub size_mm: Array1<f32>,
}

impl Heart {
    /// Creates an empty Heart struct with origin and size arrays initialized to 0.
    /// This is a convenience constructor for creating a Heart with default values.
    #[must_use]
    #[tracing::instrument(level = "debug")]
    pub fn empty() -> Self {
        debug!("Creating empty heart");
        Self {
            origin_mm: Array1::zeros(3),
            size_mm: Array1::zeros(3),
        }
    }

    /// Creates a Heart struct from the given Model config.
    ///
    /// This initializes the `origin_mm` an`size_mm`mm fields from the
    /// corresponding values in the Model config.
    #[must_use]
    #[tracing::instrument(level = "debug")]
    pub fn from_handcrafted_model_config(config: &Model) -> Self {
        debug!("Creating heart from handcrafted model config");
        Self {
            origin_mm: arr1(&config.common.heart_offset_mm),
            size_mm: arr1(&config.handcrafted.as_ref().unwrap().heart_size_mm),
        }
    }

    #[must_use]
    #[tracing::instrument(level = "debug")]
    pub fn from_mri_model_config(config: &Model) -> Self {
        debug!("Creating heart from mri model config");
        todo!();
    }

    /// Saves the heart origin and size as .npy files in the given path.
    ///
    /// # Panics
    ///
    /// Panics if the directory cannot be created or the files cannot be written.
    #[tracing::instrument(level = "trace")]
    pub(crate) fn save_npy(&self, path: &std::path::Path) {
        trace!("Saving heart origin and size to npy files");
        fs::create_dir_all(path).unwrap();
        let writer = BufWriter::new(File::create(path.join("heart_origin_mm.npy")).unwrap());
        self.origin_mm.write_npy(writer).unwrap();
        let writer = BufWriter::new(File::create(path.join("heart_size_mm.npy")).unwrap());
        self.size_mm.write_npy(writer).unwrap();
    }
}
