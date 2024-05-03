use ndarray::{Array1, ArrayBase, Dim, ViewRepr};
use tracing::trace;

/// Calculates the direction the current should flow in the
/// Voxel at `input_position`, when excited form the voxel at
/// `output_position`.
///
/// Returns:
/// An array containing the current direction, where the
/// sum over the absolute value of the components is always
/// equal to one.
#[tracing::instrument(level = "trace")]
pub fn calculate(
    input_position_mm: &ArrayBase<ViewRepr<&f32>, Dim<[usize; 1]>>,
    output_position_mm: &ArrayBase<ViewRepr<&f32>, Dim<[usize; 1]>>,
) -> Array1<f32> {
    trace!("Calculating direction");
    let distance_m = (input_position_mm - output_position_mm) / 1000.0;
    let distance_norm_m = distance_m.mapv(f32::abs).sum();

    distance_m / distance_norm_m
}

#[cfg(test)]
mod test {
    use approx::assert_relative_eq;
    use ndarray::{arr1, Array1};

    use super::calculate;

    #[test]
    fn calculate_direction_simple() {
        let output_position_mm: Array1<f32> = arr1(&[1.0, 1.0, 1.0]);
        let input_position_mm: Array1<f32> = arr1(&[2.0, 1.0, 1.0]);
        let expected: Array1<f32> = arr1(&[1.0, 0.0, 0.0]);

        let direction = calculate(&input_position_mm.view(), &output_position_mm.view());

        for i in 0..3 {
            assert_relative_eq!(direction[i], expected[i], epsilon = 0.01);
        }
    }

    #[test]
    fn calculate_direction_diag() {
        let output_position_mm: Array1<f32> = arr1(&[1.0, 1.0, 1.0]);
        let input_position_mm: Array1<f32> = arr1(&[2.0, 0.0, 1.0]);
        let expected: Array1<f32> = arr1(&[1.0 / 2.0, -1.0 / 2.0, 0.0]);

        let direction = calculate(&input_position_mm.view(), &output_position_mm.view());

        for i in 0..3 {
            assert_relative_eq!(direction[i], expected[i], epsilon = 0.01);
        }
    }

    #[test]
    fn calculate_direction_room_diag() {
        let output_position_mm: Array1<f32> = arr1(&[1.0, 1.0, 1.0]);
        let input_position_mm: Array1<f32> = arr1(&[2.0, 0.0, 2.0]);
        let expected: Array1<f32> = arr1(&[1.0 / 3.0, -1.0 / 3.0, 1.0 / 3.0]);

        let direction = calculate(&input_position_mm.view(), &output_position_mm.view());

        for i in 0..3 {
            assert_relative_eq!(direction[i], expected[i], epsilon = 0.01);
        }
    }
}
