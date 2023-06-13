use ndarray::{Array1, ArrayBase, Dim, ViewRepr};

pub fn calculate_direction(
    input_position_mm: &ArrayBase<ViewRepr<&f32>, Dim<[usize; 1]>>,
    output_position_mm: &ArrayBase<ViewRepr<&f32>, Dim<[usize; 1]>>,
) -> Array1<f32> {
    let distance_m = (input_position_mm - output_position_mm) / 1000.0;
    let distance_norm_m = distance_m.mapv(|v| v.abs()).sum();

    distance_m / distance_norm_m
}
