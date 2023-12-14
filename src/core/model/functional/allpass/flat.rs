use std::error::Error;

use approx::relative_eq;
use itertools::Itertools;
use ndarray::{arr1, s, Array1, Array3, Array4, Dim};

use ndarray_stats::QuantileExt;
use serde::{Deserialize, Serialize};

use crate::core::{
    config::model::Model,
    model::spatial::{
        voxels::{self, VoxelType},
        SpatialDescription,
    },
};

use super::{
    delay::{self, calculate_delay_samples_array_flat},
    direction, find_candidate_voxels, from_samples_to_coef, from_samples_to_usize, gain,
    shapes::{
        flat::{ArrayDelaysFlat, ArrayGainsFlat, ArrayIndicesGainsFlat},
        ArrayActivationTime,
    },
};
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct APParametersFlat {
    pub gains: ArrayGainsFlat<f32>,
    pub output_state_indices: ArrayIndicesGainsFlat,
    pub coefs: ArrayDelaysFlat<f32>,
    pub delays: ArrayDelaysFlat<usize>,
    pub activation_time_ms: ArrayActivationTime,
}

impl APParametersFlat {
    #[must_use]
    pub fn empty(number_of_states: usize, voxels_in_dims: Dim<[usize; 3]>) -> Self {
        Self {
            gains: ArrayGainsFlat::empty(number_of_states),
            output_state_indices: ArrayIndicesGainsFlat::empty(number_of_states),
            coefs: ArrayDelaysFlat::empty(number_of_states),
            delays: ArrayDelaysFlat::empty(number_of_states),
            activation_time_ms: ArrayActivationTime::empty(voxels_in_dims),
        }
    }

    /// .
    ///
    /// # Errors
    ///
    /// This function will return an error if model cant be build with
    /// given config.
    pub fn from_model_config(
        config: &Model,
        spatial_description: &SpatialDescription,
        sample_rate_hz: f32,
    ) -> Result<Self, Box<dyn Error>> {
        let mut ap_params = Self::empty(
            spatial_description.voxels.count_states(),
            spatial_description.voxels.types.values.raw_dim(),
        );

        connect_voxels(spatial_description, config, &mut ap_params);

        let delays_samples = calculate_delay_samples_array_flat(
            spatial_description,
            &config.propagation_velocities_m_per_s,
            sample_rate_hz,
        )?;

        ap_params.output_state_indices = init_output_state_indicies(spatial_description);

        ap_params
            .delays
            .values
            .iter_mut()
            .zip(delays_samples.values.iter())
            .for_each(|(delay, samples)| *delay = from_samples_to_usize(*samples));

        ap_params
            .coefs
            .values
            .iter_mut()
            .zip(delays_samples.values.iter())
            .for_each(|(coef, samples)| *coef = from_samples_to_coef(*samples));

        Ok(ap_params)
    }

    pub(crate) fn save_npy(&self, path: &std::path::Path) {
        let path = &path.join("allpass");
        self.gains.save_npy(path, "gains.npy");
        self.output_state_indices.save_npy(path);
        self.coefs.save_npy(path);
        self.delays.save_npy(path);
        self.activation_time_ms.save_npy(path);
    }
}

fn init_output_state_indicies(spatial_description: &SpatialDescription) -> ArrayIndicesGainsFlat {
    let mut output_state_indices =
        ArrayIndicesGainsFlat::empty(spatial_description.voxels.count_states());
    let v_types = &spatial_description.voxels.types.values;
    let v_numbers = &spatial_description.voxels.numbers.values;
    // TODO: write tests
    v_types
        .indexed_iter()
        .filter(|(_, v_type)| **v_type != VoxelType::None)
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
                        let output_state_index =
                            v_numbers[output_voxel_index].unwrap() + output_dimension;
                        output_state_indices.values[(
                            input_state_number,
                            offset_to_gain_index(x_offset, y_offset, z_offset, output_dimension)
                                .expect("Not all offsets to be zero."),
                        )] = Some(output_state_index);
                    }
                }
            }
        });
    output_state_indices
}

