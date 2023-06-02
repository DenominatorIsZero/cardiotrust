pub mod allpass;
pub mod control;
pub mod kalman;
pub mod measurement;

use self::{
    allpass::APParameters,
    control::{ControlFunction, ControlMatrix},
    kalman::KalmanGain,
    measurement::MeasurementMatrix,
};

use super::spatial::SpatialDescription;
use crate::core::config::simulation::Simulation;

#[derive(Debug, PartialEq)]
pub struct FunctionalDescription {
    pub ap_params: APParameters,
    pub measurement_matrix: MeasurementMatrix,
    pub control_matrix: ControlMatrix,
    pub kalman_gain: KalmanGain,
    pub control_function_values: ControlFunction,
}

impl FunctionalDescription {
    pub fn empty(
        number_of_states: usize,
        number_of_sensors: usize,
        number_of_steps: usize,
    ) -> FunctionalDescription {
        FunctionalDescription {
            ap_params: APParameters::empty(number_of_states),
            measurement_matrix: MeasurementMatrix::empty(number_of_states, number_of_sensors),
            control_matrix: ControlMatrix::empty(number_of_states),
            kalman_gain: KalmanGain::empty(number_of_states, number_of_sensors),
            control_function_values: ControlFunction::empty(number_of_steps),
        }
    }
    pub fn from_simulation_config(
        _config: &Simulation,
        _spatial_description: &SpatialDescription,
    ) -> FunctionalDescription {
        todo!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ap_empty() {
        let number_of_states = 3000;

        let _ap_params = APParameters::empty(number_of_states);
    }

    #[test]
    fn funcional_empty() {
        let number_of_states = 3000;
        let number_of_sensors = 300;
        let number_of_steps = 2000;

        let _functional_description =
            FunctionalDescription::empty(number_of_states, number_of_sensors, number_of_steps);
    }

    #[test]
    fn from_simulation_config_no_crash() {
        let config = Simulation::default();
        let spatial_description = SpatialDescription::from_simulation_config(&config);
        let _functional_description =
            FunctionalDescription::from_simulation_config(&config, &spatial_description);
    }
}
