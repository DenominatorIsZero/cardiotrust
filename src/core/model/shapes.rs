use ndarray::{Array4, Array5};
use num_traits::identities::Zero;

#[derive(Debug)]
pub struct ArrayGains<T> {
    pub values: Array5<T>,
}

impl<T> ArrayGains<T>
where
    T: Clone + Zero,
{
    pub fn new(number_of_states: usize) -> ArrayGains<T> {
        ArrayGains {
            values: Array5::zeros((number_of_states, 3, 3, 3, 3)),
        }
    }
}

#[derive(Debug)]
pub struct ArrayDelays<T> {
    pub values: Array4<T>,
}

impl<T> ArrayDelays<T>
where
    T: Clone + Zero,
{
    pub fn new(number_of_states: usize) -> ArrayDelays<T> {
        assert_eq!(number_of_states as f32 % 3.0, 0.0);
        ArrayDelays {
            values: Array4::zeros((number_of_states / 3, 3, 3, 3)),
        }
    }
}
