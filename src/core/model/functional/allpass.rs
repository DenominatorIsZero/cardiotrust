use self::shapes::{ArrayDelays, ArrayGains, ArrayIndicesGains};

pub mod shapes;

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
