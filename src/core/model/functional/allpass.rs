mod connect;
mod delay;
mod direction;
mod gain;
pub mod shapes;

use anyhow::{Context, Result};
use itertools::Itertools;
use ndarray::Dim;
#[cfg(feature = "native")]
use ocl::{Buffer, Queue};
use serde::{Deserialize, Serialize};
use tracing::{debug, trace};

use self::{
    delay::calculate_delay_samples_array,
    shapes::{ActivationTimeMs, Coefs, Gains, Indices, UnitDelays},
};
use crate::core::{config::model::Model, model::spatial::SpatialDescription};

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct APParameters {
    pub gains: Gains,
    pub output_state_indices: Indices,
    pub coefs: Coefs,
    pub delays: UnitDelays,
    pub initial_delays: Coefs,
    pub activation_time_ms: ActivationTimeMs,
}

#[cfg(feature = "native")]
pub struct APParametersGPU {
    pub gains: Buffer<f32>,
    pub output_state_indices: Buffer<i32>,
    pub coefs: Buffer<f32>,
    pub delays: Buffer<i32>,
}

impl APParameters {
    #[must_use]
    /// Creates an empty `APParameters` struct with the given number of states and
    /// voxel dimensions.
    #[tracing::instrument(level = "debug")]
    pub fn empty(number_of_states: usize, voxels_in_dims: Dim<[usize; 3]>) -> Self {
        debug!("Creating empty AP parameters");
        Self {
            gains: Gains::empty(number_of_states),
            output_state_indices: Indices::empty(number_of_states),
            coefs: Coefs::empty(number_of_states),
            delays: UnitDelays::empty(number_of_states),
            initial_delays: Coefs::empty(number_of_states),
            activation_time_ms: ActivationTimeMs::empty(voxels_in_dims),
        }
    }

    /// Creates AP parameters from the model config and spatial description.
    ///
    /// Calculates the delay samples and coefficients from the propagation velocities.
    /// Initializes the output state indices.
    ///
    /// # Errors
    ///
    /// Returns an error if the AP parameters cannot be created from the given config.
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn from_model_config(
        config: &Model,
        spatial_description: &SpatialDescription,
        sample_rate_hz: f32,
    ) -> Result<Self> {
        debug!("Creating AP parameters from model config");
        let mut ap_params = Self::empty(
            spatial_description.voxels.count_states(),
            spatial_description.voxels.types.raw_dim(),
        );

        connect::connect_voxels(spatial_description, config, &mut ap_params)?;

        let delays_samples = calculate_delay_samples_array(
            spatial_description,
            &config.common.propagation_velocities,
            sample_rate_hz,
        )?;

        ap_params.output_state_indices = init_output_state_indicies(spatial_description)?;

        ap_params
            .delays
            .iter_mut()
            .zip(delays_samples.iter())
            .for_each(|(delay, samples)| *delay = from_samples_to_usize(*samples));

        ap_params
            .coefs
            .iter_mut()
            .zip(delays_samples.iter())
            .for_each(|(coef, samples)| *coef = from_samples_to_coef(*samples));

        ap_params.initial_delays = delays_samples;

        Ok(ap_params)
    }

    /// Saves the allpass filter parameters to .npy files.
    ///
    /// # Errors
    ///
    /// Returns an error if any of the component save operations fail.
    #[cfg(feature = "native")]
    #[tracing::instrument(level = "debug")]
    pub(crate) fn save_npy(&self, path: &std::path::Path) -> Result<()> {
        debug!("Saving allpass parameters to npy");
        let path = &path.join("allpass");
        self.gains.save_npy(path, "gains.npy")?;
        self.output_state_indices.save_npy(path)?;
        self.coefs.save_npy(path)?;
        self.delays.save_npy(path)?;
        self.activation_time_ms.save_npy(path)?;
        Ok(())
    }

    #[cfg(feature = "native")]
    #[tracing::instrument(level = "trace", skip_all)]
    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    pub fn to_gpu(&self, queue: &Queue) -> Result<APParametersGPU> {
        let delays_i32: Vec<i32> = self.delays.iter().map(|&x| x as i32).collect();
        Ok(APParametersGPU {
            gains: Buffer::builder()
                .queue(queue.clone())
                .len(self.gains.len())
                .copy_host_slice(
                    self.gains
                        .as_slice()
                        .context("Failed to get gains slice for GPU copy")?,
                )
                .build()
                .context("Failed to create gains GPU buffer")?,
            output_state_indices: Buffer::builder()
                .queue(queue.clone())
                .len(self.output_state_indices.len())
                .copy_host_slice(
                    self.output_state_indices
                        .mapv(|opt| opt.map_or(-1i32, |val| val as i32))
                        .as_slice()
                        .context("Failed to get output state indices slice for GPU copy")?,
                )
                .build()
                .context("Failed to create output state indices GPU buffer")?,
            coefs: Buffer::builder()
                .queue(queue.clone())
                .len(self.coefs.len())
                .copy_host_slice(
                    self.coefs
                        .as_slice()
                        .context("Failed to get coefs slice for GPU copy")?,
                )
                .build()
                .context("Failed to create coefs GPU buffer")?,
            delays: Buffer::builder()
                .queue(queue.clone())
                .len(delays_i32.len())
                .copy_host_slice(delays_i32.as_slice())
                .build()
                .context("Failed to create delays GPU buffer")?,
        })
    }