fn connect_voxels(
    spatial_description: &SpatialDescription,
    config: &Model,
    ap_params: &mut APParametersFlat,
) {
    let mut activation_time_s =
        Array3::<Option<f32>>::from_elem(spatial_description.voxels.types.values.raw_dim(), None);
    let mut current_directions =
        Array4::<f32>::zeros(spatial_description.voxels.positions_mm.values.raw_dim());

    let v_types = &spatial_description.voxels.types.values;

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
        .values
        .iter_mut()
        .zip(activation_time_s)
        .filter(|(_, s)| s.is_some())
        .for_each(|(ms, s)| *ms = Some(s.unwrap() * 1000.0));
}

fn try_to_connect(
    voxel_offset: (i32, i32, i32),
    output_voxel_index: (usize, usize, usize),
    spatial_description: &SpatialDescription,
    activation_time_s: &mut ndarray::ArrayBase<ndarray::OwnedRepr<Option<f32>>, Dim<[usize; 3]>>,
    config: &Model,
    current_directions: &mut ndarray::ArrayBase<ndarray::OwnedRepr<f32>, Dim<[usize; 4]>>,
    ap_params: &mut APParametersFlat,
) -> bool {
    let v_types = &spatial_description.voxels.types.values;
    let v_position_mm = &spatial_description.voxels.positions_mm.values;
    let v_numbers = &spatial_description.voxels.numbers.values;
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
        && relative_eq!(config.current_factor_in_pathology, 0.0)
    {
        return false;
    }
    // Now we finally found something that we want to connect.
    let input_state_number = v_numbers[input_voxel_index].unwrap();
    let output_position_mm = &v_position_mm.slice(s![x_out, y_out, z_out, ..]);
    let [x_in, y_in, z_in] = input_voxel_index;
    let input_position_mm = &v_position_mm.slice(s![x_in, y_in, z_in, ..]);
    let propagation_velocity_m_per_s = config
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
        gain *= config.current_factor_in_pathology;
    }
    if *output_voxel_type == VoxelType::Pathological && *input_voxel_type != VoxelType::Pathological
    {
        gain *= 1.0 / config.current_factor_in_pathology;
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

fn assign_gain(
    ap_params: &mut APParametersFlat,
    input_state_number: usize,
    x_offset: i32,
    y_offset: i32,
    z_offset: i32,
    gain: &ndarray::ArrayBase<ndarray::OwnedRepr<f32>, Dim<[usize; 2]>>,
) {
    for input_dimension in 0..3 {
        for output_dimension in 0..3 {
            ap_params.gains.values[(
                input_state_number + input_dimension,
                offset_to_gain_index(x_offset, y_offset, z_offset, output_dimension)
                    .expect("Offsets to be valid"),
            )] = gain[(input_dimension, output_dimension)];
        }
    }
}

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
    let correction = if x_offset >= 0 && y_offset >= 0 && z_offset >= 0 {
        3
    } else {
        0
    };
    Some(
        output_dimension
            + (z_offset + 1) as usize * 3
            + (y_offset + 1) as usize * 9
            + (x_offset + 1) as usize * 27
            - correction,
    )
}

#[allow(clippy::cast_sign_loss)]
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

    return Some([x_offset, y_offset, z_offset, output_dimension]);
}

#[allow(clippy::cast_sign_loss)]
#[must_use]
pub const fn offset_to_delay_index(x_offset: i32, y_offset: i32, z_offset: i32) -> Option<usize> {
    if x_offset == 0 && y_offset == 0 && z_offset == 0 {
        return None;
    }
    let correction = if x_offset >= 0 && y_offset >= 0 && z_offset >= 0 {
        1
    } else {
        0
    };
    Some(
        (z_offset + 1) as usize + (y_offset + 1) as usize * 3 + (x_offset + 1) as usize * 9
            - correction,
    )
}

#[cfg(test)]
mod test {
    use crate::core::model::functional::allpass::flat::offset_to_gain_index;

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
    }
}
