use approx::relative_eq;

use ndarray::Dim;

mod delay;
mod direction;
pub mod flat;
mod gain;
pub mod shapes;

fn find_candidate_voxels(
    activation_time_s: &ndarray::ArrayBase<ndarray::OwnedRepr<Option<f32>>, Dim<[usize; 3]>>,
    current_time_s: f32,
) -> Vec<(usize, usize, usize)> {
    let output_voxel_indices: Vec<(usize, usize, usize)> = activation_time_s
        .indexed_iter()
        .filter(|(_, time_s)| time_s.is_some() && relative_eq!(time_s.unwrap(), current_time_s))
        .map(|(index, _)| index)
        .collect();
    output_voxel_indices
}

fn from_samples_to_coef(samples: f32) -> f32 {
    let fractional = samples % 1.0;
    (1.0 - fractional) / (1.0 + fractional)
}

#[allow(
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation
)]
#[must_use]
const fn from_samples_to_usize(samples: f32) -> usize {
    samples as usize
}

#[must_use]
pub fn from_coef_to_samples(coef: f32) -> f32 {
    (1.0 - coef) / (coef + 1.0)
}

#[cfg(test)]
mod test {
    use approx::assert_relative_eq;

    use crate::core::model::functional::allpass::{from_samples_to_coef, from_samples_to_usize};

    #[test]
    fn from_samples_to_usize_1() {
        assert_eq!(1, from_samples_to_usize(1.0));
        assert_eq!(1, from_samples_to_usize(1.2));
        assert_eq!(10, from_samples_to_usize(10.9));
        assert_eq!(10, from_samples_to_usize(10.0));
    }

    #[test]
    fn from_samples_to_coef_1() {
        assert_relative_eq!(1.0 / 3.0, from_samples_to_coef(0.5));
        assert_relative_eq!(1.0 / 3.0, from_samples_to_coef(1.5));
        assert_relative_eq!(1.0 / 3.0, from_samples_to_coef(99999.5));

        assert_relative_eq!(1.0, from_samples_to_coef(0.0));
        assert_relative_eq!(1.0, from_samples_to_coef(1.0));
        assert_relative_eq!(1.0, from_samples_to_coef(99999.0));
    }
}
