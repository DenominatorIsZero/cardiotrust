mod delay;
mod direction;
mod gain;
pub mod shapes;

use std::error::Error;

use approx::relative_eq;
use itertools::Itertools;
use ndarray::{arr1, s, Array1, Array3, Array4, Dim};
use ndarray_stats::QuantileExt;
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
    ) -> Result<Self, Box<dyn Error>> {
        debug!("Creating AP parameters from model config");
        let mut ap_params = Self::empty(
            spatial_description.voxels.count_states(),
            spatial_description.voxels.types.raw_dim(),
        );

        connect_voxels(spatial_description, config, &mut ap_params);

        let delays_samples = calculate_delay_samples_array(
            spatial_description,
            &config.common.propagation_velocities_m_per_s,
            sample_rate_hz,
        )?;

        ap_params.output_state_indices = init_output_state_indicies(spatial_description);

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
    #[tracing::instrument(level = "debug")]
    pub(crate) fn save_npy(&self, path: &std::path::Path) {
        debug!("Saving allpass parameters to npy");
        let path = &path.join("allpass");
        self.gains.save_npy(path, "gains.npy");
        self.output_state_indices.save_npy(path);
        self.coefs.save_npy(path);
        self.delays.save_npy(path);
        self.activation_time_ms.save_npy(path);
    }
}

/// Initializes the output state indices for the allpass filter based on the
/// spatial description. It finds neighboring output voxels for each input
/// voxel and maps the input states to the corresponding output states. This
/// allows signals to propagate from input voxels to neighboring output voxels
/// through the allpass filter.
#[tracing::instrument(level = "debug", skip_all)]
fn init_output_state_indicies(spatial_description: &SpatialDescription) -> Indices {
    debug!("Initializing output state indices");
    let mut output_state_indices = Indices::empty(spatial_description.voxels.count_states());
    let v_types = &spatial_description.voxels.types;
    let v_numbers = &spatial_description.voxels.numbers;
    // TODO: write tests
    v_types
        .indexed_iter()
        .filter(|(_, v_type)| v_type.is_connectable())
        .for_each(|(input_voxel_index, _)| {
            let (x_in, y_in, z_in) = input_voxel_index;
            for ((x_offset, y_offset), z_offset) in
                (-1..=1).cartesian_product(-1..=1).cartesian_product(-1..=1)
            {
                if x_offset == 0 && y_offset == 0 && z_offset == 0 {
                    continue;
                }
                let ouput_voxel_index_candidate = [
                    i32::try_from(x_in).unwrap() + x_offset,
                    i32::try_from(y_in).unwrap() + y_offset,
                    i32::try_from(z_in).unwrap() + z_offset,
                ];
                if !spatial_description
                    .voxels
                    .is_valid_index(ouput_voxel_index_candidate)
                {
                    continue;
                }
                let output_voxel_index = [
                    usize::try_from(ouput_voxel_index_candidate[0]).unwrap(),
                    usize::try_from(ouput_voxel_index_candidate[1]).unwrap(),
                    usize::try_from(ouput_voxel_index_candidate[2]).unwrap(),
                ];
                for input_direction in 0..3 {
                    let input_state_number =
                        v_numbers[input_voxel_index].unwrap() + input_direction;
                    for output_dimension in 0..3 {
                        let gain_index =
                            offset_to_gain_index(x_offset, y_offset, z_offset, output_dimension)
                                .unwrap();
                        let output_state_index =
                            v_numbers[output_voxel_index].unwrap() + output_dimension;
                        output_state_indices[(input_state_number, gain_index)] =
                            Some(output_state_index);
                    }
                }
            }
        });
    output_state_indices
}

