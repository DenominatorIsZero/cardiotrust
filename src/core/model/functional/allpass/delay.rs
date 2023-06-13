use std::{collections::HashMap, error::Error};

use itertools::Itertools;
use ndarray::{s, ArrayBase, Dim, ViewRepr};

use crate::core::model::spatial::{voxels::VoxelType, SpatialDescription};

use super::shapes::ArrayDelays;

pub fn calculate_delay_s(
    input_position_mm: &ArrayBase<ViewRepr<&f32>, Dim<[usize; 1]>>,
    output_position_mm: &ArrayBase<ViewRepr<&f32>, Dim<[usize; 1]>>,
    propagation_velocity_m_per_s: &f32,
) -> f32 {
    let distance_m = (input_position_mm - output_position_mm) / 1000.0;
    let distance_norm_m = distance_m.mapv(|v| v.powi(2)).sum().sqrt();
    distance_norm_m / *propagation_velocity_m_per_s
}

pub fn calculate_delay_samples_array(
    spatial_description: &SpatialDescription,
    propagation_velocities_m_per_s: &HashMap<VoxelType, f32>,
    sample_rate_hz: f32,
) -> Result<ArrayDelays<f32>, Box<dyn Error>> {
    let mut delays_samples = ArrayDelays::<f32>::empty(spatial_description.voxels.count_states());

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
        for ((x_offset, y_offset), z_offset) in
            (-1..=1).cartesian_product(-1..=1).cartesian_product(-1..=1)
        {
            if x_offset == 0 && y_offset == 0 && z_offset == 0 {
                continue;
            }
            let ouput_voxel_index = [
                x_in as i32 + x_offset,
                y_in as i32 + y_offset,
                z_in as i32 + z_offset,
            ];
            if !spatial_description.voxels.is_valid_index(ouput_voxel_index) {
                continue;
            }
            let [x_out, y_out, z_out] = [
                (x_in as i32 + x_offset) as usize,
                (y_in as i32 + y_offset) as usize,
                (z_in as i32 + z_offset) as usize,
            ];
            let output_position_mm = &v_position_mm.slice(s![x_out, y_out, z_out, ..]);

            let delay_s = calculate_delay_s(
                input_position_mm,
                output_position_mm,
                propagation_velocities_m_per_s.get(v_type).unwrap(),
            );
            let delay_samples = delay_s * sample_rate_hz;

            if delay_samples < 1.0 {
                return Err(format!(
                    "Can not configure delays below 1 sample.\
                        Calculated delay: {:?}.\
                        For voxel type: {:?}",
                    delay_samples, v_type
                )
                .into());
            }

            delays_samples.values[(
                v_numbers[input_voxel_index].unwrap() / 3,
                (1 + x_offset) as usize,
                (1 + y_offset) as usize,
                (1 + z_offset) as usize,
            )] = delay_samples;
        }
    }
    Ok(delays_samples)
}

#[cfg(test)]
mod test {
    use approx::assert_relative_eq;
    use ndarray::{arr1, Array1};

    use super::calculate_delay_s;

    #[test]
    fn calculate_delay_s_1() {
        let input_position_mm: Array1<f32> = arr1(&[1000.0, 0.0, 0.0]);
        let output_position_mm: Array1<f32> = arr1(&[2000.0, 0.0, 0.0]);
        let propagation_velocity_m_per_s = 2.0;

        let delay_s = calculate_delay_s(
            &input_position_mm.view(),
            &output_position_mm.view(),
            &propagation_velocity_m_per_s,
        );

        assert_relative_eq!(delay_s, 0.5)
    }

    #[test]
    fn calculate_delay_s_2() {
        let input_position_mm: Array1<f32> = arr1(&[1000.0, 0.0, 0.0]);
        let output_position_mm: Array1<f32> = arr1(&[4000.0, 4000.0, 0.0]);
        let propagation_velocity_m_per_s = 2.0;

        let delay_s = calculate_delay_s(
            &input_position_mm.view(),
            &output_position_mm.view(),
            &propagation_velocity_m_per_s,
        );

        assert_relative_eq!(delay_s, 2.5)
    }
}
