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
