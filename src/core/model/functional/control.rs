use std::{
    fs::{self, File},
    io::BufWriter,
    ops::{Deref, DerefMut},
};

use anyhow::{Context, Result};
use approx::RelativeEq;
use ndarray::Array1;
use ndarray_npy::{read_npy, WriteNpyExt};
use ocl::Buffer;
use rubato::{Resampler, SincFixedIn, SincInterpolationParameters};
use serde::{Deserialize, Serialize};
use tracing::{debug, trace};

use crate::core::{
    config::{self, model::Model},
    model::spatial::{voxels::VoxelType, SpatialDescription},
};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[allow(clippy::module_name_repetitions)]
pub struct ControlMatrix(Array1<f32>);

impl ControlMatrix {
    /// Creates a new empty `ControlMatrix` with the given number of states initialized
    /// to all zeros.
    #[must_use]
    #[tracing::instrument(level = "debug")]
    pub fn empty(number_of_states: usize) -> Self {
        debug!("Creating empty control matrix");
        Self(Array1::zeros(number_of_states))
    }

    /// Creates a `ControlMatrix` from the given `Model` configuration and
    /// `SpatialDescription`. Initializes the control matrix by setting the value
    /// for the state of the sinoatrial voxel to 1.0, and all other states to 0.
    ///
    /// # Errors
    ///
    /// Returns an error if the sinoatrial node voxel number is not available.
    #[tracing::instrument(level = "debug")]
    pub fn from_model_config(config: &Model, spatial_description: &SpatialDescription) -> Result<Self> {
        debug!("Creating control matrix from model config");
        let mut control_matrix = Self::empty(spatial_description.voxels.count_states());

        for (v_type, v_number) in spatial_description.voxels.types.iter().zip(spatial_description.voxels.numbers.iter()) {
            if *v_type == VoxelType::Sinoatrial {
                let voxel_number = v_number
                    .context("Sinoatrial node voxel must have a valid voxel number for control matrix initialization")?;
                control_matrix[voxel_number] = 1.0;
            }
        }

        Ok(control_matrix)
    }

    /// Saves the control matrix to a .npy file at the given path.
    /// Creates any missing directories in the path if needed.
    ///
    /// # Errors
    ///
    /// Returns an error if the directory cannot be created or the file cannot be written.
    #[tracing::instrument(level = "trace")]
    pub(crate) fn save_npy(&self, path: &std::path::Path) -> Result<()> {
        trace!("Saving control matrix to npy");
        fs::create_dir_all(path)
            .with_context(|| format!("Failed to create directory for control matrix: {}", path.display()))?;

        let file_path = path.join("control_matrix.npy");
        let writer = BufWriter::new(File::create(&file_path)
            .with_context(|| format!("Failed to create control matrix file: {}", file_path.display()))?);

        self.write_npy(writer)
            .with_context(|| format!("Failed to write control matrix to: {}", file_path.display()))?;

        Ok(())
    }

    #[tracing::instrument(level = "trace", skip_all)]
    pub(crate) fn to_gpu(&self, queue: &ocl::Queue) -> Result<Buffer<f32>> {
        let buffer = Buffer::builder()
            .queue(queue.clone())
            .len(self.len())
            .copy_host_slice(
                self.as_slice()
                    .context("Failed to get array slice for GPU copy")?,
            )
            .build()
            .context("Failed to build GPU buffer for control matrix")?;
        Ok(buffer)
    }

    #[tracing::instrument(level = "trace", skip_all)]
    pub(crate) fn update_from_gpu(&mut self, control_matrix: &Buffer<f32>) -> Result<()> {
        control_matrix
            .read(
                self.as_slice_mut()
                    .context("Failed to get mutable array slice for GPU read")?,
            )
            .enq()
            .context("Failed to read control matrix from GPU buffer")?;
        Ok(())
    }
}

impl Deref for ControlMatrix {
    type Target = Array1<f32>;

