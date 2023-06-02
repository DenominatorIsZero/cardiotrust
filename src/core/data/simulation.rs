use crate::core::{
    algorithm::estimation::calculate_system_prediction,
    config::simulation::Simulation as config,
    data::ArrayMeasurements,
    model::{shapes::ArrayGains, Model},
};

use super::shapes::ArraySystemStates;

#[derive(Debug, PartialEq)]
pub struct Simulation {
    pub measurements: ArrayMeasurements,
    pub system_states: ArraySystemStates,
    pub model: Model,
}
impl Simulation {
    pub fn empty(
        number_of_sensors: usize,
        number_of_states: usize,
        number_of_steps: usize,
    ) -> Simulation {
        Simulation {
            measurements: ArrayMeasurements::empty(number_of_steps, number_of_sensors),
            system_states: ArraySystemStates::empty(number_of_steps, number_of_states),
            model: Model::empty(number_of_states, number_of_sensors, number_of_steps),
        }
    }

    pub fn from_config(config: &config) -> Simulation {
        let model = Model::from_simulation_config(config);
        let number_of_sensors = model.spatial_description.sensors.count();
        let number_of_states = model.spatial_description.voxels.count_states();
        let number_of_steps = (config.sample_rate_hz * config.duration_s) as usize;

        let mut measurements = ArrayMeasurements::empty(number_of_steps, number_of_sensors);
        let mut system_states = ArraySystemStates::empty(number_of_steps, number_of_states);

        Simulation::run(&mut measurements, &mut system_states, &model);

        Simulation {
            measurements,
            system_states,
            model,
        }
    }

    fn run(
        measurements: &mut ArrayMeasurements,
        system_states: &mut ArraySystemStates,
        model: &Model,
    ) {
        let mut ap_outputs = ArrayGains::empty(system_states.values.shape()[1]);
        for time_index in 0..system_states.values.shape()[0] {
            calculate_system_prediction(
                &mut ap_outputs,
                system_states,
                measurements,
                &model.functional_description,
                time_index,
            )
        }
        // TODO: Add noise to measurements here
    }
}
