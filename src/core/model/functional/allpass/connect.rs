use anyhow::{Context, Result};
use approx::relative_eq;
use ndarray::{arr1, s, Array3, Array4, Dim};
use ndarray_stats::QuantileExt;
use tracing::{debug, trace};

use super::APParameters;
use crate::core::{
    config::model::Model,
    model::{
        functional::allpass::delay,
        spatial::{
            voxels::{self, VoxelType},
            SpatialDescription,
        },
    },
};

/// Connects voxels in the model based on voxel type and proximity.
/// Iteratively activates voxels by updating `activation_time_s` and `current_directions`.
/// Stops when no more voxels can be connected at the current time step.
#[tracing::instrument(level = "debug", skip_all)]
pub(super) fn connect_voxels(
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
                        )
                        .unwrap_or_else(|e| {
                            tracing::error!("Connection failed: {}", e);
                            false
                        });
                    }
                }
            }
        }
        let candidate_times_s: Vec<f32> = activation_time_s
            .iter()
            .filter_map(|&t| t)
            .filter(|&t| t > current_time_s)
            .collect();
        let candidate_times_s = ndarray::Array1::from_vec(candidate_times_s);
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
pub(super) fn try_to_connect(
    voxel_offset: (i32, i32, i32),
    output_voxel_index: (usize, usize, usize),
    spatial_description: &SpatialDescription,
    activation_time_s: &mut ndarray::ArrayBase<ndarray::OwnedRepr<Option<f32>>, Dim<[usize; 3]>>,
    config: &Model,
    current_directions: &mut ndarray::ArrayBase<ndarray::OwnedRepr<f32>, Dim<[usize; 4]>>,
    ap_params: &mut APParameters,
) -> Result<bool> {
    use super::{direction, gain};
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
        .with_context(|| format!("Output voxel x-coordinate {x_out} exceeds i32::MAX"))?;
    let y_out_i32 = i32::try_from(y_out)
        .with_context(|| format!("Output voxel y-coordinate {y_out} exceeds i32::MAX"))?;
    let z_out_i32 = i32::try_from(z_out)
        .with_context(|| format!("Output voxel z-coordinate {z_out} exceeds i32::MAX"))?;

    let input_voxel_index = [
        x_out_i32 - x_offset,
        y_out_i32 - y_offset,
        z_out_i32 - z_offset,
    ];
    // Skip if the input voxel doesn't exist
    if !spatial_description.voxels.is_valid_index(input_voxel_index) {
        return Ok(false);
    }
    let x_in_usize = usize::try_from(x_out_i32 - x_offset).with_context(|| {
        format!(
            "Input voxel x-coordinate {} cannot be converted to usize",
            x_out_i32 - x_offset
        )
    })?;
    let y_in_usize = usize::try_from(y_out_i32 - y_offset).with_context(|| {
        format!(
            "Input voxel y-coordinate {} cannot be converted to usize",
            y_out_i32 - y_offset
        )
    })?;
    let z_in_usize = usize::try_from(z_out_i32 - z_offset).with_context(|| {
        format!(
            "Input voxel z-coordinate {} cannot be converted to usize",
            z_out_i32 - z_offset
        )
    })?;

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
        .with_context(|| format!("Input voxel at {input_voxel_index:?} has no assigned number"))?;
    let output_position_mm = &v_position_mm.slice(s![x_out, y_out, z_out, ..]);
    let [x_in, y_in, z_in] = input_voxel_index;
    let input_position_mm = &v_position_mm.slice(s![x_in, y_in, z_in, ..]);
    let propagation_velocity_m_per_s = config.common.propagation_velocities.get(*input_voxel_type);
    let delay_s = delay::calculate_delay_s(
        input_position_mm,
        output_position_mm,
        propagation_velocity_m_per_s,
    );
    // update activation time of input voxel, marking them as connected
    let output_activation_time = activation_time_s[output_voxel_index].with_context(|| {
        format!("Output voxel at {output_voxel_index:?} has no activation time")
    })?;
    activation_time_s[input_voxel_index] = Some(output_activation_time + delay_s);
    let direction = direction::calculate(input_position_mm, output_position_mm);
    current_directions
        .slice_mut(s![x_in, y_in, z_in, ..])
        .assign(&direction);
    let mut gain_val = gain::calculate(
        &direction,
        current_directions.slice(s![x_out, y_out, z_out, ..]),
    );
    if *input_voxel_type == VoxelType::Pathological && *output_voxel_type != VoxelType::Pathological
    {
        gain_val *= config.common.current_factor_in_pathology;
    }
    if *output_voxel_type == VoxelType::Pathological && *input_voxel_type != VoxelType::Pathological
    {
        gain_val *= 1.0 / config.common.current_factor_in_pathology;
    }
    assign_gain(
        ap_params,
        input_state_number,
        x_offset,
        y_offset,
        z_offset,
        &gain_val,
    );
    Ok(true)
}

/// Assigns the given gain values to the appropriate indices in the
/// all-pass filter parameter gains array.
#[tracing::instrument(level = "trace")]
pub(super) fn assign_gain(
    ap_params: &mut APParameters,
    input_state_number: usize,
    x_offset: i32,
    y_offset: i32,
    z_offset: i32,
    gain: &ndarray::ArrayBase<ndarray::OwnedRepr<f32>, Dim<[usize; 2]>>,
) {
    use super::offset_to_gain_index;
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

/// Finds candidate voxels that are activated at the given `current_time_s`.
#[tracing::instrument(level = "trace")]
pub(super) fn find_candidate_voxels(
    activation_time_s: &ndarray::ArrayBase<ndarray::OwnedRepr<Option<f32>>, Dim<[usize; 3]>>,
    current_time_s: f32,
) -> Vec<(usize, usize, usize)> {
    trace!("Finding candidate voxels at time {}", current_time_s);
    let output_voxel_indices: Vec<(usize, usize, usize)> = activation_time_s
        .indexed_iter()
        .filter_map(|(index, &time_s)| {
            time_s
                .filter(|&t| relative_eq!(t, current_time_s))
                .map(|_| index)
        })
        .collect();
    output_voxel_indices
}
