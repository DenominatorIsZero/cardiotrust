use ndarray::{ArrayBase, Dim, ViewRepr};

pub fn calculate_delay_s(
    input_position_mm: &ArrayBase<ViewRepr<&f32>, Dim<[usize; 1]>>,
    output_position_mm: &ArrayBase<ViewRepr<&f32>, Dim<[usize; 1]>>,
    propagation_velocity_m_per_s: &f32,
) -> f32 {
    let distance_m = (input_position_mm - output_position_mm) / 1000.0;
    let distance_norm_m = distance_m.mapv(|v| v.powi(2)).sum().sqrt();
    distance_norm_m / *propagation_velocity_m_per_s
}

#[cfg(test)]
mod test {
    use approx::assert_relative_eq;
    use ndarray::{arr1, Array1};

    use super::calculate_delay_s;

    #[test]
    fn calculate_delay_s_1() {
        let input_position_mm: Array1<f32> = arr1(&[1000.0, 0.0, 0.0]);
        let output_position_mm: Array1<f32> = arr1(&[2000.0, 0.0, 0.0]);
        let propagation_velocity_m_per_s = 2.0;

        let delay_s = calculate_delay_s(
            &input_position_mm.view(),
            &output_position_mm.view(),
            &propagation_velocity_m_per_s,
        );

        assert_relative_eq!(delay_s, 0.5)
    }

    #[test]
    fn calculate_delay_s_2() {
        let input_position_mm: Array1<f32> = arr1(&[1000.0, 0.0, 0.0]);
        let output_position_mm: Array1<f32> = arr1(&[4000.0, 4000.0, 0.0]);
        let propagation_velocity_m_per_s = 2.0;

        let delay_s = calculate_delay_s(
            &input_position_mm.view(),
            &output_position_mm.view(),
            &propagation_velocity_m_per_s,
        );

        assert_relative_eq!(delay_s, 2.5)
    }
}