/// Connects voxels in the model based on voxel type and proximity.
/// Iteratively activates voxels by updating `activation_time_s` and `current_directions`.
/// Stops when no more voxels can be connected at the current time step.
#[tracing::instrument(level = "debug", skip_all)]
fn connect_voxels(
    spatial_description: &SpatialDescription,
    config: &Model,
    ap_params: &mut APParameters,
) {
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
            .filter(|time_s| time_s.is_some())
            .any(|time_s| time_s.unwrap() > current_time_s)
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
                        );
                    }
                }
            }
        }
        let candidate_times_s: Vec<f32> = activation_time_s
            .iter()
            .filter(|t| t.is_some() && t.unwrap() > current_time_s)
            .map(|t| t.unwrap())
            .collect();
        let candidate_times_s = Array1::from_vec(candidate_times_s);
        current_time_s = *candidate_times_s.min_skipnan();
    }
    ap_params
        .activation_time_ms
        .iter_mut()
        .zip(activation_time_s)
        .filter(|(_, s)| s.is_some())
        .for_each(|(ms, s)| *ms = Some(s.unwrap() * 1000.0));
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
) -> bool {
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
        return false;
    }
    let (x_out, y_out, z_out) = output_voxel_index;
    let input_voxel_index = [
        i32::try_from(x_out).unwrap() - x_offset,
        i32::try_from(y_out).unwrap() - y_offset,
        i32::try_from(z_out).unwrap() - z_offset,
    ];
    // Skip if the input voxel doesn't exist
    if !spatial_description.voxels.is_valid_index(input_voxel_index) {
        return false;
    }
    let input_voxel_index = [
        usize::try_from(i32::try_from(x_out).unwrap() - x_offset).unwrap(),
        usize::try_from(i32::try_from(y_out).unwrap() - y_offset).unwrap(),
        usize::try_from(i32::try_from(z_out).unwrap() - z_offset).unwrap(),
    ];
    // SKip if the input voxel is already connected
    if activation_time_s[input_voxel_index].is_some() {
        return false;
    }
    let output_voxel_type = &v_types[output_voxel_index];
    let input_voxel_type = &v_types[input_voxel_index];
    // Skip if connection is not alowed
    if !voxels::is_connection_allowed(output_voxel_type, input_voxel_type) {
        return false;
    }
    // Skip pathologies if the propagation factor is zero
    if input_voxel_type == &VoxelType::Pathological
        && relative_eq!(config.common.current_factor_in_pathology, 0.0)
    {
        return false;
    }
    // Now we finally found something that we want to connect.
    let input_state_number = v_numbers[input_voxel_index].unwrap();
    let output_position_mm = &v_position_mm.slice(s![x_out, y_out, z_out, ..]);
    let [x_in, y_in, z_in] = input_voxel_index;
    let input_position_mm = &v_position_mm.slice(s![x_in, y_in, z_in, ..]);
    let propagation_velocity_m_per_s = config
        .common
        .propagation_velocities_m_per_s
        .get(input_voxel_type)
        .unwrap();
    let delay_s = delay::calculate_delay_s(
        input_position_mm,
        output_position_mm,
        *propagation_velocity_m_per_s,
    );
    // update activation time of input voxel, marking them as connected
    activation_time_s[input_voxel_index] =
        Some(activation_time_s[output_voxel_index].unwrap() + delay_s);
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
    true
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
        .filter(|(_, time_s)| time_s.is_some() && relative_eq!(time_s.unwrap(), current_time_s))
        .map(|(index, _)| index)
        .collect();
    output_voxel_indices
}

/// Converts a sample value in the range to the corresponding
/// all-pass filter coefficient.
#[tracing::instrument(level = "trace")]
fn from_samples_to_coef(samples: f32) -> f32 {
    trace!("Converting {} samples to coefficient", samples);
    let fractional = samples % 1.0;
    (1.0 - fractional) / (1.0 + fractional)
}

/// Computes the integer part of the given samples value.
#[allow(
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation
)]
#[must_use]
const fn from_samples_to_usize(samples: f32) -> usize {
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

        assert_relative_eq!(1.0, from_samples_to_coef(0.0));
        assert_relative_eq!(1.0, from_samples_to_coef(1.0));
        assert_relative_eq!(1.0, from_samples_to_coef(99999.0));
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
