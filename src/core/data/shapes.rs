use ndarray::Array2;
use ndarray_npy::WriteNpyExt;
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::BufWriter,
};
use tracing::trace;

/// Shape for the simulated/estimated system states
///
/// Has dimensions (`number_of_steps` `number_of_states`)
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ArraySystemStates {
    pub values: Array2<f32>,
}

impl ArraySystemStates {
    /// Creates an empty `ArraySystemStates` with the given dimensions.
    #[must_use]
    #[tracing::instrument(level = "trace")]
    pub fn empty(number_of_steps: usize, number_of_states: usize) -> Self {
        trace!("Creating empty system states");
        Self {
            values: Array2::zeros((number_of_steps, number_of_states)),
        }
    }

    /// Saves the `ArraySystemStates` to a .npy file at the given path.
    ///
    /// Creates any missing directories in the path, opens a file at that path,
    /// and writes the underlying `values` array to it in .npy format.
    ///
    /// # Panics
    ///
    /// Panics if directory of file cant be created.
    #[tracing::instrument(level = "trace")]
    pub fn save_npy(&self, path: &std::path::Path) {
        trace!("Saving system states");
        fs::create_dir_all(path).unwrap();

        let writer = BufWriter::new(File::create(path.join("system_states.npy")).unwrap());
        self.values.write_npy(writer).unwrap();
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ArrayMeasurements {
    pub values: Array2<f32>,
}

impl ArrayMeasurements {
    #[must_use]
    #[tracing::instrument(level = "trace")]
    /// Creates an empty `ArrayMeasurements` with the given dimensions.
    pub fn empty(number_of_steps: usize, number_of_sensors: usize) -> Self {
        trace!("Creating empty measurements");
        Self {
            values: Array2::zeros((number_of_steps, number_of_sensors)),
        }
    }

    /// Panics if file or directory can't be written.
    /// Saves the `ArrayMeasurements` to a .npy file at the given path.
    ///
    /// Creates any missing directories in the path, opens a file at that path,
    /// and writes the underlying `values` array to it in .npy format.
    ///
    /// # Panics
    ///
    /// Panics if directory of file cant be created.
    #[tracing::instrument(level = "trace")]
    pub fn save_npy(&self, path: &std::path::Path) {
        trace!("Saving measurements");
        fs::create_dir_all(path).unwrap();
        let writer = BufWriter::new(File::create(path.join("measurements.npy")).unwrap());
        self.values.write_npy(writer).unwrap();
    }
}
