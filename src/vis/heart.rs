use bevy::prelude::*;
use ndarray::{arr1, s, Array1};
use ndarray_stats::QuantileExt;
use num_traits::FromPrimitive;
use scarlet::{
    color::RGBColor,
    colormap::{ColorMap, ListedColorMap},
};
use strum::EnumCount;

use super::{
    cutting_plane::CuttingPlaneSettings,
    options::{VisMode, VisOptions},
    sample_tracker::SampleTracker,
};
use crate::{
    core::{model::spatial::voxels::VoxelType, scenario::Scenario},
    vis::options::VisSource,
    ScenarioList, SelectedSenario,
};

#[derive(Component)]
pub struct VoxelData {
    index: usize,
    colors: Array1<Handle<StandardMaterial>>,
    position_xyz: Array1<usize>,
    posision_mm: Vec3,
}

#[derive(Resource)]
pub struct MaterialAtlas {
    pub voxel_types: [Handle<StandardMaterial>; VoxelType::COUNT],
    pub scalar: [Handle<StandardMaterial>; 256],
}

#[derive(Resource)]
pub struct MeshAtlas {
    pub voxels: Handle<Mesh>,
}

#[allow(clippy::cast_possible_truncation, clippy::cast_lossless)]
pub(crate) fn setup_material_atlas(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut voxel_types: Vec<Handle<StandardMaterial>> = Vec::with_capacity(VoxelType::COUNT);

    for i in 0..VoxelType::COUNT {
        voxel_types.push(materials.add(StandardMaterial {
            base_color: type_to_color(VoxelType::from_usize(i).unwrap()),
            metallic: 0.0,
            ..Default::default()
        }));
    }

    let mut scalar = Vec::with_capacity(256);

    let color_map = ListedColorMap::viridis();

    for i in 0..256 {
        let color: RGBColor = color_map.transform_single(i as f64 / 255.0);
        let color = Color::rgb(color.r as f32, color.g as f32, color.b as f32);
        scalar.push(materials.add(StandardMaterial {
            base_color: color,
            metallic: 0.0,
            ..Default::default()
        }));
    }

    let atlas = MaterialAtlas {
        voxel_types: voxel_types.try_into().unwrap(),
        scalar: scalar.try_into().unwrap(),
    };
    commands.insert_resource(atlas);
}

pub(crate) fn setup_mesh_atlas(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    let mesh = meshes.add(Mesh::from(Cuboid {
        half_size: Vec3::new(1.0, 1.0, 1.0),
    }));
    let atlas = MeshAtlas { voxels: mesh };
    commands.insert_resource(atlas);
}

/// Initializes voxel components by iterating through the voxel grid
/// data and spawning a `PbrBundle` for each voxel. Sets up voxel data
/// component with index, colors, and position. Also positions the
/// camera based on voxel grid bounds.
///
/// # Panics
///
/// Panics if the scenario data is None.
#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
#[tracing::instrument(level = "debug", skip_all)]
pub fn init_voxels(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &Res<MaterialAtlas>,
    mesh_atlas: &mut ResMut<MeshAtlas>,
    scenario: &Scenario,
    sample_tracker: &SampleTracker,
    camera: &mut Transform,
) {
    debug!("Running system to initialize voxel components.");
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

    let half_size = Vec3::new(
        voxels.size_mm / 2.0,
        voxels.size_mm / 2.0,
        voxels.size_mm / 2.0,
    );

    meshes.remove(mesh_atlas.voxels.clone());

    let mesh = meshes.add(Mesh::from(Cuboid { half_size }));
    mesh_atlas.voxels = mesh.clone();
    for x in 0..voxel_count[0] {
        for y in 0..voxel_count[1] {
            for z in 0..voxel_count[2] {
                let voxel_type = voxels.types.values[(x, y, z)];
                if !voxel_type.is_connectable() {
                    continue;
                }
                let position = voxels.positions_mm.values.slice(s!(x, y, z, ..));
                commands.spawn((
                    PbrBundle {
                        mesh: mesh.clone(),
                        material: materials.voxel_types[voxel_type as usize].clone(),
                        transform: Transform::from_xyz(position[0], position[2], position[1]),
                        ..default()
                    },
                    VoxelData {
                        index: voxels.numbers.values[(x, y, z)].expect("Voxel numbes to be some."),
                        colors: Array1::from_elem(
                            sample_tracker.max_sample,
                            materials.voxel_types[voxel_type as usize].clone(),
                        ),
                        position_xyz: arr1(&[x, y, z]),
                        posision_mm: Vec3 {
                            x: position[0],
                            y: position[2],
                            z: position[1],
                        },
                    },
                ));
            }
        }
    }
}

