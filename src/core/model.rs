pub mod shapes;

use self::shapes::{ArrayDelays, ArrayGains, ArrayIndicesGains, ArrayKalmanGain, ArrayMeasMat};

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
    pub kalman_gain: ArrayKalmanGain,
}

impl FunctionalDescription {
    pub fn new(number_of_states: usize, number_of_sensors: usize) -> FunctionalDescription {
        FunctionalDescription {
            ap_params: APParameters::new(number_of_states),
            measurement_matrix: ArrayMeasMat::new(number_of_states, number_of_sensors),
            kalman_gain: ArrayKalmanGain::new(number_of_states, number_of_sensors),
        }
    }
}
