use itertools::Itertools;
use ndarray::{s, ArrayBase, Dim, ViewRepr};
use std::{collections::HashMap, error::Error};
use tracing::trace;

use super::{offset_to_delay_index, shapes::Coefs};
use crate::core::model::spatial::{voxels::VoxelType, SpatialDescription};

/// Calculates the delay in seconds for a given input and output position,
/// based on the propagation velocity. Takes the Euclidean distance between
/// the input and output positions (converted to meters), divides by the  
/// propagation velocity to get the delay in seconds.
#[tracing::instrument(level = "trace")]
pub fn calculate_delay_s(
    input_position_mm: &ArrayBase<ViewRepr<&f32>, Dim<[usize; 1]>>,
    output_position_mm: &ArrayBase<ViewRepr<&f32>, Dim<[usize; 1]>>,
    propagation_velocity_m_per_s: f32,
) -> f32 {
    trace!("Calculating delay in seconds");
    let distance_m = (input_position_mm - output_position_mm) / 1000.0;
    let distance_norm_m = distance_m.mapv(|v| v.powi(2)).sum().sqrt();
    distance_norm_m / propagation_velocity_m_per_s
}

/// Calculates an array of delay values in samples for each voxel and its neighborhood,
/// based on the spatial description, material propagation velocities, and sample rate.
///
/// The delay values are calculated by taking the Euclidean distance between each voxel
/// and its neighbors, dividing by the propagation velocity to get delay in seconds,
/// and multiplying by the sample rate to convert to samples.
///
/// Returns the 2D array of delay values, with dimensions corresponding to the
/// voxel numbers and neighbor offsets.
#[tracing::instrument(level = "trace")]
pub fn calculate_delay_samples_array(
    spatial_description: &SpatialDescription,
    propagation_velocities_m_per_s: &HashMap<VoxelType, f32>,
    sample_rate_hz: f32,
) -> Result<Coefs, Box<dyn Error>> {
    trace!("Calculating delay samples array");
    let mut delay_samples_array = Coefs::empty(spatial_description.voxels.count_states());

    let v_types = &spatial_description.voxels.types;
    let v_position_mm = &spatial_description.voxels.positions_mm;
    let v_numbers = &spatial_description.voxels.numbers;

    // Fill the delays_samples tensor
    for (input_voxel_index, v_type) in v_types.indexed_iter() {
        if !v_type.is_connectable() {
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

            delay_samples_array[(
                v_numbers[input_voxel_index].unwrap() / 3,
                offset_to_delay_index(x_offset, y_offset, z_offset)
                    .expect("Offsets to not all be zero."),
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
            &config.common.propagation_velocities_m_per_s,
            sample_rate_hz,
        )
        .unwrap();

        let max = delay_samples.max_skipnan();
        let expected = (spatial_description.voxels.size_mm / 1000.0)
            / config
                .common
                .propagation_velocities_m_per_s
                .get(&VoxelType::Atrioventricular)
                .unwrap()
            * sample_rate_hz
            * 2.0_f32.sqrt();

        assert_relative_eq!(*max, expected);
    }
}
