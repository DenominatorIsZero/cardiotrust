use ndarray::Array2;
use ndarray_npy::WriteNpyExt;
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::BufWriter,
};

/// Shape for the simulated/estimated system states
///
/// Has dimensions (`number_of_steps` `number_of_states`)
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ArraySystemStates {
    pub values: Array2<f32>,
}

impl ArraySystemStates {
    #[must_use]
    pub fn empty(number_of_steps: usize, number_of_states: usize) -> Self {
        Self {
            values: Array2::zeros((number_of_steps, number_of_states)),
        }
    }

    /// .
    ///
    /// # Panics
    ///
    /// Panics if directory of file cant be created.
    pub fn save_npy(&self, path: &std::path::Path) {
        fs::create_dir_all(path.clone()).unwrap();

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
    pub fn empty(number_of_steps: usize, number_of_sensors: usize) -> Self {
        Self {
            values: Array2::zeros((number_of_steps, number_of_sensors)),
        }
    }

    ///
    /// # Panics
    ///
    /// Panics if file or directory can't be written.
    pub fn save_npy(&self, path: &std::path::Path) {
        fs::create_dir_all(path).unwrap();
        let writer = BufWriter::new(File::create(path.join("measurements.npy")).unwrap());
        self.values.write_npy(writer).unwrap();
    }
}
