pub mod functional;
pub mod spatial;

use self::{functional::FunctionalDescription, spatial::SpatialDescription};

use super::config::simulation::Simulation;

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
    ) -> Model {
        Model {
            functional_description: FunctionalDescription::empty(
                number_of_states,
                number_of_sensors,
                number_of_steps,
            ),
            spatial_description: SpatialDescription::empty(
                number_of_sensors,
                [number_of_states / 3 as usize, 1, 1],
            ),
        }
    }
    pub fn from_simulation_config(config: &Simulation) -> Model {
        let spatial_description = SpatialDescription::from_simulation_config(config);
        let functional_description =
            FunctionalDescription::from_simulation_config(config, &spatial_description);
        Model {
            functional_description,
            spatial_description,
        }
    }
}
