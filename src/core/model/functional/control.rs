use ndarray::Array1;
use ndarray_npy::{read_npy, WriteNpyExt};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::BufWriter,
};
use tracing::{debug, trace};

use crate::core::{
    config::model::Model,
    model::spatial::{voxels::VoxelType, SpatialDescription},
};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[allow(clippy::module_name_repetitions)]
pub struct ControlMatrix {
    pub values: Array1<f32>,
}

impl ControlMatrix {
    /// Creates a new empty `ControlMatrix` with the given number of states initialized
    /// to all zeros.
    #[must_use]
    #[tracing::instrument(level = "debug")]
    pub fn empty(number_of_states: usize) -> Self {
        debug!("Creating empty control matrix");
        Self {
            values: Array1::zeros(number_of_states),
        }
    }

    /// Creates a `ControlMatrix` from the given `Model` configuration and
    /// `SpatialDescription`. Initializes the control matrix by setting the value
    /// for the state of the sinoatrial voxel to 1.0, and all other states to 0.
    ///
    /// # Panics
    ///
    /// Panics if the `v_number` of the sinoatrial node is None.
    #[must_use]
    #[tracing::instrument(level = "debug")]
    pub fn from_model_config(config: &Model, spatial_description: &SpatialDescription) -> Self {
        debug!("Creating control matrix from model config");
        let mut control_matrix = Self::empty(spatial_description.voxels.count_states());
        spatial_description
            .voxels
            .types
            .values
            .iter()
            .zip(spatial_description.voxels.numbers.values.iter())
            .for_each(|(v_type, v_number)| {
                if *v_type == VoxelType::Sinoatrial {
                    control_matrix.values[v_number.unwrap()] = 1.0;
                }
            });
        control_matrix
    }

    /// Saves the control matrix to a .npy file at the given path.
    /// Creates any missing directories in the path if needed.
    #[tracing::instrument(level = "trace")]
    pub(crate) fn save_npy(&self, path: &std::path::Path) {
        trace!("Saving control matrix to npy");
        fs::create_dir_all(path).unwrap();
        let writer = BufWriter::new(File::create(path.join("control_matrix.npy")).unwrap());
        self.values.write_npy(writer).unwrap();
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[allow(clippy::module_name_repetitions)]
pub struct ControlFunction {
    pub values: Array1<f32>,
}

impl ControlFunction {
    #[must_use]
    #[tracing::instrument(level = "debug")]
    pub fn empty(number_of_steps: usize) -> Self {
        debug!("Creating empty control function");
        Self {
            values: Array1::zeros(number_of_steps),
        }
    }

    /// Creates a new `ControlFunction` by reading a control function .npy file,
    /// resampling it to match the given sample rate and duration, and returning
    /// the resampled values as a new `ControlFunction`.
    ///
    /// The control function .npy file is assumed to be located in `assets/`.
    /// The resampling is done by looping through the target number of samples
    /// based on sample rate and duration, and taking values from the .npy file
    /// using modulo to wrap the index.
    ///
    /// This allows creating a `ControlFunction` of arbitrary duration from a fixed
    /// length control function file.
    ///
    /// # Panics
    ///
    /// Panics if the control function input file is missing.
    #[must_use]
    #[tracing::instrument(level = "debug")]
    pub fn from_model_config(config: &Model, sample_rate_hz: f32, duration_s: f32) -> Self {
        debug!("Creating control function from model config");
        //let _sample_rate_hz_in = 2000.0;
        let control_function_raw: Array1<f32> =
            read_npy("assets/control_function_ohara.npy").unwrap();

        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_precision_loss,
            clippy::cast_sign_loss
        )]
        let desired_length_samples = (duration_s * sample_rate_hz) as usize;

        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_precision_loss,
            clippy::cast_sign_loss
        )]
        let control_function_values: Vec<f32> = (0..desired_length_samples)
            .map(|i| {
                let index = i % control_function_raw.len();
                control_function_raw[index]
            })
            .collect();

        Self {
            values: Array1::from(control_function_values),
        }
    }

    /// Saves the control function values to a .npy file at the given path.
    /// Creates any missing directories in the path, opens a file for writing,
    /// and writes the values using the numpy npy format.
    #[tracing::instrument(level = "trace")]
    pub(crate) fn save_npy(&self, path: &std::path::Path) {
        trace!("Saving control function values to npy");
        fs::create_dir_all(path).unwrap();
        let writer =
            BufWriter::new(File::create(path.join("control_function_values.npy")).unwrap());
        self.values.write_npy(writer).unwrap();
    }
}

#[cfg(test)]
mod test {

    use std::path::Path;

    use approx::assert_relative_eq;

    use crate::vis::plotting::line::standard_time_plot;

    use super::*;

    #[test]
    fn matrix_from_model_config_no_crash() {
        let config = Model::default();
        let spatial_description = SpatialDescription::from_model_config(&config);

        let control_matrix = ControlMatrix::from_model_config(&config, &spatial_description);
        let sum = control_matrix.values.sum();
        assert_relative_eq!(sum, 1.0);
    }

    #[test]
    fn function_from_model_config_no_crash() {
        let sample_rate_hz = 3000.0;
        let duration_s = 1.5;
        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_precision_loss,
            clippy::cast_sign_loss
        )]
        let expected_length_samples = (sample_rate_hz * duration_s) as usize;
        let config = Model::default();

        let control_function =
            ControlFunction::from_model_config(&config, sample_rate_hz, duration_s);
        assert_eq!(expected_length_samples, control_function.values.shape()[0]);
    }

    #[test]
    #[ignore]
    fn function_from_model_config_no_crash_and_plot() {
        let sample_rate_hz = 3000.0;
        let duration_s = 1.5;
        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_precision_loss,
            clippy::cast_sign_loss
        )]
        let expected_length_samples = (sample_rate_hz * duration_s) as usize;
        let config = Model::default();

        let control_function =
            ControlFunction::from_model_config(&config, sample_rate_hz, duration_s);
        assert_eq!(expected_length_samples, control_function.values.shape()[0]);

        standard_time_plot(
            &control_function.values,
            sample_rate_hz,
            Path::new("tests/control_function"),
            "Control Function",
            "j [A/mm^2]",
        )
        .unwrap();
    }
}