    #[cfg(feature = "native")]
    #[allow(clippy::cast_sign_loss)]
    #[tracing::instrument(level = "trace", skip_all)]
    pub(crate) fn update_from_gpu(&mut self, ap_params: &APParametersGPU) -> Result<()> {
        ap_params
            .gains
            .read(
                self.gains
                    .as_slice_mut()
                    .context("Failed to get mutable gains slice for GPU read")?,
            )
            .enq()
            .context("Failed to read gains from GPU buffer")?;
        ap_params
            .coefs
            .read(
                self.coefs
                    .as_slice_mut()
                    .context("Failed to get mutable coefs slice for GPU read")?,
            )
            .enq()
            .context("Failed to read coefs from GPU buffer")?;
        let mut temp_i32 = vec![0i32; self.delays.len()];
        ap_params
            .delays
            .read(&mut temp_i32)
            .enq()
            .context("Failed to read delays from GPU buffer")?;
        self.delays
            .iter_mut()
            .zip(temp_i32.iter())
            .for_each(|(dest, &src)| *dest = src as usize);
        Ok(())
    }
}

/// Initializes the output state indices for the allpass filter based on the
/// spatial description. It finds neighboring output voxels for each input
/// voxel and maps the input states to the corresponding output states. This
/// allows signals to propagate from input voxels to neighboring output voxels
/// through the allpass filter.
#[tracing::instrument(level = "debug", skip_all)]
fn init_output_state_indicies(spatial_description: &SpatialDescription) -> Result<Indices> {
    debug!("Initializing output state indices");
    let mut output_state_indices = Indices::empty(spatial_description.voxels.count_states());
    let v_types = &spatial_description.voxels.types;
    let v_numbers = &spatial_description.voxels.numbers;
    // TODO: write tests
    for (input_voxel_index, v_type) in v_types.indexed_iter() {
        if !v_type.is_connectable() {
            continue;
        }
        let (x_in, y_in, z_in) = input_voxel_index;
        for ((x_offset, y_offset), z_offset) in
            (-1..=1).cartesian_product(-1..=1).cartesian_product(-1..=1)
        {
            if x_offset == 0 && y_offset == 0 && z_offset == 0 {
                continue;
            }
            let x_in_i32 = i32::try_from(x_in)
                .with_context(|| format!("Voxel x-coordinate {x_in} exceeds i32::MAX"))?;
            let y_in_i32 = i32::try_from(y_in)
                .with_context(|| format!("Voxel y-coordinate {y_in} exceeds i32::MAX"))?;
            let z_in_i32 = i32::try_from(z_in)
                .with_context(|| format!("Voxel z-coordinate {z_in} exceeds i32::MAX"))?;

            let ouput_voxel_index_candidate = [
                x_in_i32 + x_offset,
                y_in_i32 + y_offset,
                z_in_i32 + z_offset,
            ];
            if !spatial_description
                .voxels
                .is_valid_index(ouput_voxel_index_candidate)
            {
                continue;
            }
            let x_out_usize =
                usize::try_from(ouput_voxel_index_candidate[0]).with_context(|| {
                    format!(
                        "Output voxel x-coordinate {} cannot be converted to usize",
                        ouput_voxel_index_candidate[0]
                    )
                })?;
            let y_out_usize =
                usize::try_from(ouput_voxel_index_candidate[1]).with_context(|| {
                    format!(
                        "Output voxel y-coordinate {} cannot be converted to usize",
                        ouput_voxel_index_candidate[1]
                    )
                })?;
            let z_out_usize =
                usize::try_from(ouput_voxel_index_candidate[2]).with_context(|| {
                    format!(
                        "Output voxel z-coordinate {} cannot be converted to usize",
                        ouput_voxel_index_candidate[2]
                    )
                })?;

            let output_voxel_index = [x_out_usize, y_out_usize, z_out_usize];
            for input_direction in 0..3 {
                let input_base_number = v_numbers[input_voxel_index].with_context(|| {
                    format!("Input voxel at {input_voxel_index:?} has no assigned number")
                })?;
                let input_state_number = input_base_number + input_direction;
                for output_dimension in 0..3 {
                    let gain_index = offset_to_gain_index(x_offset, y_offset, z_offset, output_dimension)
                        .with_context(|| format!("Failed to calculate gain index for offset ({x_offset}, {y_offset}, {z_offset}) and output dimension {output_dimension}"))?;
                    let output_base_number = v_numbers[output_voxel_index].with_context(|| {
                        format!("Output voxel at {output_voxel_index:?} has no assigned number")
                    })?;
                    let output_state_index = output_base_number + output_dimension;
                    output_state_indices[(input_state_number, gain_index)] =
                        Some(output_state_index);
                }
            }
        }
    }
    Ok(output_state_indices)
}