/// Updates the voxel colors in the heart model by getting the current
/// sample color from the sample tracker and setting the material base
/// color to that sample color. This allows the heart model to animate
/// through the different sample colors over time.
#[allow(clippy::needless_pass_by_value)]
#[tracing::instrument(skip(query), level = "trace")]
pub(crate) fn update_heart_voxel_colors(
    sample_tracker: Res<SampleTracker>,
    mut query: Query<(&mut Handle<StandardMaterial>, &VoxelData)>,
) {
    trace!("Running system to update heart voxel colors.");
    query.par_iter_mut().for_each(|(mut handle, data)| {
        *handle = data.colors[sample_tracker.current_sample].clone();
    });
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn update_heart_voxel_visibility(
    mut commands: Commands,
    voxels: Query<(Entity, &VoxelData)>,
    cutting_plane: Res<CuttingPlaneSettings>,
    mesh_atlas: Res<MeshAtlas>,
) {
    if cutting_plane.is_changed() {
        for (entity, data) in voxels.iter() {
            if voxel_is_visible(data.posision_mm, &cutting_plane) {
                commands.entity(entity).insert(PbrBundle {
                    mesh: mesh_atlas.voxels.clone(),
                    material: data.colors[0].clone(),
                    transform: Transform::from_xyz(
                        data.posision_mm[0],
                        data.posision_mm[1],
                        data.posision_mm[2],
                    ),
                    ..Default::default()
                });
            } else {
                commands.entity(entity).remove::<PbrBundle>();
            }
        }
    }
}

fn voxel_is_visible(position: Vec3, cutting_plane: &CuttingPlaneSettings) -> bool {
    !cutting_plane.enabled || ((position - cutting_plane.position).dot(cutting_plane.normal) < 0.0)
}

/// Updates the voxel colors in the heart model based on the current
/// visualization mode and scenario selection. Retrieves the scenario
/// data and uses it to set the voxel colors according to the selected
/// visualization mode.
///
/// # Panics
///
/// Panics if selected scenario is corrupted.
#[allow(clippy::needless_pass_by_value)]
#[tracing::instrument(level = "debug", skip_all)]
pub fn on_vis_mode_changed(
    vis_options: Res<VisOptions>,
    query: Query<&mut VoxelData>,
    scenario_list: Res<ScenarioList>,
    selected_scenario: Res<SelectedSenario>,
    materials: Res<MaterialAtlas>,
) {
    trace!("Running system to change visualization mode.");
    if selected_scenario.index.is_none() {
        return;
    }
    if !vis_options.is_changed() {
        return;
    }
    debug!("Visualization mode changed to {:?}.", vis_options.mode);
    let scenario =
        &scenario_list.entries[selected_scenario.index.expect("index to be some.")].scenario;

    match vis_options.mode {
        VisMode::EstimationVoxelTypes => {
            set_heart_voxel_colors_to_types(query, materials, scenario, false);
        }
        VisMode::SimulationVoxelTypes => {
            set_heart_voxel_colors_to_types(query, materials, scenario, true);
        }
        VisMode::EstimatedCdeNorm => {
            set_heart_voxel_colors_to_norm(query, materials, scenario, false);
        }
        VisMode::SimulatedCdeNorm => {
            set_heart_voxel_colors_to_norm(query, materials, scenario, true);
        }
        VisMode::EstimatedCdeMax => {
            set_heart_voxel_colors_to_max(
                query,
                materials,
                scenario,
                VisSource::Estimation,
                vis_options.relative_coloring,
            );
        }
        VisMode::SimulatedCdeMax => {
            set_heart_voxel_colors_to_max(
                query,
                materials,
                scenario,
                VisSource::Simulation,
                vis_options.relative_coloring,
            );
        }
        VisMode::DeltaCdeMax => {
            set_heart_voxel_colors_to_max(
                query,
                materials,
                scenario,
                VisSource::Delta,
                vis_options.relative_coloring,
            );
        }
        VisMode::EstimatedActivationTime => {
            set_heart_voxel_colors_to_activation_time(
                query,
                materials,
                scenario,
                VisSource::Estimation,
                vis_options.relative_coloring,
            );
        }
        VisMode::SimulatedActivationTime => {
            set_heart_voxel_colors_to_activation_time(
                query,
                materials,
                scenario,
                VisSource::Simulation,
                vis_options.relative_coloring,
            );
        }
        VisMode::DeltaActivationTime => {
            set_heart_voxel_colors_to_activation_time(
                query,
                materials,
                scenario,
                VisSource::Delta,
                vis_options.relative_coloring,
            );
        }
    }
}

/// Sets the voxel colors based on the voxel types from the
/// scenario. Retrieves the voxel types from either the model or simulation
/// results based on the `simulation_not_model` flag.
#[allow(clippy::needless_pass_by_value)]
#[tracing::instrument(level = "debug", skip_all)]
fn set_heart_voxel_colors_to_types(
    mut query: Query<&mut VoxelData>,
    materials: Res<MaterialAtlas>,
    scenario: &Scenario,
    simulation_not_model: bool,
) {
    debug!("Setting heart voxel colors to types.");
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

    query.par_iter_mut().for_each(|mut data| {
        for sample in 0..data.colors.shape()[0] {
            let x = data.position_xyz[0];
            let y = data.position_xyz[1];
            let z = data.position_xyz[2];
            data.colors[sample] =
                materials.voxel_types[voxel_types.values[(x, y, z)] as usize].clone();
        }
    });
}

/// Maps `VoxelType` enum variants to RGBA colors. Used to colorize voxels in the visualization based on voxel type.
#[must_use]
pub const fn type_to_color(voxel_type: VoxelType) -> Color {
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
        VoxelType::Atrioventricular | VoxelType::Vessel => Color::Rgba {
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
        VoxelType::Torso => Color::Rgba {
            red: 0.63,
            green: 0.69,
            blue: 0.73,
            alpha,
        },
        VoxelType::Chamber => Color::Rgba {
            red: 0.12,
            green: 0.35,
            blue: 0.54,
            alpha,
        },
    }
}

/// Sets the voxel colors in the heart visualization to represent
/// the activation norm (sum of absolute values) for the current
/// timestep. Uses the provided scenario data or results to look up
/// system states over time. Applies a Viridis color map to the norm
/// values to generate RGB colors for each voxel.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
#[tracing::instrument(level = "debug", skip_all)]
fn set_heart_voxel_colors_to_norm(
    mut query: Query<&mut VoxelData>,
    materials: Res<MaterialAtlas>,
    scenario: &Scenario,
    simulation_not_model: bool,
) {
    debug!("Setting heart voxel colors to norm.");
    let system_states = if simulation_not_model {
        &scenario
            .data
            .as_ref()
            .expect("Data to be some")
            .simulation
            .as_ref()
            .expect("Simulation to be some")
            .system_states_spherical
            .magnitude
    } else {
        &scenario
            .results
            .as_ref()
            .expect("Results to be some.")
            .estimations
            .system_states_spherical
            .magnitude
    };

    query.par_iter_mut().for_each(|mut data| {
        let state = data.index;
        for sample in 0..data.colors.shape()[0] {
            let norm = system_states[(sample, state / 3)];
            let index = (norm * 255.0) as usize;
            let index = index.clamp(0, 255);
            data.colors[sample] = materials.scalar[index].clone();
        }
    });
}

/// Sets the voxel colors in the heart visualization to the maximum activation
/// (sum of absolute values) for each voxel over time. Applies a Viridis color map
/// to the max values to generate RGB colors for each voxel. Can do relative coloring
/// based on min/max of activation across voxels if `relative_coloring` is true.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
#[tracing::instrument(level = "debug", skip_all)]
fn set_heart_voxel_colors_to_max(
    mut query: Query<&mut VoxelData>,
    materials: Res<MaterialAtlas>,
    scenario: &Scenario,
    source: VisSource,
    relative_coloring: bool,
) {
    debug!("Setting heart voxel colors to max.");
    let system_states = match source {
        VisSource::Simulation => {
            &scenario
                .data
                .as_ref()
                .expect("Data to be some")
                .simulation
                .as_ref()
                .expect("Simulation to be some")
                .system_states_spherical_max
                .magnitude
        }
        VisSource::Estimation => {
            &scenario
                .results
                .as_ref()
                .expect("Results to be some.")
                .estimations
                .system_states_spherical_max
                .magnitude
        }
        VisSource::Delta => {
            &scenario
                .results
                .as_ref()
                .expect("Results to be some.")
                .estimations
                .system_states_spherical_max_delta
                .magnitude
        }
    };

    let mut offset = 0.0;
    let mut scaling = 1.0;

    if relative_coloring {
        let max: f32 = *system_states.max_skipnan();
        let min: f32 = *system_states.min_skipnan();
        if (max - min) > 0.01 {
            offset = -min;
            scaling = 1.0 / (max - min);
        }
    }

    query.par_iter_mut().for_each(|mut data| {
        let state = data.index;
        let mut max = system_states[state / 3];
        max = (max + offset) * scaling;
        let index = (max * 255.0) as usize;
        let index = index.clamp(0, 255);
        for sample in 0..data.colors.shape()[0] {
            data.colors[sample] = materials.scalar[index].clone();
        }
    });
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
#[tracing::instrument(level = "debug", skip_all)]
fn set_heart_voxel_colors_to_activation_time(
    mut query: Query<&mut VoxelData>,
    materials: Res<MaterialAtlas>,
    scenario: &Scenario,
    source: VisSource,
    relative_coloring: bool,
) {
    debug!("Setting heart voxel colors to max.");
    let activation_time_ms = match source {
        VisSource::Simulation => {
            &scenario
                .data
                .as_ref()
                .expect("Data to be some")
                .simulation
                .as_ref()
                .expect("Simulation to be some")
                .activation_times
                .time_ms
        }
        VisSource::Estimation => {
            &scenario
                .results
                .as_ref()
                .expect("Results to be some.")
                .estimations
                .activation_times
                .time_ms
        }
        VisSource::Delta => {
            &scenario
                .results
                .as_ref()
                .expect("Results to be some.")
                .estimations
                .activation_times_delta
                .time_ms
        }
    };

    let mut offset = 0.0;
    let mut scaling = 1.0;

    if relative_coloring {
        let max: f32 = *activation_time_ms.max_skipnan();
        let min: f32 = *activation_time_ms.min_skipnan();
        offset = -min;
        scaling = 1.0 / (max - min);
    };

    query.par_iter_mut().for_each(|mut data| {
        let state = data.index;
        let mut max = activation_time_ms[state / 3];
        max = (max + offset) * scaling;
        let index = (max * 255.0) as usize;
        let index = index.clamp(0, 255);

        for sample in 0..data.colors.shape()[0] {
            data.colors[sample] = materials.scalar[index].clone();
        }
    });
}
