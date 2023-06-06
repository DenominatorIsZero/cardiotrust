use approx::relative_eq;
use itertools::izip;
use ndarray::{arr1, arr3, s, Array1, Array2, Array3, Array4, ArrayBase, Dim, ViewRepr};

use crate::core::{
    config::model::Model,
    model::spatial::{voxels::VoxelType, SpatialDescription},
};

use self::shapes::{ArrayActivationTime, ArrayDelays, ArrayGains, ArrayIndicesGains};

pub mod shapes;

#[derive(Debug, PartialEq)]
pub struct APParameters {
    pub gains: ArrayGains<f32>,
    pub output_state_indices: ArrayIndicesGains,
    pub coefs: ArrayDelays<f32>,
    pub delays: ArrayDelays<usize>,
    pub activation_time_ms: ArrayActivationTime,
}

impl APParameters {
    pub fn empty(number_of_states: usize) -> APParameters {
        APParameters {
            gains: ArrayGains::empty(number_of_states),
            output_state_indices: ArrayIndicesGains::empty(number_of_states),
            coefs: ArrayDelays::empty(number_of_states),
            delays: ArrayDelays::empty(number_of_states),
            activation_time_ms: ArrayActivationTime::empty([number_of_states, 1, 1]),
        }
    }

    pub fn from_model_config(
        config: &Model,
        spatial_description: &SpatialDescription,
        sample_rate_hz: f32,
    ) -> Result<APParameters, String> {
        let mut ap_params = APParameters::empty(spatial_description.voxels.count_states());
        let mut activation_time_s = Array3::<Option<f32>>::from_elem(
            spatial_description.voxels.types.values.raw_dim(),
            None,
        );
        let mut current_directions =
            Array4::<f32>::zeros(spatial_description.voxels.positions_mm.values.raw_dim());
        let mut delays_samples =
            ArrayDelays::<f32>::empty(spatial_description.voxels.count_states());

        let v_types = &spatial_description.voxels.types.values;
        let v_position_mm = &spatial_description.voxels.positions_mm.values;
        let v_numbers = &spatial_description.voxels.numbers.values;

        // Fill the delays_samples tensor
        for (input_voxel_index, v_type) in v_types.indexed_iter() {
            if *v_type == VoxelType::None {
                continue;
            }
            let (x_in, y_in, z_in) = input_voxel_index;
            let input_position_mm = &v_position_mm.slice(s![x_in, y_in, z_in, ..]);
            for (x_offset, y_offset, z_offset) in izip!([-1, 0, 1], [-1, 0, 1], [-1, 0, 1]) {
                if x_offset == 0 && y_offset == 0 && z_offset == 0 {
                    continue;
                }
                let ouput_voxel_index = [
                    x_in as i32 - x_offset,
                    y_in as i32 - y_offset,
                    z_in as i32 - z_offset,
                ];
                if !spatial_description.voxels.is_valid_index(ouput_voxel_index) {
                    continue;
                }
                let [x_out, y_out, z_out] = [
                    (x_in as i32 - x_offset) as usize,
                    (y_in as i32 - y_offset) as usize,
                    (z_in as i32 - z_offset) as usize,
                ];
                let output_position_mm = &v_position_mm.slice(s![x_out, y_out, z_out, ..]);

                let delay_s = calculate_delay_s(
                    input_position_mm,
                    output_position_mm,
                    config.propagation_velocities_m_per_s.get(v_type).unwrap(),
                );
                let delay_samples = delay_s * sample_rate_hz;

                if delay_samples < 1.0 {
                    return Err(format!(
                        "Can not configure delays below 1 sample.\
                        Calculated delay: {:?}.\
                        For voxel type: {:?}",
                        delay_samples, v_type
                    ));
                }

                // TODO: Check if these indices are correct. Crosscheck with algorithm implementation.
                delays_samples.values[(
                    v_numbers[input_voxel_index].unwrap() / 3,
                    (1 + x_offset) as usize,
                    (1 + y_offset) as usize,
                    (1 + z_offset) as usize,
                )] = delay_samples;
            }
        }

        let current_time_s: f32 = 0.0;

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
            // have to check the activation times because there might be come connection possible
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
            let output_voxel_indices: Vec<(usize, usize, usize)> = activation_time_s
                .indexed_iter()
                .filter(|(_, time_s)| {
                    time_s.is_some() && relative_eq!(time_s.unwrap(), current_time_s)
                })
                .map(|(index, _)| index)
                .collect();

            for output_voxel_index in output_voxel_indices {
                let output_state_number = v_numbers[output_voxel_index].unwrap();
                for (x_offset, y_offset, z_offset) in izip!([-1, 0, 1], [-1, 0, 1], [-1, 0, 1]) {
                    // no self connection allowed
                    if x_offset == 0 && y_offset == 0 && z_offset == 0 {
                        continue;
                    }
                    let (x_out, y_out, z_out) = output_voxel_index;
                    let input_voxel_index = [
                        x_out as i32 + x_offset,
                        y_out as i32 + y_offset,
                        z_out as i32 + z_offset,
                    ];
                    // Skip if the input voxel doesn't exist
                    if !spatial_description.voxels.is_valid_index(input_voxel_index) {
                        continue;
                    }
                    let input_voxel_index = [
                        (x_out as i32 + x_offset) as usize,
                        (y_out as i32 + y_offset) as usize,
                        (z_out as i32 + z_offset) as usize,
                    ];
                    // SKip if the input voxel is already connected
                    if activation_time_s[input_voxel_index].is_some() {
                        continue;
                    }
                    // Skip if connection is not allowed
                    todo!();
                }
            }
        }

        ap_params
            .delays
            .values
            .iter_mut()
            .zip(delays_samples.values.iter())
            .for_each(|(delay, samples)| *delay = *samples as usize);

        ap_params
            .coefs
            .values
            .iter_mut()
            .zip(delays_samples.values.iter())
            .for_each(|(coef, samples)| {
                let fractional = *samples % 1.0;
                *coef = (1.0 - fractional) / (1.0 + fractional)
            });

        Ok(ap_params)
    }
}

fn calculate_delay_s(
    input_position_mm: &ArrayBase<ViewRepr<&f32>, Dim<[usize; 1]>>,
    output_position_mm: &ArrayBase<ViewRepr<&f32>, Dim<[usize; 1]>>,
    propagation_velocity_m_per_s: &f32,
) -> f32 {
    let distance_m = (input_position_mm - output_position_mm) / 1000.0;
    let distance_m = distance_m.mapv(|v| v.powi(2)).sum().sqrt();
    distance_m / *propagation_velocity_m_per_s
}
