use std::error::Error;

use crate::core::{
    algorithm::estimation::calculate_system_prediction,
    config::simulation::Simulation as SimulationConfig,
    data::ArrayMeasurements,
    model::{functional::allpass::shapes::ArrayGains, Model},
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

    pub fn from_config(config: &SimulationConfig) -> Result<Simulation, Box<dyn Error>> {
        let model =
            Model::from_model_config(&config.model, config.sample_rate_hz, config.duration_s)?;
        let number_of_sensors = model.spatial_description.sensors.count();
        let number_of_states = model.spatial_description.voxels.count_states();
        let number_of_steps = (config.sample_rate_hz * config.duration_s) as usize;

        let measurements = ArrayMeasurements::empty(number_of_steps, number_of_sensors);
        let system_states = ArraySystemStates::empty(number_of_steps, number_of_states);

        Ok(Simulation {
            measurements,
            system_states,
            model,
        })
    }

    pub fn run(&mut self) {
        let measurements = &mut self.measurements;
        let system_states = &mut self.system_states;
        let model = &self.model;
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

#[cfg(test)]
mod test {
    use approx::{assert_relative_eq, relative_eq, RelativeEq};
    use ndarray::s;

    use crate::{core::model::spatial::voxels::VoxelType, vis::plotting::standard_time_plot};

    use super::*;

    #[test]
    fn create_simulation_no_crash() {
        let config = &SimulationConfig::default();
        let simulation = Simulation::from_config(config);
        assert!(simulation.is_ok());
        let simulation = simulation.unwrap();
        let max = *simulation
            .system_states
            .values
            .iter()
            .reduce(|max, e| if e > max { e } else { max })
            .unwrap_or(&f32::MIN);
        assert_relative_eq!(max, 0.0);
        let max = *simulation
            .measurements
            .values
            .iter()
            .reduce(|max, e| if e > max { e } else { max })
            .unwrap_or(&f32::MIN);
        assert_relative_eq!(max, 0.0);
    }

    #[test]
    fn run_simulation_default() {
        let config = &SimulationConfig::default();
        let mut simulation = Simulation::from_config(config).unwrap();
        simulation.run();
        let max = *simulation
            .system_states
            .values
            .iter()
            .reduce(|max, e| if e > max { e } else { max })
            .unwrap_or(&f32::MIN);
        assert!(max.relative_eq(&1.0, 0.0001, 0.0001));
        let max = *simulation
            .measurements
            .values
            .iter()
            .reduce(|max, e| if e > max { e } else { max })
            .unwrap_or(&f32::MIN);
        assert!(max > 0.0);

        let mut sa_index = 0;
        simulation
            .model
            .spatial_description
            .voxels
            .types
            .values
            .iter()
            .zip(
                simulation
                    .model
                    .spatial_description
                    .voxels
                    .numbers
                    .values
                    .iter(),
            )
            .filter(|(v_type, _)| **v_type == VoxelType::Sinoatrial)
            .for_each(|(_, number)| sa_index = number.unwrap());

        let y = &simulation
            .system_states
            .values
            .slice(s![.., sa_index])
            .to_owned();
        standard_time_plot(
            y,
            config.sample_rate_hz,
            "tests/simulation_sa_x",
            "Simulated Current Density Sinoatrial Node",
            "j [A/mm^2]",
        );
        let y = &simulation
            .system_states
            .values
            .slice(s![.., sa_index + 1])
            .to_owned();
        standard_time_plot(
            y,
            config.sample_rate_hz,
            "tests/simulation_sa_y",
            "Simulated Current Density Sinoatrial Node",
            "j [A/mm^2]",
        );
        let y = &simulation
            .system_states
            .values
            .slice(s![.., sa_index + 2])
            .to_owned();
        standard_time_plot(
            y,
            config.sample_rate_hz,
            "tests/simulation_sa_z",
            "Simulated Current Density Sinoatrial Node",
            "j [A/mm^2]",
        );
    }
}
