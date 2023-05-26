use crate::core::model::shapes::{ArrayDelays, ArrayGains};

pub struct Derivatives {
    pub gains: ArrayGains,
    pub delays: ArrayDelays<f32>,
}

impl Derivatives {
    pub fn new(number_of_states: usize) -> Derivatives {
        Derivatives {
            gains: ArrayGains::new(number_of_states),
            delays: ArrayDelays::new(number_of_states),
        }
    }
}
