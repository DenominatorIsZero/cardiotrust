use approx::relative_eq;
use ndarray::{Array2, ArrayBase, Dim, OwnedRepr, ViewRepr};

pub fn calculate(
    input_direction: &ArrayBase<OwnedRepr<f32>, Dim<[usize; 1]>>,
    output_direction: ArrayBase<ViewRepr<&f32>, Dim<[usize; 1]>>,
) -> Array2<f32> {
    let mut gain = Array2::<f32>::zeros((3, 3));

    for input_dimension in 0..3 {
        for output_dimension in 0..3 {
            let c = output_direction[output_dimension];
            let mult = if relative_eq!(c, 0.0) {
                0.0
            } else {
                c.signum()
            };
            gain[(input_dimension, output_dimension)] = input_direction[input_dimension] * mult;
        }
    }

    gain
}

#[cfg(test)]
mod test {
    use ndarray::{arr1, Array2};

    use super::calculate;

    #[test]
    fn calculate_gain_same_direction() {
        let output_direction = arr1(&[1.0, 0.0, 0.0]);
        let input_direction = arr1(&[1.0, 0.0, 0.0]);

        let gain = calculate(&input_direction, output_direction.view());

        let mut expected_gain = Array2::<f32>::zeros((3, 3));
        expected_gain[(0, 0)] = 1.0;

        // TODO: readd test.
        //assert_close_l1!(&gain, &expected_gain, 0.01);
    }

    #[test]
    fn calculate_gain_opposite_direction() {
        let output_direction = arr1(&[1.0, 0.0, 0.0]);
        let input_direction = arr1(&[-1.0, 0.0, 0.0]);

        let gain = calculate(&input_direction, output_direction.view());

        let mut expected_gain = Array2::<f32>::zeros((3, 3));
        expected_gain[(0, 0)] = -1.0;
        // TODO: readd test
        //assert_close_l1!(&gain, &expected_gain, 0.01);
    }

    #[test]
    fn calculate_gain_some_direction() {
        let output_direction = arr1(&[0.5, -0.5, 0.0]);
        let input_direction = arr1(&[-0.5, 0.0, 0.5]);

        let gain = calculate(&input_direction, output_direction.view());

        let mut expected_gain = Array2::<f32>::zeros((3, 3));
        expected_gain[(0, 0)] = -0.5;
        expected_gain[(0, 1)] = 0.5;
        expected_gain[(2, 0)] = 0.5;
        expected_gain[(2, 1)] = -0.5;

        // TODO: readd test
        //assert_close_l1!(&gain, &expected_gain, 0.01);
    }
    #[test]
    fn calculate_gain_three_to_one_direction() {
        let output_direction = arr1(&[0.2, -0.5, 0.3]);
        let input_direction = arr1(&[1.0, 0.0, 0.0]);

        let gain = calculate(&input_direction, output_direction.view());

        let mut expected_gain = Array2::<f32>::zeros((3, 3));
        expected_gain[(0, 0)] = 1.0;
        expected_gain[(0, 1)] = -1.0;
        expected_gain[(0, 2)] = 1.0;

        // TODO: readd test
        //assert_close_l1!(&gain, &expected_gain, 0.01);
    }
}
