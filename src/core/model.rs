pub mod shapes;

use self::shapes::{
    ArrayControlFunction, ArrayCtlMat, ArrayDelays, ArrayGains, ArrayIndicesGains, ArrayKalmanGain,
    ArrayMeasMat,
};

pub struct APParameters {
    pub gains: ArrayGains<f32>,
    pub output_state_indices: ArrayIndicesGains,
    pub coefs: ArrayDelays<f32>,
    pub delays: ArrayDelays<usize>,
}

impl APParameters {
    pub fn new(number_of_states: usize) -> APParameters {
        APParameters {
            gains: ArrayGains::new(number_of_states),
            output_state_indices: ArrayIndicesGains::new(number_of_states),
            coefs: ArrayDelays::new(number_of_states),
            delays: ArrayDelays::new(number_of_states),
        }
    }
}

pub struct FunctionalDescription {
    pub ap_params: APParameters,
    pub measurement_matrix: ArrayMeasMat,
    pub control_matrix: ArrayCtlMat,
    pub kalman_gain: ArrayKalmanGain,
    pub control_function_values: ArrayControlFunction,
}

impl FunctionalDescription {
    pub fn new(
        number_of_states: usize,
        number_of_sensors: usize,
        number_of_steps: usize,
    ) -> FunctionalDescription {
        FunctionalDescription {
            ap_params: APParameters::new(number_of_states),
            measurement_matrix: ArrayMeasMat::new(number_of_states, number_of_sensors),
            control_matrix: ArrayCtlMat::new(number_of_states),
            kalman_gain: ArrayKalmanGain::new(number_of_states, number_of_sensors),
            control_function_values: ArrayControlFunction::new(number_of_steps),
        }
    }
}
