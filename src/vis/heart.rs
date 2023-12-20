use bevy::{math::vec3, prelude::*};
use ciborium::de;
use ndarray::{arr1, s, Array1, Array2};
use ndarray_stats::QuantileExt;
use scarlet::{
    color::RGBColor,
    colormap::{ColorMap, ListedColorMap},
};

use super::{
    options::{VisMode, VisOptions},
    sample_tracker::{init_sample_tracker, SampleTracker},
};
use crate::{
    core::{model::spatial::voxels::VoxelType, scenario::Scenario},
    ScenarioList, SelectedSenario,
};

#[derive(Component)]
pub struct VoxelData {
    index: usize,
    colors: Array1<Color>,
    position: Array1<usize>,
}
/// .
///
/// # Panics
///
/// Panics if data is missing in sceario.
#[allow(clippy::cast_precision_loss)]
pub fn setup_heart_voxels(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    sample_tracker: &mut SampleTracker,
    scenario: &Scenario,
) {
    init_sample_tracker(sample_tracker, scenario);
    init_voxels(commands, meshes, materials, scenario, sample_tracker);
}

#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
fn init_voxels(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    scenario: &Scenario,
    sample_tracker: &SampleTracker,
) {
    let data = scenario.data.as_ref().expect("Data to be some");
    let model = data.get_model();
    let voxels = &model.spatial_description.voxels;
    let voxel_count = model.spatial_description.voxels.count_xyz();

    let mesh = meshes.add(Mesh::from(shape::Cube {
        size: voxels.size_mm,
    }));
    for x in 0..voxel_count[0] {
        for y in 0..voxel_count[1] {
            for z in 0..voxel_count[2] {
                if voxels.types.values[(x, y, z)] == VoxelType::None {
                    break;
                }
                let position = voxels.positions_mm.values.slice(s!(x, y, z, ..));
                commands.spawn((
                    PbrBundle {
                        mesh: mesh.clone(),
                        material: materials.add(Color::rgb(x as f32, y as f32, z as f32).into()),
                        transform: Transform::from_xyz(position[0], position[2], position[1]),
                        ..default()
                    },
                    VoxelData {
                        index: voxels.numbers.values[(x, y, z)].expect("Voxel numbes to be some."),
                        colors: Array1::from_elem(
                            sample_tracker.max_sample,
                            Color::rgb(x as f32, y as f32, z as f32),
                        ),
                        position: arr1(&[x, y, z]),
                    },
                ));
            }
        }
    }
}

/// # Panics
/// if material doesnt exist
#[allow(clippy::needless_pass_by_value)]
pub fn update_heart_voxel_colors(
    sample_tracker: Res<SampleTracker>,
    mut query: Query<(&Handle<StandardMaterial>, &VoxelData)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (handle, data) in &mut query {
        let material = materials.get_mut(handle).unwrap();
        material.base_color = data.colors[sample_tracker.current_sample];
    }
}

/// .
///
/// # Panics
///
/// Panics if selected scenario is corrupted.
#[allow(clippy::needless_pass_by_value)]
pub fn on_vis_mode_changed(
    vis_options: Res<VisOptions>,
    mut query: Query<(&mut VoxelData)>,
    scenario_list: Res<ScenarioList>,
    selected_scenario: Res<SelectedSenario>,
) {
    if selected_scenario.index.is_none() {
        return;
    }
    if !vis_options.is_changed() {
        return;
    }
    let index = selected_scenario.index.expect("Index to be some");
    let len = scenario_list.entries.len();
    info!("index {index}, len {len}");
    let scenario =
        &scenario_list.entries[selected_scenario.index.expect("index to be some.")].scenario;

    match vis_options.mode {
        VisMode::EstimationVoxelTypes => {
            set_heart_voxel_colors_to_types(query, scenario, false);
        }
        VisMode::SimulationVoxelTypes => {
            set_heart_voxel_colors_to_types(query, scenario, true);
        }
        VisMode::EstimatedCdeNorm => {
            set_heart_voxel_colors_to_norm(query, scenario, false);
        }
        VisMode::SimulatedCdeNorm => {
            set_heart_voxel_colors_to_norm(query, scenario, true);
        }
        VisMode::EstimatedCdeMax => {
            set_heart_voxel_colors_to_max(query, scenario, false);
        }
        VisMode::SimulatedCdeMax => {
            set_heart_voxel_colors_to_max(query, scenario, true);
        }
    }
}
#[allow(clippy::needless_pass_by_value)]
fn set_heart_voxel_colors_to_types(
    mut query: Query<(&mut VoxelData)>,
    scenario: &Scenario,
    simulation_not_model: bool,
) {
    let voxel_types = if simulation_not_model {
        scenario
            .data
            .as_ref()
            .expect("Data to be some")
            .get_voxel_types()
    } else {
        &scenario
            .results
            .as_ref()
            .expect("Results to be some.")
            .model
            .as_ref()
            .expect("Model to be some.")
            .spatial_description
            .voxels
            .types
    };

    for mut data in &mut query {
        for sample in 0..data.colors.shape()[0] {
            let x = data.position[0];
            let y = data.position[1];
            let z = data.position[2];
            data.colors[sample] = type_to_color(voxel_types.values[(x, y, z)]);
        }
    }
}

