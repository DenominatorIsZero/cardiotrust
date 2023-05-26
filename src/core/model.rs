pub mod shapes;

use self::shapes::{ArrayDelays, ArrayGains};

pub struct APParameters {
    pub gains: ArrayGains,
    pub coefs: ArrayDelays<f32>,
    pub delays: ArrayDelays<u32>,
}

impl APParameters {
    pub fn new(number_of_states: usize) -> APParameters {
        APParameters {
            gains: ArrayGains::new(number_of_states),
            coefs: ArrayDelays::new(number_of_states),
            delays: ArrayDelays::new(number_of_states),
        }
    }
}
