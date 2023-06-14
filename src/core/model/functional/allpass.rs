use std::{collections::HashMap, error::Error};

use approx::relative_eq;

use itertools::Itertools;
use ndarray::{arr1, s, Array1, Array3, Array4, ArrayBase, Dim, ViewRepr};
use ndarray_stats::QuantileExt;

use crate::core::{
    config::model::Model,
    model::spatial::{voxels, voxels::VoxelType, SpatialDescription},
};

use self::{
    delay::calculate_delay_samples_array,
    shapes::{ArrayActivationTime, ArrayDelays, ArrayGains, ArrayIndicesGains},
};

mod delay;
mod direction;
mod gain;
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
    pub fn empty(number_of_states: usize, voxels_in_dims: Dim<[usize; 3]>) -> APParameters {
        APParameters {
            gains: ArrayGains::empty(number_of_states),
            output_state_indices: ArrayIndicesGains::empty(number_of_states),
            coefs: ArrayDelays::empty(number_of_states),
            delays: ArrayDelays::empty(number_of_states),
            activation_time_ms: ArrayActivationTime::empty(voxels_in_dims),
        }
    }

    pub fn from_model_config(
        config: &Model,
        spatial_description: &SpatialDescription,
        sample_rate_hz: f32,
    ) -> Result<APParameters, Box<dyn Error>> {
        let voxels_in_dims = spatial_description.voxels.types.values.raw_dim();
        let mut ap_params =
            APParameters::empty(spatial_description.voxels.count_states(), voxels_in_dims);
        let mut activation_time_s = Array3::<Option<f32>>::from_elem(
            spatial_description.voxels.types.values.raw_dim(),
            None,
        );
        let mut current_directions =
            Array4::<f32>::zeros(spatial_description.voxels.positions_mm.values.raw_dim());

        let v_types = &spatial_description.voxels.types.values;
        let v_position_mm = &spatial_description.voxels.positions_mm.values;
        let v_numbers = &spatial_description.voxels.numbers.values;

        // TODO: Extract into function
        // TODO: write tests
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
                for x_offset in -1..=1 {
                    for y_offset in -1..=1 {
                        for z_offset in -1..=1 {
                            // no self connection allowed
                            if x_offset == 0 && y_offset == 0 && z_offset == 0 {
                                continue;
                            }
                            let (x_out, y_out, z_out) = output_voxel_index;
                            let input_voxel_index = [
                                x_out as i32 - x_offset,
                                y_out as i32 - y_offset,
                                z_out as i32 - z_offset,
                            ];
                            // Skip if the input voxel doesn't exist
                            if !spatial_description.voxels.is_valid_index(input_voxel_index) {
                                continue;
                            }
                            let input_voxel_index = [
                                (x_out as i32 - x_offset) as usize,
                                (y_out as i32 - y_offset) as usize,
                                (z_out as i32 - z_offset) as usize,
                            ];
                            // SKip if the input voxel is already connected
                            if activation_time_s[input_voxel_index].is_some() {
                                continue;
                            }
                            // Skip if connection is not allowed
                            let output_voxel_type = &v_types[output_voxel_index];
                            let input_voxel_type = &v_types[input_voxel_index];
                            if !voxels::is_connection_allowed(output_voxel_type, input_voxel_type) {
                                continue;
                            }
                            // Skip pathologies as anways if the propagation factor is zero
                            if input_voxel_type == &VoxelType::Pathological
                                && relative_eq!(config.current_factor_in_pathology, 0.0)
                            {
                                continue;
                            }

                            // Now we finally found something that we want to connect.
                            let input_state_number = v_numbers[input_voxel_index].unwrap();
                            connected_something = true;

                            let output_position_mm =
                                &v_position_mm.slice(s![x_out, y_out, z_out, ..]);
                            let [x_in, y_in, z_in] = input_voxel_index;
                            let input_position_mm = &v_position_mm.slice(s![x_in, y_in, z_in, ..]);
                            let propagation_velocity_m_per_s = config
                                .propagation_velocities_m_per_s
                                .get(input_voxel_type)
                                .unwrap();

                            let delay_s = delay::calculate_delay_s(
                                input_position_mm,
                                output_position_mm,
                                propagation_velocity_m_per_s,
                            );
                            let direction = direction::calculate_current_direction(
                                input_position_mm,
                                output_position_mm,
                            );

                            // update activation time of input voxel, marking them as connected
                            activation_time_s[input_voxel_index] =
                                Some(activation_time_s[output_voxel_index].unwrap() + delay_s);

                            current_directions
                                .slice_mut(s![x_in, y_in, z_in, ..])
                                .assign(&direction);

                            let mut gain = gain::calculate_gain(
                                &direction,
                                current_directions.slice(s![x_out, y_out, z_out, ..]),
                            );

                            if *input_voxel_type == VoxelType::Pathological
                                && *output_voxel_type != VoxelType::Pathological
                            {
                                gain = gain * config.current_factor_in_pathology;
                            }
                            if *output_voxel_type == VoxelType::Pathological
                                && *input_voxel_type != VoxelType::Pathological
                            {
                                gain = gain * (1.0 / config.current_factor_in_pathology);
                            }

                            for input_dimension in 0..3 {
                                for output_dimension in 0..3 {
                                    ap_params.gains.values[(
                                        input_state_number + input_dimension,
                                        (1 + x_offset) as usize,
                                        (1 + y_offset) as usize,
                                        (1 + z_offset) as usize,
                                        output_dimension,
                                    )] = gain[(input_dimension, output_dimension)];
                                }
                            }
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

        // TODO: Extract into function
        // TODO: write tests
        // init output state indices
        v_types
            .indexed_iter()
            .filter(|(_, v_type)| **v_type != VoxelType::None)
            .for_each(|(input_voxel_index, _)| {
                let (x_in, y_in, z_in) = input_voxel_index;
                for x_offset in -1..=1 {
                    for y_offset in -1..=1 {
                        for z_offset in -1..=1 {
                            let ouput_voxel_index = [
                                x_in as i32 + x_offset,
                                y_in as i32 + y_offset,
                                z_in as i32 + z_offset,
                            ];
                            if !spatial_description.voxels.is_valid_index(ouput_voxel_index) {
                                continue;
                            }
                            let output_voxel_index = [
                                (x_in as i32 + x_offset) as usize,
                                (y_in as i32 + y_offset) as usize,
                                (z_in as i32 + z_offset) as usize,
                            ];
                            for input_direction in 0..3 {
                                let input_state_number =
                                    v_numbers[input_voxel_index].unwrap() + input_direction;
                                for output_direction in 0..3 {
                                    let output_state_index =
                                        v_numbers[output_voxel_index].unwrap() + output_direction;
                                    ap_params.output_state_indices.values[(
                                        input_state_number,
                                        (1 + x_offset) as usize,
                                        (1 + y_offset) as usize,
                                        (1 + z_offset) as usize,
                                        output_direction,
                                    )] = Some(output_state_index);
                                }
                            }
                        }
                    }
                }
            });

        ap_params
            .activation_time_ms
            .values
            .iter_mut()
            .zip(activation_time_s)
            .filter(|(_, s)| s.is_some())
            .for_each(|(ms, s)| *ms = Some(s.unwrap() * 1000.0));

        let delays_samples = calculate_delay_samples_array(
            spatial_description,
            &config.propagation_velocities_m_per_s,
            sample_rate_hz,
        )?;

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
}

fn from_samples_to_coef(samples: f32) -> f32 {
    let fractional = samples % 1.0;
    (1.0 - fractional) / (1.0 + fractional)
}

fn from_samples_to_usize(samples: f32) -> usize {
    samples as usize
}

#[cfg(test)]
mod test {
    use crate::{
        core::{
            config::model::Model,
            model::{
                functional::allpass::{from_samples_to_coef, from_samples_to_usize},
                spatial::{voxels::VoxelType, SpatialDescription},
            },
        },
        vis::plotting::plot_activation_time,
    };

    use super::APParameters;

    #[test]
    fn from_samples_to_usize_1() {
        assert_eq!(1, from_samples_to_usize(1.0));
        assert_eq!(1, from_samples_to_usize(1.2));
        assert_eq!(10, from_samples_to_usize(10.9));
        assert_eq!(10, from_samples_to_usize(10.0));
    }

    #[test]
    fn from_samples_to_coef_1() {
        assert_eq!(1.0 / 3.0, from_samples_to_coef(0.5));
        assert_eq!(1.0 / 3.0, from_samples_to_coef(1.5));
        assert_eq!(1.0 / 3.0, from_samples_to_coef(99999.5));

        assert_eq!(1.0, from_samples_to_coef(0.0));
        assert_eq!(1.0, from_samples_to_coef(1.0));
        assert_eq!(1.0, from_samples_to_coef(99999.0));
    }

    #[test]
    fn activation_time_is_some() {
        let config = &Model::default();
        let spatial_description = &SpatialDescription::from_model_config(config);
        let sample_rate_hz = 2000.0;
        let ap_params =
            APParameters::from_model_config(config, spatial_description, sample_rate_hz).unwrap();

        for (index, activation_time_ms) in ap_params.activation_time_ms.values.indexed_iter() {
            assert!(
                activation_time_ms.is_some(),
                "Activation time at {:?} was none.",
                index
            );
            assert!(activation_time_ms.unwrap() >= 0.0);
        }
        plot_activation_time(
            &ap_params.activation_time_ms,
            "tests/ap_params_activation_times_default",
            "Activation times [ms]",
        )
    }

    #[test]
    fn activation_time_fast_av() {
        let mut config = Model::default();
        config
            .propagation_velocities_m_per_s
            .insert(VoxelType::Atrioventricular, 0.8);
        let spatial_description = &SpatialDescription::from_model_config(&config);
        let sample_rate_hz = 2000.0;
        let ap_params =
            APParameters::from_model_config(&config, spatial_description, sample_rate_hz).unwrap();

        for (index, activation_time_ms) in ap_params.activation_time_ms.values.indexed_iter() {
            assert!(
                activation_time_ms.is_some(),
                "Activation time at {:?} was none.",
                index
            );
            assert!(activation_time_ms.unwrap() >= 0.0);
        }
        plot_activation_time(
            &ap_params.activation_time_ms,
            "tests/ap_params_activation_times_fast_av",
            "Activation times [ms]",
        )
    }
}
