use std::env::consts::DLL_SUFFIX;

use approx::RelativeEq;
use bevy::{math::vec3, prelude::*, utils::petgraph::algo::tred::dag_to_toposorted_adjacency_list};
use ciborium::de;
use nalgebra::{Rotation3, Vector3};
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
    camera: &mut Transform,
    ass: Res<AssetServer>,
) {
    init_sample_tracker(sample_tracker, scenario);
    spawn_sensors(commands, ass, materials, scenario);
    init_voxels(
        commands,
        meshes,
        materials,
        scenario,
        sample_tracker,
        camera,
    );
}

fn spawn_sensors(
    commands: &mut Commands,
    ass: Res<AssetServer>,
    materials: &mut Assets<StandardMaterial>,
    scenario: &Scenario,
) {
    let data = scenario.data.as_ref().expect("Data to be some");
    let model = data.get_model();
    let sensors = &model.spatial_description.sensors;

    // note that we have to include the `Scene0` label
    let shaft_mesh: Handle<Mesh> = ass.load("RoundArrow.glb#Mesh0/Primitive0");
    let point_mesh: Handle<Mesh> = ass.load("RoundArrow.glb#Mesh1/Primitive0");

    for index_sensor in 0..sensors.positions_mm.shape()[0] {
        let pos_x_mm = sensors.positions_mm[(index_sensor, 0)];
        let pos_y_mm = sensors.positions_mm[(index_sensor, 2)];
        let pos_z_mm = sensors.positions_mm[(index_sensor, 1)];
        let ori_x = sensors.orientations_xyz[(index_sensor, 0)];
        let ori_y = sensors.orientations_xyz[(index_sensor, 2)];
        let ori_z = sensors.orientations_xyz[(index_sensor, 1)];

        let from = Vector3::new(0.0, 0.0, 1.0);
        let to = Vector3::new(ori_x, ori_y, ori_z);
        let rot = Rotation3::rotation_between(&to, &from).expect("Rotation matrix to exist");
        let (rot_x, rot_y, rot_z) = rot.euler_angles();

        commands.spawn(PbrBundle {
            mesh: shaft_mesh.clone(),
            // Notice how there is no need to set the `alpha_mode` explicitly here.
            // When converting a color to a material using `into()`, the alpha mode is
            // automatically set to `Blend` if the alpha channel is anything lower than 1.0.
            material: materials.add(Color::rgba(ori_x, ori_z, ori_y, 1.0).into()),
            transform: Transform::from_xyz(pos_x_mm, pos_y_mm, pos_z_mm)
                .with_scale(Vec3::ONE * 10.0)
                .with_rotation(Quat::from_euler(EulerRot::XYZ, rot_x, rot_y, rot_z)),
            ..default()
        });
        commands.spawn(PbrBundle {
            mesh: point_mesh.clone(),
            // Notice how there is no need to set the `alpha_mode` explicitly here.
            // When converting a color to a material using `into()`, the alpha mode is
            // automatically set to `Blend` if the alpha channel is anything lower than 1.0.
            material: materials.add(Color::rgba(ori_x, ori_z, ori_y, 1.0).into()),
            transform: Transform::from_xyz(pos_x_mm, pos_y_mm, pos_z_mm)
                .with_scale(Vec3::ONE * 10.0)
                .with_rotation(Quat::from_euler(EulerRot::XYZ, rot_x, rot_y, rot_z)),
            ..default()
        });
    }

    // to position our 3d model, simply use the Transform
    // in the SceneBundlex
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
    camera: &mut Transform,
) {
    info!("Initializing voxels!");
    let data = scenario.data.as_ref().expect("Data to be some");
    let model = data.get_model();
    let voxels = &model.spatial_description.voxels;
    let voxel_count = model.spatial_description.voxels.count_xyz();
    info!("Voxel count: {voxel_count:?}");
    let size = voxels.size_mm;
    info!("Voxel size: {size:?}");

    let x = voxel_count[0] - 1;
    let y = voxel_count[1] - 1;
    let z = voxel_count[2] - 1;
    let position = voxels.positions_mm.values.slice(s!(x, y, z, ..));
    camera.translation.x = position[0]; //position[0] + 20.0;
    camera.translation.z = position[2];
    camera.translation.y = position[1] + 100.0;
    let x = voxel_count[0] / 2;
    let y = voxel_count[1] / 2;
    let z = voxel_count[2] / 2;
    let position = voxels.positions_mm.values.slice(s!(x, y, z, ..));
    camera.look_at(
        Vec3 {
            x: position[0],
            y: position[2],
            z: position[1],
        },
        Vec3::Y,
    );

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
            set_heart_voxel_colors_to_norm(query, scenario, false, vis_options.relative_coloring);
        }
        VisMode::SimulatedCdeNorm => {
            set_heart_voxel_colors_to_norm(query, scenario, true, vis_options.relative_coloring);
        }
        VisMode::EstimatedCdeMax => {
            set_heart_voxel_colors_to_max(query, scenario, false, vis_options.relative_coloring);
        }
        VisMode::SimulatedCdeMax => {
            set_heart_voxel_colors_to_max(query, scenario, true, vis_options.relative_coloring);
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
    relative_coloring: bool,
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

    let mut offset = 0.0;
    let mut scaling = 1.0;

    if relative_coloring {
        let mut norm = Array1::zeros(system_states.values.shape()[0]);
        let mut max: f32 = 0.0;
        let mut min: f32 = 10000.0;
        for state in (0..system_states.values.shape()[1]).step_by(3) {
            for sample in 0..system_states.values.shape()[0] {
                norm[sample] = system_states.values[[sample, state]].abs()
                    + system_states.values[[sample, state + 1]].abs()
                    + system_states.values[[sample, state + 2]].abs();
            }
            max = max.max(*norm.max_skipnan());
            min = min.min(*norm.max_skipnan());
        }
        if (max - min) > 0.01 {
            offset = -min;
            scaling = 1.0 / (max - min);
        }
    }

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
    relative_coloring: bool,
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

    let mut offset = 0.0;
    let mut scaling = 1.0;

    if relative_coloring {
        let mut norm = Array1::zeros(system_states.values.shape()[0]);
        let mut max: f32 = 0.0;
        let mut min: f32 = 10000.0;
        for state in (0..system_states.values.shape()[1]).step_by(3) {
            for sample in 0..system_states.values.shape()[0] {
                norm[sample] = system_states.values[[sample, state]].abs()
                    + system_states.values[[sample, state + 1]].abs()
                    + system_states.values[[sample, state + 2]].abs();
            }
            max = max.max(*norm.max_skipnan());
            min = min.min(*norm.max_skipnan());
        }
        if (max - min) > 0.01 {
            offset = -min;
            scaling = 1.0 / (max - min);
        }
    }

    for mut data in &mut query {
        let mut norm = Array1::zeros(data.colors.shape()[0]);
        let mut max = 0.0;
        let state = data.index;
        for sample in 0..data.colors.shape()[0] {
            norm[sample] = system_states.values[[sample, state]].abs()
                + system_states.values[[sample, state + 1]].abs()
                + system_states.values[[sample, state + 2]].abs();
        }
        max = *norm.max_skipnan();
        max = (max + offset) * scaling;
        let color: RGBColor = color_map.transform_single(f64::from(max));
        let color = Color::Rgba {
            red: color.r as f32,
            green: color.g as f32,
            blue: color.b as f32,
            alpha: 1.0,
        };
        for sample in 0..data.colors.shape()[0] {
            data.colors[sample] = color;
        }
    }
}
