mod delay;
mod direction;
mod gain;
pub mod shapes;

use anyhow::{Context, Result};
use approx::relative_eq;
use itertools::Itertools;
use ndarray::{arr1, s, Array1, Array3, Array4, Dim};
use ndarray_stats::QuantileExt;
use ocl::{Buffer, Queue};
use serde::{Deserialize, Serialize};
use tracing::{debug, trace};

use self::{
    delay::calculate_delay_samples_array,
    shapes::{ActivationTimeMs, Coefs, Gains, Indices, UnitDelays},
};
use crate::core::{
    config::model::Model,
    model::spatial::{
        voxels::{self, VoxelType},
        SpatialDescription,
    },
};

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

        connect_voxels(spatial_description, config, &mut ap_params)?;

        let delays_samples = calculate_delay_samples_array(
            spatial_description,
            &config.common.propagation_velocities_m_per_s,
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

    #[tracing::instrument(level = "trace", skip_all)]
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_possible_wrap,
        clippy::missing_panics_doc
    )]
    #[must_use]
    pub fn to_gpu(&self, queue: &Queue) -> Result<APParametersGPU> {
        let delays_i32: Vec<i32> = self.delays.iter().map(|&x| x as i32).collect();
        Ok(APParametersGPU {
            gains: Buffer::builder()
                .queue(queue.clone())
                .len(self.gains.len())
                .copy_host_slice(
                    self.gains.as_slice()
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
                    self.coefs.as_slice()
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

    #[allow(clippy::cast_sign_loss)]
    #[tracing::instrument(level = "trace", skip_all)]
    pub(crate) fn update_from_gpu(&mut self, ap_params: &APParametersGPU) -> Result<()> {
        ap_params
            .gains
            .read(
                self.gains.as_slice_mut()
                    .context("Failed to get mutable gains slice for GPU read")?,
            )
            .enq()
            .context("Failed to read gains from GPU buffer")?;
        ap_params
            .coefs
            .read(
                self.coefs.as_slice_mut()
                    .context("Failed to get mutable coefs slice for GPU read")?,
            )
            .enq()
            .context("Failed to read coefs from GPU buffer")?;
        let mut temp_i32 = vec![0i32; self.delays.len()];
        ap_params.delays.read(&mut temp_i32).enq()
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
                .with_context(|| format!("Voxel x-coordinate {} exceeds i32::MAX", x_in))?;
            let y_in_i32 = i32::try_from(y_in)
                .with_context(|| format!("Voxel y-coordinate {} exceeds i32::MAX", y_in))?;
            let z_in_i32 = i32::try_from(z_in)
                .with_context(|| format!("Voxel z-coordinate {} exceeds i32::MAX", z_in))?;

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
            let x_out_usize = usize::try_from(ouput_voxel_index_candidate[0])
                .with_context(|| format!("Output voxel x-coordinate {} cannot be converted to usize", ouput_voxel_index_candidate[0]))?;
            let y_out_usize = usize::try_from(ouput_voxel_index_candidate[1])
                .with_context(|| format!("Output voxel y-coordinate {} cannot be converted to usize", ouput_voxel_index_candidate[1]))?;
            let z_out_usize = usize::try_from(ouput_voxel_index_candidate[2])
                .with_context(|| format!("Output voxel z-coordinate {} cannot be converted to usize", ouput_voxel_index_candidate[2]))?;

            let output_voxel_index = [x_out_usize, y_out_usize, z_out_usize];
            for input_direction in 0..3 {
                let input_base_number = v_numbers[input_voxel_index]
                    .with_context(|| format!("Input voxel at {:?} has no assigned number", input_voxel_index))?;
                let input_state_number = input_base_number + input_direction;
                for output_dimension in 0..3 {
                    let gain_index = offset_to_gain_index(x_offset, y_offset, z_offset, output_dimension)
                        .with_context(|| format!("Failed to calculate gain index for offset ({}, {}, {}) and output dimension {}", x_offset, y_offset, z_offset, output_dimension))?;
                    let output_base_number = v_numbers[output_voxel_index]
                        .with_context(|| format!("Output voxel at {:?} has no assigned number", output_voxel_index))?;
                    let output_state_index = output_base_number + output_dimension;
                    output_state_indices[(input_state_number, gain_index)] =
                        Some(output_state_index);
                }
            }
        }
    }
    Ok(output_state_indices)
}

/// Connects voxels in the model based on voxel type and proximity.
/// Iteratively activates voxels by updating `activation_time_s` and `current_directions`.
/// Stops when no more voxels can be connected at the current time step.
#[tracing::instrument(level = "debug", skip_all)]
fn connect_voxels(
    spatial_description: &SpatialDescription,
    config: &Model,
    ap_params: &mut APParameters,
) -> Result<()> {
    debug!("Connecting voxels");
    let mut activation_time_s =
        Array3::<Option<f32>>::from_elem(spatial_description.voxels.types.raw_dim(), None);
    let mut current_directions =
        Array4::<f32>::zeros(spatial_description.voxels.positions_mm.raw_dim());

    let v_types = &spatial_description.voxels.types;

    let mut current_time_s: f32 = 0.0;
    // Handle Sinoatrial node
    v_types
        .indexed_iter()
        .filter(|(_, v_type)| **v_type == VoxelType::Sinoatrial)
        .for_each(|(index, _)| {
            activation_time_s[index] = Some(current_time_s);
            current_directions
                .slice_mut(s![index.0, index.1, index.2, ..])
                .assign(&arr1(&[1.0, 0.0, 0.0]));
        });
    let mut connected_something = true;

    while connected_something {
        // reset the connected something variable so we don't get stuck here forever
        // have to check the activation times because there might be some connection possible
        // with a voxel that is not yet activated.
        if !activation_time_s
            .iter()
            .filter_map(|time_s| *time_s)
            .any(|time_s| time_s > current_time_s)
        {
            connected_something = false;
        }
        // find all voxels with an activation time equal to the current time
        // i.e., currently activated voxels
        let output_voxel_indices = find_candidate_voxels(&activation_time_s, current_time_s);

        for output_voxel_index in output_voxel_indices {
            for x_offset in -1..=1 {
                for y_offset in -1..=1 {
                    for z_offset in -1..=1 {
                        connected_something |= try_to_connect(
                            (x_offset, y_offset, z_offset),
                            output_voxel_index,
                            spatial_description,
                            &mut activation_time_s,
                            config,
                            &mut current_directions,
                            ap_params,
                        ).unwrap_or_else(|e| { tracing::error!("Connection failed: {}", e); false });
                    }
                }
            }
        }
        let candidate_times_s: Vec<f32> = activation_time_s
            .iter()
            .filter_map(|&t| t)
            .filter(|&t| t > current_time_s)
            .collect();
        let candidate_times_s = Array1::from_vec(candidate_times_s);
        current_time_s = *candidate_times_s.min_skipnan();
    }
    ap_params
        .activation_time_ms
        .iter_mut()
        .zip(activation_time_s)
        .for_each(|(ms, s)| *ms = s.map(|time| time * 1000.0));
    Ok(())
}

/// Attempts to connect the voxel at the given offset from the output voxel.
/// Returns true if a connection was made, false otherwise.
#[tracing::instrument(level = "trace")]
fn try_to_connect(
    voxel_offset: (i32, i32, i32),
    output_voxel_index: (usize, usize, usize),
    spatial_description: &SpatialDescription,
    activation_time_s: &mut ndarray::ArrayBase<ndarray::OwnedRepr<Option<f32>>, Dim<[usize; 3]>>,
    config: &Model,
    current_directions: &mut ndarray::ArrayBase<ndarray::OwnedRepr<f32>, Dim<[usize; 4]>>,
    ap_params: &mut APParameters,
) -> Result<bool> {
    trace!(
        "Trying to connect voxel at offset {:?} to output voxel {:?}",
        voxel_offset,
        output_voxel_index
    );
    let v_types = &spatial_description.voxels.types;
    let v_position_mm = &spatial_description.voxels.positions_mm;
    let v_numbers = &spatial_description.voxels.numbers;
    let (x_offset, y_offset, z_offset) = voxel_offset;

    // no self connection allowed
    if x_offset == 0 && y_offset == 0 && z_offset == 0 {
        return Ok(false);
    }
    let (x_out, y_out, z_out) = output_voxel_index;
    let x_out_i32 = i32::try_from(x_out)
        .with_context(|| format!("Output voxel x-coordinate {} exceeds i32::MAX", x_out))?;
    let y_out_i32 = i32::try_from(y_out)
        .with_context(|| format!("Output voxel y-coordinate {} exceeds i32::MAX", y_out))?;
    let z_out_i32 = i32::try_from(z_out)
        .with_context(|| format!("Output voxel z-coordinate {} exceeds i32::MAX", z_out))?;

    let input_voxel_index = [
        x_out_i32 - x_offset,
        y_out_i32 - y_offset,
        z_out_i32 - z_offset,
    ];
    // Skip if the input voxel doesn't exist
    if !spatial_description.voxels.is_valid_index(input_voxel_index) {
        return Ok(false);
    }
    let x_in_usize = usize::try_from(x_out_i32 - x_offset)
        .with_context(|| format!("Input voxel x-coordinate {} cannot be converted to usize", x_out_i32 - x_offset))?;
    let y_in_usize = usize::try_from(y_out_i32 - y_offset)
        .with_context(|| format!("Input voxel y-coordinate {} cannot be converted to usize", y_out_i32 - y_offset))?;
    let z_in_usize = usize::try_from(z_out_i32 - z_offset)
        .with_context(|| format!("Input voxel z-coordinate {} cannot be converted to usize", z_out_i32 - z_offset))?;

    let input_voxel_index = [x_in_usize, y_in_usize, z_in_usize];
    // SKip if the input voxel is already connected
    if activation_time_s[input_voxel_index].is_some() {
        return Ok(false);
    }
    let output_voxel_type = &v_types[output_voxel_index];
    let input_voxel_type = &v_types[input_voxel_index];
    // Skip if connection is not alowed
    if !voxels::is_connection_allowed(output_voxel_type, input_voxel_type) {
        return Ok(false);
    }
    // Skip pathologies if the propagation factor is zero
    if input_voxel_type == &VoxelType::Pathological
        && relative_eq!(config.common.current_factor_in_pathology, 0.0)
    {
        return Ok(false);
    }
    // Now we finally found something that we want to connect.
    let input_state_number = v_numbers[input_voxel_index]
        .with_context(|| format!("Input voxel at {:?} has no assigned number", input_voxel_index))?;
    let output_position_mm = &v_position_mm.slice(s![x_out, y_out, z_out, ..]);
    let [x_in, y_in, z_in] = input_voxel_index;
    let input_position_mm = &v_position_mm.slice(s![x_in, y_in, z_in, ..]);
    let Some(propagation_velocity_m_per_s) = config
        .common
        .propagation_velocities_m_per_s
        .get(input_voxel_type)
    else {
        tracing::warn!("No propagation velocity found for voxel type {:?}", input_voxel_type);
        return Ok(false);
    };
    let delay_s = delay::calculate_delay_s(
        input_position_mm,
        output_position_mm,
        *propagation_velocity_m_per_s,
    );
    // update activation time of input voxel, marking them as connected
    let output_activation_time = activation_time_s[output_voxel_index]
        .with_context(|| format!("Output voxel at {:?} has no activation time", output_voxel_index))?;
    activation_time_s[input_voxel_index] = Some(output_activation_time + delay_s);
    let direction = direction::calculate(input_position_mm, output_position_mm);
    current_directions
        .slice_mut(s![x_in, y_in, z_in, ..])
        .assign(&direction);
    let mut gain = gain::calculate(
        &direction,
        current_directions.slice(s![x_out, y_out, z_out, ..]),
    );
    if *input_voxel_type == VoxelType::Pathological && *output_voxel_type != VoxelType::Pathological
    {
        gain *= config.common.current_factor_in_pathology;
    }
    if *output_voxel_type == VoxelType::Pathological && *input_voxel_type != VoxelType::Pathological
    {
        gain *= 1.0 / config.common.current_factor_in_pathology;
    }
    assign_gain(
        ap_params,
        input_state_number,
        x_offset,
        y_offset,
        z_offset,
        &gain,
    );
    Ok(true)
}

/// Assigns the given gain values to the appropriate indices in the
/// all-pass filter parameter gains array. Maps the gain values from the
/// (`input_dim`, `output_dim`) coordinate space to the flattened 22D gains array
/// using the provided state number and offset indices.
#[tracing::instrument(level = "trace")]
fn assign_gain(
    ap_params: &mut APParameters,
    input_state_number: usize,
    x_offset: i32,
    y_offset: i32,
    z_offset: i32,
    gain: &ndarray::ArrayBase<ndarray::OwnedRepr<f32>, Dim<[usize; 2]>>,
) {
    trace!(
        "Assigning gain {:?} to input state number {}",
        gain,
        input_state_number
    );
    for input_dimension in 0..3 {
        for output_dimension in 0..3 {
            ap_params.gains[(
                input_state_number + input_dimension,
                offset_to_gain_index(x_offset, y_offset, z_offset, output_dimension)
                    .expect("Offsets to be valid"),
            )] = gain[(input_dimension, output_dimension)];
        }
    }
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

/// Finds candidate voxels that are activated at the given `current_time_s`.
///
/// Filters the `activation_time_s` array for voxels with activation time
/// equal to `current_time_s`, returning a vector of their indices.
#[tracing::instrument(level = "trace")]
fn find_candidate_voxels(
    activation_time_s: &ndarray::ArrayBase<ndarray::OwnedRepr<Option<f32>>, Dim<[usize; 3]>>,
    current_time_s: f32,
) -> Vec<(usize, usize, usize)> {
    trace!("Finding candidate voxels at time {}", current_time_s);
    let output_voxel_indices: Vec<(usize, usize, usize)> = activation_time_s
        .indexed_iter()
        .filter_map(|(index, &time_s)| {
            time_s.filter(|&t| relative_eq!(t, current_time_s)).map(|_| index)
        })
        .collect();
    output_voxel_indices
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
mod test {
    use approx::assert_relative_eq;

    use crate::core::model::functional::allpass::{
        from_samples_to_coef, from_samples_to_usize, offset_to_gain_index,
    };

    #[test]
    fn from_samples_to_usize_1() {
        assert_eq!(1, from_samples_to_usize(1.0));
        assert_eq!(1, from_samples_to_usize(1.2));
        assert_eq!(10, from_samples_to_usize(10.9));
        assert_eq!(10, from_samples_to_usize(10.0));
    }

    #[test]
    fn from_samples_to_coef_1() {
        assert_relative_eq!(1.0 / 3.0, from_samples_to_coef(0.5));
        assert_relative_eq!(1.0 / 3.0, from_samples_to_coef(1.5));
        assert_relative_eq!(1.0 / 3.0, from_samples_to_coef(99999.5));

        assert_relative_eq!(0.9999, from_samples_to_coef(0.0));
        assert_relative_eq!(0.9999, from_samples_to_coef(1.0));
        assert_relative_eq!(0.9999, from_samples_to_coef(99999.0));
    }

    #[test]
    fn offset_to_index_test() {
        let desired = 2;
        let actual = offset_to_gain_index(-1, -1, -1, 2).expect("Offsets to be valid.");
        assert_eq!(desired, actual);

        let desired = 5;
        let actual = offset_to_gain_index(-1, -1, 0, 2).expect("Offsets to be valid.");
        assert_eq!(desired, actual);

        let desired = 8;
        let actual = offset_to_gain_index(-1, -1, 1, 2).expect("Offsets to be valid.");
        assert_eq!(desired, actual);

        let desired = 77;
        let actual = offset_to_gain_index(1, 1, 1, 2).expect("Offsets to be valid.");
        assert_eq!(desired, actual);

        let desired = 42;
        let actual = offset_to_gain_index(0, 1, -1, 0).expect("Offsets to be valid.");
        assert_eq!(desired, actual);

        let desired = 45;
        let actual = offset_to_gain_index(0, 1, 0, 0).expect("Offsets to be valid.");
        assert_eq!(desired, actual);

        let desired = 60;
        let actual = offset_to_gain_index(1, 0, -1, 0).expect("Offsets to be valid.");
        assert_eq!(desired, actual);

        let desired = 63;
        let actual = offset_to_gain_index(1, 0, 0, 0).expect("Offsets to be valid.");
        assert_eq!(desired, actual);
    }
}
