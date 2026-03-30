use super::*;
use crate::core::config::model::{Common, Handcrafted};

const _COMMON_PATH: &str = "tests/core/model/spatial/voxel/";

#[test]
fn count_states_none() {
    let voxels_in_dims = [1000, 1, 1];
    let voxels = Voxels::empty(voxels_in_dims);

    assert_eq!(0, voxels.count_states());
}

#[test]
fn number_of_states_some() {
    let voxels_in_dims = [1000, 1, 1];
    let mut voxels = Voxels::empty(voxels_in_dims);
    voxels.types[(0, 0, 0)] = VoxelType::Atrioventricular;

    assert_eq!(3, voxels.count_states());
}

#[test]
fn no_pathology_full_states() -> Result<()> {
    let config = Model {
        handcrafted: Some(Handcrafted {
            heart_size_mm: [10.0, 10.0, 10.0],
            ..Default::default()
        }),
        common: Common {
            voxel_size_mm: 1.0,
            ..Default::default()
        },
        ..Default::default()
    };
    let voxels = Voxels::from_handcrafted_model_config(&config)?;

    assert_eq!(1000, voxels.count());
    assert_eq!(3000, voxels.count_states());
    Ok(())
}

#[test]
fn is_connection_allowed_true() {
    let output_voxel_type = VoxelType::HPS;
    let input_voxel_type = VoxelType::Ventricle;

    let allowed = is_connection_allowed(&output_voxel_type, &input_voxel_type);

    assert!(allowed);
}

#[test]
fn is_connection_allowed_false() {
    let output_voxel_type = VoxelType::Atrium;
    let input_voxel_type = VoxelType::Ventricle;

    let allowed = is_connection_allowed(&output_voxel_type, &input_voxel_type);

    assert!(!allowed);
}

#[test]
fn some_voxel_types_default() -> Result<()> {
    let config = Model::default();
    let types = VoxelTypes::from_handcrafted_model_config(&config)?;

    let num_sa = types
        .iter()
        .filter(|v_type| **v_type == VoxelType::Sinoatrial)
        .count();

    assert_eq!(num_sa, 1);

    let num_atrium = types
        .iter()
        .filter(|v_type| **v_type == VoxelType::Atrium)
        .count();

    assert!(num_atrium > 0);

    let num_avn = types
        .iter()
        .filter(|v_type| **v_type == VoxelType::Atrioventricular)
        .count();

    assert_eq!(num_avn, 1);

    let num_ventricle = types
        .iter()
        .filter(|v_type| **v_type == VoxelType::Ventricle)
        .count();

    assert!(num_ventricle > 0);

    let num_hps = types
        .iter()
        .filter(|v_type| **v_type == VoxelType::HPS)
        .count();

    assert!(num_hps > 0);

    let num_pathological = types
        .iter()
        .filter(|v_type| **v_type == VoxelType::Pathological)
        .count();

    assert_eq!(num_pathological, 0);
    Ok(())
}