#[must_use]
const fn type_to_color(voxel_type: VoxelType) -> Color {
    let alpha = 1.0;
    match voxel_type {
        VoxelType::None => Color::Rgba {
            red: 1.0,
            green: 1.0,
            blue: 1.0,
            alpha: 0.0,
        },
        VoxelType::Sinoatrial => Color::Rgba {
            red: 1.0,
            green: 0.776,
            blue: 0.118,
            alpha,
        },
        VoxelType::Atrium => Color::Rgba {
            red: 0.686,
            green: 0.345,
            blue: 0.541,
            alpha,
        },
        VoxelType::Atrioventricular => Color::Rgba {
            red: 0.0,
            green: 0.804,
            blue: 0.424,
            alpha,
        },
        VoxelType::HPS => Color::Rgba {
            red: 0.0,
            green: 0.604,
            blue: 0.871,
            alpha,
        },
        VoxelType::Ventricle => Color::Rgba {
            red: 1.0,
            green: 0.122,
            blue: 0.357,
            alpha,
        },
        VoxelType::Pathological => Color::Rgba {
            red: 0.651,
            green: 0.463,
            blue: 0.114,
            alpha,
        },
    }
}

#[allow(clippy::cast_possible_truncation)]
fn set_heart_voxel_colors_to_norm(
    mut query: Query<(&mut VoxelData)>,
    scenario: &Scenario,
    simulation_not_model: bool,
) {
    let system_states = if simulation_not_model {
        scenario
            .data
            .as_ref()
            .expect("Data to be some")
            .get_system_states()
    } else {
        &scenario
            .results
            .as_ref()
            .expect("Results to be some.")
            .estimations
            .system_states
    };
    let color_map = ListedColorMap::viridis();

    for mut data in &mut query {
        let state = data.index;
        for sample in 0..data.colors.shape()[0] {
            let norm = system_states.values[[sample, state]].abs()
                + system_states.values[[sample, state + 1]].abs()
                + system_states.values[[sample, state + 2]].abs();
            let color: RGBColor = color_map.transform_single(f64::from(norm));
            data.colors[sample] = Color::Rgba {
                red: color.r as f32,
                green: color.g as f32,
                blue: color.b as f32,
                alpha: 1.0,
            }
        }
    }
}

#[allow(clippy::cast_possible_truncation)]
fn set_heart_voxel_colors_to_max(
    mut query: Query<(&mut VoxelData)>,
    scenario: &Scenario,
    simulation_not_model: bool,
) {
    let system_states = if simulation_not_model {
        scenario
            .data
            .as_ref()
            .expect("Data to be some")
            .get_system_states()
    } else {
        &scenario
            .results
            .as_ref()
            .expect("Results to be some.")
            .estimations
            .system_states
    };
    let color_map = ListedColorMap::viridis();
    for mut data in &mut query {
        let mut norm = Array1::zeros(data.colors.shape()[1]);
        let mut max = 0.0;
        let state = data.index;
        for sample in 0..data.colors.shape()[0] {
            norm[sample] = system_states.values[[sample, state]].abs()
                + system_states.values[[sample, state + 1]].abs()
                + system_states.values[[sample, state + 2]].abs();
        }
        max = *norm.max_skipnan();
        let color: RGBColor = color_map.transform_single(f64::from(max));
        let color = Color::Rgba {
            red: color.r as f32,
            green: color.g as f32,
            blue: color.b as f32,
            alpha: 1.0,
        };
        for sample in 0..data.colors.shape()[1] {
            data.colors[sample] = color;
        }
    }
}
