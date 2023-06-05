use ndarray::Array1;

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

    pub fn from_model_config(_config: &Model) -> ControlFunction {
        todo!();
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
        let config = Model::default();

        let _control_function = ControlFunction::from_model_config(&config);
    }
}