    #[tracing::instrument(level = "trace")]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ControlMatrix {
    #[tracing::instrument(level = "trace")]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[allow(clippy::module_name_repetitions)]
pub struct ControlFunction(Array1<f32>);

impl ControlFunction {
    #[must_use]
    #[tracing::instrument(level = "debug")]
    pub fn empty(number_of_steps: usize) -> Self {
        debug!("Creating empty control function");
        Self(Array1::zeros(number_of_steps))
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
    /// # Errors
    ///
    /// Returns an error if the control function input file cannot be read or
    /// if resampling operations fail.
    #[tracing::instrument(level = "debug")]
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_precision_loss,
        clippy::cast_sign_loss
    )]
    pub fn from_model_config(config: &Model, sample_rate_hz: f32, duration_s: f32) -> Result<Self> {
        debug!("Creating control function from model config");
        let desired_length_samples = (duration_s * sample_rate_hz) as usize;

        match config.common.control_function {
            config::model::ControlFunction::Ohara => {
                let mut control_function_raw: Array1<f32> =
                    read_npy("assets/control_function_ohara.npy")
                        .context("Failed to load O'Hara control function from assets/control_function_ohara.npy")?;

                let from_sample_rate_hz = 2000.0;

                if !from_sample_rate_hz.relative_eq(&sample_rate_hz, 1e-3, 1e-3) {
                    let params = SincInterpolationParameters {
                        sinc_len: 256,
                        f_cutoff: 0.95,
                        oversampling_factor: 256,
                        interpolation: rubato::SincInterpolationType::Cubic,
                        window: rubato::WindowFunction::BlackmanHarris2,
                    };
                    let mut resampler = SincFixedIn::<f32>::new(
                        f64::from(sample_rate_hz) / f64::from(from_sample_rate_hz),
                        10.0,
                        params,
                        control_function_raw.len(),
                        1,
                    )
                    .with_context(|| format!(
                        "Failed to create resampler for O'Hara control function (from {from_sample_rate_hz}Hz to {sample_rate_hz}Hz)"
                    ))?;

                    let input_frames: Vec<Vec<f32>> = vec![control_function_raw.to_vec()];

                    let output_frames = resampler.process(&input_frames, None)
                        .with_context(|| format!(
                            "Failed to resample O'Hara control function from {from_sample_rate_hz}Hz to {sample_rate_hz}Hz"
                        ))?;

                    control_function_raw = output_frames[0].clone().into();
                }

                let control_function_values: Vec<f32> = (0..desired_length_samples)
                    .map(|i| {
                        let index = i % control_function_raw.len();
                        control_function_raw[index]
                    })
                    .collect();

                Ok(Self(Array1::from(control_function_values)))
            }
            config::model::ControlFunction::Triangle => {
                let mut control_function_values = Array1::<f32>::zeros(desired_length_samples);

                let triangle_half_length = (0.5 * sample_rate_hz) as i32;

                let increase_per_step = 1.0 / (triangle_half_length + 1) as f32;

                for i in 0..triangle_half_length {
                    let value = (i + 1) as f32 * increase_per_step;
                    control_function_values[i as usize] = value;
                    control_function_values[2 * triangle_half_length as usize - i as usize - 1] =
                        value;
                }

                control_function_values[triangle_half_length as usize] = 1.0;

                for i in sample_rate_hz as usize..desired_length_samples {
                    control_function_values[i] =
                        control_function_values[i % sample_rate_hz as usize];
                }

                Ok(Self(control_function_values))
            }
            config::model::ControlFunction::Ramp => {
                let mut control_function_values = Array1::<f32>::zeros(desired_length_samples);

                let increase_per_step = 1.0 / (desired_length_samples - 1) as f32;

                for i in 1..desired_length_samples {
                    let value = i as f32 * increase_per_step;
                    control_function_values[i] = -value;
                }
                Ok(Self(control_function_values))
            }
        }
    }

    /// Saves the control function values to a .npy file at the given path.
    /// Creates any missing directories in the path, opens a file for writing,
    /// and writes the values using the numpy npy format.
    ///
    /// # Errors
    ///
    /// Returns an error if the directory cannot be created or the file cannot be written.
    #[tracing::instrument(level = "trace")]
    pub(crate) fn save_npy(&self, path: &std::path::Path) -> Result<()> {
        trace!("Saving control function values to npy");
        fs::create_dir_all(path)
            .with_context(|| format!("Failed to create directory for control function: {}", path.display()))?;

        let file_path = path.join("control_function_values.npy");
        let writer = BufWriter::new(File::create(&file_path)
            .with_context(|| format!("Failed to create control function file: {}", file_path.display()))?);

        self.write_npy(writer)
            .with_context(|| format!("Failed to write control function to: {}", file_path.display()))?;

        Ok(())
    }

    #[tracing::instrument(level = "trace", skip_all)]
    pub(crate) fn to_gpu(&self, queue: &ocl::Queue) -> Result<Buffer<f32>> {
        let buffer = ocl::Buffer::builder()
            .queue(queue.clone())
            .len(self.len())
            .copy_host_slice(
                self.as_slice()
                    .context("Failed to get array slice for GPU copy")?,
            )
            .build()
            .context("Failed to build GPU buffer for control function values")?;
        Ok(buffer)
    }

