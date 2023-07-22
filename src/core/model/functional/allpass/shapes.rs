use approx::assert_relative_eq;
use ndarray::{Array3, Array4, Array5, Dim};
use num_traits::Zero;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
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
    #[must_use]
    pub fn empty(number_of_states: usize) -> Self {
        Self {
            values: Array5::zeros((number_of_states, 3, 3, 3, 3)),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct ArrayIndicesGains {
    pub values: Array5<Option<usize>>,
}

impl ArrayIndicesGains {
    #[must_use]
    pub fn empty(number_of_states: usize) -> Self {
        Self {
            values: Array5::from_elem((number_of_states, 3, 3, 3, 3), None),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
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
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn empty(number_of_states: usize) -> Self {
        assert_relative_eq!(number_of_states as f32 % 3.0, 0.0);
        Self {
            values: Array4::zeros((number_of_states / 3, 3, 3, 3)),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ArrayActivationTime {
    pub values: Array3<Option<f32>>,
}

impl ArrayActivationTime {
    #[must_use]
    pub fn empty(voxels_in_dims: Dim<[usize; 3]>) -> Self {
        Self {
            values: Array3::from_elem(voxels_in_dims, None),
        }
    }
}
