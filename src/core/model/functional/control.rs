use ndarray::Array1;

use crate::core::{config::simulation::Simulation, model::spatial::SpatialDescription};

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

    pub fn from_simulation_config(
        config: &Simulation,
        spatial_description: &SpatialDescription,
    ) -> ControlMatrix {
        todo!()
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

    pub fn from_simulation_config(config: &Simulation) -> ControlFunction {
        todo!()
    }
}