    #[tracing::instrument(level = "trace", skip_all)]
    pub(crate) fn update_from_gpu(&mut self, control_function_values: &Buffer<f32>) -> Result<()> {
        control_function_values
            .read(
                self.as_slice_mut()
                    .context("Failed to get mutable array slice for GPU read")?,
            )
            .enq()
            .context("Failed to read control function values from GPU buffer")?;
        Ok(())
    }
}

impl Deref for ControlFunction {
    type Target = Array1<f32>;

    #[tracing::instrument(level = "trace")]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ControlFunction {
    #[tracing::instrument(level = "trace")]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[cfg(test)]
mod test {

    use std::path::Path;

    use approx::assert_relative_eq;

    use super::*;
    use crate::{core::config::model::Model, vis::plotting::png::line::standard_time_plot};

    const COMMON_PATH: &str = "tests/core/model/functional/control/";

    #[tracing::instrument(level = "trace")]
    fn setup(folder: Option<&str>) {
        let path = folder.map_or_else(
            || Path::new(COMMON_PATH).to_path_buf(),
            |folder| Path::new(COMMON_PATH).join(folder),
        );

        if !path.exists() {
            std::fs::create_dir_all(path)
                .expect("Failed to create test directory - filesystem issue");
        }
    }

    #[test]
    fn matrix_from_model_config_no_crash() -> Result<()> {
        let config = Model::default();
        let spatial_description = SpatialDescription::from_model_config(&config)?;

        let control_matrix = ControlMatrix::from_model_config(&config, &spatial_description)?;
        let sum = control_matrix.sum();
        assert_relative_eq!(sum, 1.0);
        Ok(())
    }

    #[test]
    fn function_from_model_config_no_crash() -> Result<()> {
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
            ControlFunction::from_model_config(&config, sample_rate_hz, duration_s)?;
        assert_eq!(expected_length_samples, control_function.shape()[0]);
        Ok(())
    }

    #[test]
    fn function_from_model_config_no_crash_and_plot() -> Result<()> {
        setup(None);
        let sample_rate_hz = 3000.0;
        let duration_s = 1.5;
        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_precision_loss,
            clippy::cast_sign_loss
        )]
        let expected_length_samples = (sample_rate_hz * duration_s) as usize;
        let mut config = Model::default();
        config.common.control_function = config::model::ControlFunction::Ohara;

        let control_function =
            ControlFunction::from_model_config(&config, sample_rate_hz, duration_s)?;
        assert_eq!(expected_length_samples, control_function.shape()[0]);

        let path = Path::new(COMMON_PATH).join("control_function_ohara.png");
        standard_time_plot(
            &control_function,
            sample_rate_hz,
            path.as_path(),
            "Control Function",
            "j [A/mm^2]",
        )
        .context("Failed to generate control function plot")?;
        Ok(())
    }

    #[test]
    fn triangle_function_from_model_config_no_crash_and_plot() -> Result<()> {
        setup(None);
        let sample_rate_hz = 3000.0;
        let duration_s = 1.5;
        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_precision_loss,
            clippy::cast_sign_loss
        )]
        let expected_length_samples = (sample_rate_hz * duration_s) as usize;
        let mut config = Model::default();
        config.common.control_function = config::model::ControlFunction::Triangle;

        let control_function =
            ControlFunction::from_model_config(&config, sample_rate_hz, duration_s)?;
        assert_eq!(expected_length_samples, control_function.shape()[0]);

        let path = Path::new(COMMON_PATH).join("control_function_triangle.png");
        standard_time_plot(
            &control_function,
            sample_rate_hz,
            path.as_path(),
            "Control Function",
            "j [A/mm^2]",
        )
        .context("Failed to generate control function plot")?;
        Ok(())
    }

    #[test]
    fn ramp_function_from_model_config_no_crash_and_plot() -> Result<()> {
        setup(None);
        let sample_rate_hz = 3000.0;
        let duration_s = 1.5;
        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_precision_loss,
            clippy::cast_sign_loss
        )]
        let expected_length_samples = (sample_rate_hz * duration_s) as usize;
        let mut config = Model::default();
        config.common.control_function = config::model::ControlFunction::Ramp;

        let control_function =
            ControlFunction::from_model_config(&config, sample_rate_hz, duration_s)?;
        assert_eq!(expected_length_samples, control_function.shape()[0]);

        let path = Path::new(COMMON_PATH).join("control_function_ramp.png");
        standard_time_plot(
            &control_function,
            sample_rate_hz,
            path.as_path(),
            "Control Function",
            "j [A/mm^2]",
        )
        .context("Failed to generate control function plot")?;
        Ok(())
    }
}
