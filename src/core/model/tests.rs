use ndarray::s;

use super::Model;
use crate::core::config;

#[test]
fn test_ap_gain_init_sum_default() {
    let config = &config::model::Model::default();
    let sample_rate_hz = 2000.0;
    let duration_s = 1.0;

    let model = Model::from_model_config(config, sample_rate_hz, duration_s).unwrap();

    let x_y_z = model.spatial_description.voxels.count_xyz();

    for x in 0..x_y_z[0] {
        for y in 0..x_y_z[1] {
            for z in 0..x_y_z[2] {
                if !model.spatial_description.voxels.types[(x, y, z)].is_connectable() {
                    continue;
                }
                let state = model.spatial_description.voxels.numbers[(x, y, z)].unwrap();

                let row_x = model
                    .functional_description
                    .ap_params
                    .gains
                    .slice(s![state, ..]);
                let row_y = model
                    .functional_description
                    .ap_params
                    .gains
                    .slice(s![state + 1, ..]);
                let row_z = model
                    .functional_description
                    .ap_params
                    .gains
                    .slice(s![state + 2, ..]);
                let sum_x =
                    row_x.mapv(f32::abs).sum() + model.functional_description.control_matrix[state];
                let sum_y = row_y.mapv(f32::abs).sum()
                    + model.functional_description.control_matrix[state + 1];
                let sum_z = row_z.mapv(f32::abs).sum()
                    + model.functional_description.control_matrix[state + 2];

                let sum = sum_x + sum_y + sum_z;

                assert!(
            0.99 < sum && sum < 3.01,
            "sum: {sum}, state: {state}, sum_x: {sum_x}, sum_y: {sum_y}, sum_z: {sum_z}, gains_x: {row_x}, gains_y: {row_y}, gains_z: {row_z}",
        );
            }
        }
    }
}

#[test]
fn test_ap_gain_init_sum_mri() {
    let mut config = config::model::Model::default();
    config.handcrafted = None;
    config.mri = Some(config::model::Mri::default());

    let sample_rate_hz = 2000.0;
    let duration_s = 1.0;

    let model = Model::from_model_config(&config, sample_rate_hz, duration_s).unwrap();

    let x_y_z = model.spatial_description.voxels.count_xyz();

    for x in 0..x_y_z[0] {
        for y in 0..x_y_z[1] {
            for z in 0..x_y_z[2] {
                if !model.spatial_description.voxels.types[(x, y, z)].is_connectable() {
                    continue;
                }
                let state = model.spatial_description.voxels.numbers[(x, y, z)].unwrap();

                let row_x = model
                    .functional_description
                    .ap_params
                    .gains
                    .slice(s![state, ..]);
                let row_y = model
                    .functional_description
                    .ap_params
                    .gains
                    .slice(s![state + 1, ..]);
                let row_z = model
                    .functional_description
                    .ap_params
                    .gains
                    .slice(s![state + 2, ..]);
                let sum_x =
                    row_x.mapv(f32::abs).sum() + model.functional_description.control_matrix[state];
                let sum_y = row_y.mapv(f32::abs).sum()
                    + model.functional_description.control_matrix[state + 1];
                let sum_z = row_z.mapv(f32::abs).sum()
                    + model.functional_description.control_matrix[state + 2];

                let sum = sum_x + sum_y + sum_z;

                assert!(
            0.99 < sum && sum < 3.01,
            "sum: {sum}, state: {state}, sum_x: {sum_x}, sum_y: {sum_y}, sum_z: {sum_z}, gains_x: {row_x}, gains_y: {row_y}, gains_z: {row_z}",
        );
            }
        }
    }
}
