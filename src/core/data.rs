pub mod measurement;
pub mod shapes;
pub mod simulation;

use ndarray::Dim;

use self::measurement::Measurement;
use self::shapes::ArraySystemStates;
use self::simulation::Simulation;

use crate::core::config::simulation::Simulation as SimulationConfig;
use crate::core::data::shapes::ArrayMeasurements;

use super::model::functional::allpass::shapes::{ArrayDelays, ArrayGains};

#[derive(Debug, PartialEq, Clone)]
pub struct Data {
    simulation: Option<Simulation>,
    measurement: Option<Measurement>,
}
impl Data {
    pub fn empty(
        number_of_sensors: usize,
        number_of_states: usize,
        number_of_steps: usize,
        voxels_in_dims: Dim<[usize; 3]>,
    ) -> Data {
        Data {
            simulation: Some(Simulation::empty(
                number_of_sensors,
                number_of_states,
                number_of_steps,
                voxels_in_dims,
            )),
            measurement: None,
        }
    }

    pub fn from_simulation_config(config: &SimulationConfig) -> Data {
        let mut simulation = Simulation::from_config(config).unwrap();
        simulation.run();
        Data {
            simulation: Some(simulation),
            measurement: None,
        }
    }

    pub fn get_measurements(&self) -> &ArrayMeasurements {
        if let Some(simulation) = self.simulation.as_ref() {
            &(simulation.measurements)
        } else {
            &(self.measurement.as_ref().unwrap().measurements)
        }
    }

    pub fn get_system_states(&self) -> &ArraySystemStates {
        match &self.simulation {
            Some(simulation) => &simulation.system_states,
            None => todo!("Non simulation case not implemented yet."),
        }
    }

    pub fn get_gains(&self) -> &ArrayGains<f32> {
        match &self.simulation {
            Some(simulation) => &simulation.model.functional_description.ap_params.gains,
            None => todo!("Non simulation case not implemented yet."),
        }
    }

    pub fn get_coefs(&self) -> &ArrayDelays<f32> {
        match &self.simulation {
            Some(simulation) => &simulation.model.functional_description.ap_params.coefs,
            None => todo!("Non simulation case not implemented yet."),
        }
    }

    pub fn get_delays(&self) -> &ArrayDelays<usize> {
        match &self.simulation {
            Some(simulation) => &simulation.model.functional_description.ap_params.delays,
            None => todo!("Non simulation case not implemented yet."),
        }
    }
}
