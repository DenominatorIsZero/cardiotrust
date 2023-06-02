use super::{
    shapes::{
        ArrayControlFunction, ArrayCtlMat, ArrayDelays, ArrayGains, ArrayIndicesGains,
        ArrayKalmanGain, ArrayMeasMat,
    },
    spatial::SpatialDescription,
};
use crate::core::config::simulation::Simulation;

#[derive(Debug, PartialEq)]
pub struct APParameters {
    pub gains: ArrayGains<f32>,
    pub output_state_indices: ArrayIndicesGains,
    pub coefs: ArrayDelays<f32>,
    pub delays: ArrayDelays<usize>,
}

impl APParameters {
    pub fn empty(number_of_states: usize) -> APParameters {
        APParameters {
            gains: ArrayGains::empty(number_of_states),
            output_state_indices: ArrayIndicesGains::empty(number_of_states),
            coefs: ArrayDelays::empty(number_of_states),
            delays: ArrayDelays::empty(number_of_states),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct FunctionalDescription {
    pub ap_params: APParameters,
    pub measurement_matrix: ArrayMeasMat,
    pub control_matrix: ArrayCtlMat,
    pub kalman_gain: ArrayKalmanGain,
    pub control_function_values: ArrayControlFunction,
}

impl FunctionalDescription {
    pub fn empty(
        number_of_states: usize,
        number_of_sensors: usize,
        number_of_steps: usize,
    ) -> FunctionalDescription {
        FunctionalDescription {
            ap_params: APParameters::empty(number_of_states),
            measurement_matrix: ArrayMeasMat::empty(number_of_states, number_of_sensors),
            control_matrix: ArrayCtlMat::empty(number_of_states),
            kalman_gain: ArrayKalmanGain::empty(number_of_states, number_of_sensors),
            control_function_values: ArrayControlFunction::empty(number_of_steps),
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
