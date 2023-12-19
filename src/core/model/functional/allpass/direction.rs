use ndarray::{Array1, ArrayBase, Dim, ViewRepr};

/// Calculates the direction the current should flow in the
/// Voxel at `input_position`, when excited form the voxel at
/// `output_position`.
///
/// Returns:
/// An array containing the current direction, where the
/// sum over the absolute value of the components is always
/// equal to one.
pub fn calculate(
    input_position_mm: &ArrayBase<ViewRepr<&f32>, Dim<[usize; 1]>>,
    output_position_mm: &ArrayBase<ViewRepr<&f32>, Dim<[usize; 1]>>,
) -> Array1<f32> {
    let distance_m = (input_position_mm - output_position_mm) / 1000.0;
    let distance_norm_m = distance_m.mapv(f32::abs).sum();

    distance_m / distance_norm_m
}

#[cfg(test)]
mod test {
    use ndarray::{arr1, Array1};

    use super::calculate;

    #[test]
    fn calculate_direction_simple() {
        let output_position_mm: Array1<f32> = arr1(&[1.0, 1.0, 1.0]);
        let input_position_mm: Array1<f32> = arr1(&[2.0, 1.0, 1.0]);
        let _expected: Array1<f32> = arr1(&[1.0, 0.0, 0.0]);

        let _direction = calculate(&input_position_mm.view(), &output_position_mm.view());

        //TODO: readd test
        //assert_close_l1!(&expected, &direction, 0.001);
    }

    #[test]
    fn calculate_direction_diag() {
        let output_position_mm: Array1<f32> = arr1(&[1.0, 1.0, 1.0]);
        let input_position_mm: Array1<f32> = arr1(&[2.0, 0.0, 2.0]);
        let _expected: Array1<f32> = arr1(&[1.0 / 3.0, -1.0 / 3.0, 1.0 / 3.0]);

        let _direction = calculate(&input_position_mm.view(), &output_position_mm.view());

        //TODO: readd test
        //assert_close_l1!(&expected, &direction, 0.001);
    }
}
