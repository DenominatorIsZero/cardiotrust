use std::{
    fs::{self, File},
    io::BufWriter,
};

use ndarray::Array1;
use ndarray_npy::{read_npy, WriteNpyExt};

use samplerate::{self, ConverterType};
use serde::{Deserialize, Serialize};

use crate::core::{
    config::model::Model,
    model::spatial::{voxels::VoxelType, VoxelTypes},
};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[allow(clippy::module_name_repetitions)]
pub struct ControlMatrix {
    pub values: Array1<f32>,
}

impl ControlMatrix {
    #[must_use]
    pub fn empty(number_of_states: usize) -> Self {
        Self {
            values: Array1::zeros(number_of_states),
        }
    }

    /// .
    ///
    /// # Panics
    ///
    /// Panics if the `v_number` of the sinoatrial node is None.
    #[must_use]
    pub fn from_model_config(_config: &Model, spatial_description: &VoxelTypes) -> Self {
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

    pub(crate) fn save_npy(&self, path: &std::path::Path) {
        fs::create_dir_all(path.clone()).unwrap();
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
    pub fn empty(number_of_steps: usize) -> Self {
        Self {
            values: Array1::zeros(number_of_steps),
        }
    }

    /// .
    ///
    /// # Panics
    ///
    /// Panics if the control function input file is missing.
    #[must_use]
    pub fn from_model_config(_config: &Model, sample_rate_hz: f32, duration_s: f32) -> Self {
        let sample_rate_hz_in = 2000.0;
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
        let control_function_converted = samplerate::convert(
            sample_rate_hz_in as u32,
            sample_rate_hz as u32,
            1,
            ConverterType::SincBestQuality,
            &control_function_raw.to_vec(),
        )
        .unwrap();

        let control_function_values: Vec<f32> = (0..desired_length_samples)
            .map(|i| {
                let index = i % control_function_converted.len();
                control_function_converted[index]
            })
            .collect();

        Self {
            values: Array1::from(control_function_values),
        }
    }

    pub(crate) fn save_npy(&self, path: &std::path::Path) {
        fs::create_dir_all(path.clone()).unwrap();
        let writer =
            BufWriter::new(File::create(path.join("control_function_values.npy")).unwrap());
        self.values.write_npy(writer).unwrap();
    }
}

#[cfg(test)]
mod test {

    use approx::assert_relative_eq;

    use crate::vis::plotting;

    use super::*;

    #[test]
    fn matrix_from_model_config_no_crash() {
        let config = Model::default();
        let spatial_description = VoxelTypes::from_model_config(&config);

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

        plotting::time::standard_time_plot(
            &control_function.values,
            sample_rate_hz,
            "tests/control_function",
            "Control Function",
            "j [A/mm^2]",
        );
    }
}
