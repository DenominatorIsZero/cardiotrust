pub mod shapes;

use self::shapes::{ArrayDelays, ArrayGains, ArrayIndicesGains};

pub struct APParameters {
    pub gains: ArrayGains<f32>,
    pub output_state_indices: ArrayIndicesGains,
    pub coefs: ArrayDelays<f32>,
    pub delays: ArrayDelays<u32>,
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
}
