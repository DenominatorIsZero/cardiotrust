use ndarray::Array1;
use ndarray_npy::read_npy;
use samplerate::{self, ConverterType};

use crate::core::{
    config::model::Model,
    model::spatial::{voxels::VoxelType, SpatialDescription},
};

#[derive(Debug, PartialEq)]
pub struct ControlMatrix {
    pub values: Array1<f32>,
}

impl ControlMatrix {
    pub fn empty(number_of_states: usize) -> ControlMatrix {
        ControlMatrix {
            values: Array1::zeros(number_of_states),
        }
    }

    pub fn from_model_config(
        _config: &Model,
        spatial_description: &SpatialDescription,
    ) -> ControlMatrix {
        let mut control_matrix = ControlMatrix::empty(spatial_description.voxels.count_states());
        spatial_description
            .voxels
            .types
            .values
            .iter()
            .zip(spatial_description.voxels.numbers.values.iter())
            .for_each(|(v_type, v_number)| {
                if *v_type == VoxelType::Sinoatrial {
                    control_matrix.values[v_number.unwrap()] = 1.0;
                }
            });
        control_matrix
    }
}

#[derive(Debug, PartialEq)]
pub struct ControlFunction {
    pub values: Array1<f32>,
}

impl ControlFunction {
    pub fn empty(number_of_steps: usize) -> ControlFunction {
        ControlFunction {
            values: Array1::zeros(number_of_steps),
        }
    }

    pub fn from_model_config(
        _config: &Model,
        sample_rate_hz: f32,
        duration_s: f32,
    ) -> ControlFunction {
        let sample_rate_hz_in = 2000.0;
        let control_function_raw: Array1<f32> =
            read_npy("assets/control_function_ohara.npy").unwrap();

        let desired_length_samples = (duration_s * sample_rate_hz) as usize;

        let control_function_converted = samplerate::convert(
            sample_rate_hz_in as u32,
            sample_rate_hz as u32,
            1,
            ConverterType::SincBestQuality,
            &control_function_raw.to_vec(),
        )
        .unwrap();

        let control_function_values: Vec<f32> = (0..desired_length_samples)
            .map(|i| {
                let index = i % control_function_converted.len();
                control_function_converted[index]
            })
            .collect();

        ControlFunction {
            values: Array1::from(control_function_values),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn matrix_from_model_config_no_crash() {
        let config = Model::default();
        let spatial_description = SpatialDescription::from_model_config(&config);

        let _control_matrix = ControlMatrix::from_model_config(&config, &spatial_description);
    }

    #[test]
    fn function_from_model_config_no_crash() {
        let sample_rate_hz = 3000.0;
        let duration_s = 3.0;
        let expected_length_samples = (sample_rate_hz * duration_s) as usize;
        let config = Model::default();

        let control_function =
            ControlFunction::from_model_config(&config, sample_rate_hz, duration_s);
        assert_eq!(expected_length_samples, control_function.values.shape()[0]);
    }
}
