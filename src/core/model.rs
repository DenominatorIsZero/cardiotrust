pub mod functional;
pub mod spatial;

use std::error::Error;

use ndarray::Dim;

use self::{functional::FunctionalDescription, spatial::SpatialDescription};

use super::config::model::Model as ModelConfig;

#[derive(Debug, PartialEq)]
pub struct Model {
    pub functional_description: FunctionalDescription,
    pub spatial_description: SpatialDescription,
}

impl Model {
    pub fn empty(
        number_of_states: usize,
        number_of_sensors: usize,
        number_of_steps: usize,
        voxels_in_dims: Dim<[usize; 3]>,
    ) -> Model {
        Model {
            functional_description: FunctionalDescription::empty(
                number_of_states,
                number_of_sensors,
                number_of_steps,
                voxels_in_dims,
            ),
            spatial_description: SpatialDescription::empty(
                number_of_sensors,
                [number_of_states / 3 as usize, 1, 1],
            ),
        }
    }
    pub fn from_model_config(
        config: &ModelConfig,
        sample_rate_hz: f32,
        duration_s: f32,
    ) -> Result<Model, Box<dyn Error>> {
        let spatial_description = SpatialDescription::from_model_config(config);
        let functional_description = FunctionalDescription::from_model_config(
            config,
            &spatial_description,
            sample_rate_hz,
            duration_s,
        )?;
        Ok(Model {
            functional_description,
            spatial_description,
        })
    }
}