/// Converts the given x, y, z offset values to an index in the 2D gains array.
///
/// The offsets are relative to a given input voxel. The output dimension
/// indicates which output voxel the gain value is for. Handles converting the
/// 3D coordinate offsets to 1D index. Returns None if offsets are all zero.
#[allow(clippy::cast_sign_loss)]
#[must_use]
pub const fn offset_to_gain_index(
    x_offset: i32,
    y_offset: i32,
    z_offset: i32,
    output_dimension: usize,
) -> Option<usize> {
    if x_offset == 0 && y_offset == 0 && z_offset == 0 {
        return None;
    }
    let mut index = output_dimension
        + (z_offset + 1) as usize * 3
        + (y_offset + 1) as usize * 9
        + (x_offset + 1) as usize * 27;
    if index > 27 + 9 + 3 {
        index -= 3;
    }
    Some(index)
}

/// Converts a 1D index into the gains array to the corresponding
/// x, y, z offset values and output dimension. Returns None if the index
/// is out of bounds of the gains array.
#[allow(
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap
)]
#[must_use]
pub const fn gain_index_to_offset(gain_index: usize) -> Option<[i32; 4]> {
    if gain_index > 77 {
        return None;
    }
    let corrected_index = if gain_index >= 27 + 9 + 3 {
        gain_index + 3
    } else {
        gain_index
    };

    let output_dimension = (corrected_index % 3) as i32;
    let z_offset = ((corrected_index / 3) % 3) as i32 - 1;
    let y_offset = ((corrected_index / 9) % 3) as i32 - 1;
    let x_offset = ((corrected_index / 27) % 3) as i32 - 1;

    Some([x_offset, y_offset, z_offset, output_dimension])
}

/// Converts the given x, y, z offset values to a 1D index into the delays array.
/// Returns None if x, y, z offsets are all 0.
#[allow(clippy::cast_sign_loss)]
#[must_use]
pub const fn offset_to_delay_index(x_offset: i32, y_offset: i32, z_offset: i32) -> Option<usize> {
    if x_offset == 0 && y_offset == 0 && z_offset == 0 {
        return None;
    }
    let mut index =
        (z_offset + 1) as usize + (y_offset + 1) as usize * 3 + (x_offset + 1) as usize * 9;
    if index > 9 + 3 + 1 {
        index -= 1;
    }
    Some(index)
}

/// Converts a 1D index into the delay array to the corresponding
/// x, y, z offset values. Returns None if the index
/// is out of bounds of the gains array.
#[allow(
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap
)]
#[must_use]
pub const fn delay_index_to_offset(delay_index: usize) -> Option<[i32; 3]> {
    if delay_index > 26 {
        return None;
    }
    let corrected_index = if delay_index > 9 + 3 {
        delay_index + 1
    } else {
        delay_index
    };

    let z_offset = (corrected_index % 3) as i32 - 1;
    let y_offset = ((corrected_index / 3) % 3) as i32 - 1;
    let x_offset = ((corrected_index / 9) % 3) as i32 - 1;

    Some([x_offset, y_offset, z_offset])
}

/// Converts a sample value in the range to the corresponding
/// all-pass filter coefficient.
#[tracing::instrument(level = "trace")]
pub fn from_samples_to_coef(samples: f32) -> f32 {
    trace!("Converting {} samples to coefficient", samples);
    let fractional = samples % 1.0;
    let coef = (1.0 - fractional) / (1.0 + fractional);
    coef.clamp(1e-4, 1.0 - 1e-4)
}

/// Computes the integer part of the given samples value.
#[allow(
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation
)]
#[must_use]
pub const fn from_samples_to_usize(samples: f32) -> usize {
    samples as usize
}

/// Converts an all-pass filter coefficient to the corresponding delay in samples.
#[must_use]
#[tracing::instrument(level = "trace")]
pub fn from_coef_to_samples(coef: f32) -> f32 {
    trace!("Converting coefficient {} to samples", coef);
    (1.0 - coef) / (coef + 1.0)
}

#[cfg(test)]
mod tests;
