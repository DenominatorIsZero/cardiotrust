use std::{collections::HashMap, error::Error};

use itertools::Itertools;
use ndarray::{s, ArrayBase, Dim, ViewRepr};

use crate::core::model::spatial::{voxels::VoxelType, SpatialDescription};

use super::shapes::normal::ArrayDelaysNormal;

pub fn calculate_delay_s(
    input_position_mm: &ArrayBase<ViewRepr<&f32>, Dim<[usize; 1]>>,
    output_position_mm: &ArrayBase<ViewRepr<&f32>, Dim<[usize; 1]>>,
    propagation_velocity_m_per_s: f32,
) -> f32 {
    let distance_m = (input_position_mm - output_position_mm) / 1000.0;
    let distance_norm_m = distance_m.mapv(|v| v.powi(2)).sum().sqrt();
    distance_norm_m / propagation_velocity_m_per_s
}

pub fn calculate_delay_samples_array(
    spatial_description: &SpatialDescription,
    propagation_velocities_m_per_s: &HashMap<VoxelType, f32>,
    sample_rate_hz: f32,
) -> Result<ArrayDelaysNormal<f32>, Box<dyn Error>> {
    let mut delay_samples_array =
        ArrayDelaysNormal::<f32>::empty(spatial_description.voxels.count_states());

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
                i32::try_from(x_in).unwrap() + x_offset,
                i32::try_from(y_in).unwrap() + y_offset,
                i32::try_from(z_in).unwrap() + z_offset,
            ];
            if !spatial_description.voxels.is_valid_index(ouput_voxel_index) {
                continue;
            }
            let [x_out, y_out, z_out] = [
                usize::try_from(i32::try_from(x_in).unwrap() + x_offset).unwrap(),
                usize::try_from(i32::try_from(y_in).unwrap() + y_offset).unwrap(),
                usize::try_from(i32::try_from(z_in).unwrap() + z_offset).unwrap(),
            ];
            let output_position_mm = &v_position_mm.slice(s![x_out, y_out, z_out, ..]);

            let delay_s = calculate_delay_s(
                input_position_mm,
                output_position_mm,
                *propagation_velocities_m_per_s.get(v_type).unwrap(),
            );
            let delay_samples = delay_s * sample_rate_hz;

            if delay_samples < 1.0 {
                return Err(format!(
                    "Can not configure delays below 1 sample.\
                        Calculated delay: {delay_samples}.\
                        For voxel type: {v_type:?}",
                )
                .into());
            }

            delay_samples_array.values[(
                v_numbers[input_voxel_index].unwrap() / 3,
                usize::try_from(1 + x_offset).unwrap(),
                usize::try_from(1 + y_offset).unwrap(),
                usize::try_from(1 + z_offset).unwrap(),
            )] = delay_samples;
        }
    }
    Ok(delay_samples_array)
}

#[cfg(test)]
mod test {
    use approx::assert_relative_eq;
    use ndarray::{arr1, Array1};
    use ndarray_stats::QuantileExt;

    use crate::core::{
        config::model::Model,
        model::spatial::{voxels::VoxelType, SpatialDescription},
    };

    use super::{calculate_delay_s, calculate_delay_samples_array};

    #[test]
    fn calculate_delay_s_1() {
        let input_position_mm: Array1<f32> = arr1(&[1000.0, 0.0, 0.0]);
        let output_position_mm: Array1<f32> = arr1(&[2000.0, 0.0, 0.0]);
        let propagation_velocity_m_per_s = 2.0;

        let delay_s = calculate_delay_s(
            &input_position_mm.view(),
            &output_position_mm.view(),
            propagation_velocity_m_per_s,
        );

        assert_relative_eq!(delay_s, 0.5);
    }

    #[test]
    fn calculate_delay_s_2() {
        let input_position_mm: Array1<f32> = arr1(&[1000.0, 0.0, 0.0]);
        let output_position_mm: Array1<f32> = arr1(&[4000.0, 4000.0, 0.0]);
        let propagation_velocity_m_per_s = 2.0;

        let delay_s = calculate_delay_s(
            &input_position_mm.view(),
            &output_position_mm.view(),
            propagation_velocity_m_per_s,
        );

        assert_relative_eq!(delay_s, 2.5);
    }

    #[test]
    fn calculate_delay_samples_array_1() {
        let config = &Model::default();
        let spatial_description = &SpatialDescription::from_model_config(config);
        let sample_rate_hz = 2000.0;

        let delay_samples = calculate_delay_samples_array(
            spatial_description,
            &config.propagation_velocities_m_per_s,
            sample_rate_hz,
        )
        .unwrap();

        let max = delay_samples.values.max_skipnan();
        let expected = (spatial_description.voxels.size_mm / 1000.0)
            / config
                .propagation_velocities_m_per_s
                .get(&VoxelType::Atrioventricular)
                .unwrap()
            * sample_rate_hz
            * 2.0_f32.sqrt();

        assert_relative_eq!(*max, expected);
    }
}
