use ndarray::{Array3, Array4, Array5, Dim};
use num_traits::Zero;

#[derive(Debug, PartialEq, Clone)]
pub struct ArrayGains<T>
where
    T: Clone + Zero + PartialEq,
{
    pub values: Array5<T>,
}

impl<T> ArrayGains<T>
where
    T: Clone + Zero + PartialEq,
{
    pub fn empty(number_of_states: usize) -> ArrayGains<T> {
        ArrayGains {
            values: Array5::zeros((number_of_states, 3, 3, 3, 3)),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct ArrayIndicesGains {
    pub values: Array5<Option<usize>>,
}

impl ArrayIndicesGains {
    pub fn empty(number_of_states: usize) -> ArrayIndicesGains {
        ArrayIndicesGains {
            values: Array5::from_elem((number_of_states, 3, 3, 3, 3), None),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct ArrayDelays<T>
where
    T: Clone + Zero + PartialEq,
{
    pub values: Array4<T>,
}

impl<T> ArrayDelays<T>
where
    T: Clone + Zero + PartialEq,
{
    pub fn empty(number_of_states: usize) -> ArrayDelays<T> {
        assert_eq!(number_of_states as f32 % 3.0, 0.0);
        ArrayDelays {
            values: Array4::zeros((number_of_states / 3, 3, 3, 3)),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct ArrayActivationTime {
    pub values: Array3<Option<f32>>,
}

impl ArrayActivationTime {
    pub fn empty(voxels_in_dims: Dim<[usize; 3]>) -> ArrayActivationTime {
        ArrayActivationTime {
            values: Array3::from_elem(voxels_in_dims, None),
        }
    }
}
