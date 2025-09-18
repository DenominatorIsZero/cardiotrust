use bevy::prelude::*;
use ndarray::{arr1, s, Array1};
use ndarray_stats::QuantileExt;
use num_traits::FromPrimitive;
use scarlet::{
    color::RGBColor,
    colormap::{ColorMap, ListedColorMap},
};
use tracing::error;
use strum::EnumCount;

use super::{
    cutting_plane::CuttingPlaneSettings,
    options::{ColorMode, ColorOptions, VisibilityOptions},
    sample_tracker::SampleTracker,
};
use crate::{
    core::{model::spatial::voxels::VoxelType, scenario::Scenario},
    vis::options::ColorSource,
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
#[tracing::instrument(level = "trace", skip_all)]
pub(crate) fn setup_material_atlas(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut voxel_types: Vec<Handle<StandardMaterial>> = Vec::with_capacity(VoxelType::COUNT);

    for i in 0..VoxelType::COUNT {
        let Some(voxel_type) = VoxelType::from_usize(i) else {
            error!("Invalid voxel type index: {}", i);
            continue;
        };
        voxel_types.push(materials.add(StandardMaterial {
            base_color: type_to_color(voxel_type),
            metallic: 0.0,
            ..Default::default()
        }));
    }

    let mut scalar = Vec::with_capacity(256);

    let color_map = ListedColorMap::viridis();

    for i in 0..256 {
        let color: RGBColor = color_map.transform_single(i as f64 / 255.0);
        let color = Color::srgb(color.r as f32, color.g as f32, color.b as f32);
        scalar.push(materials.add(StandardMaterial {
            base_color: color,
            metallic: 0.0,
            ..Default::default()
        }));
    }

    let voxel_types_array: [Handle<StandardMaterial>; VoxelType::COUNT] = match voxel_types.try_into() {
        Ok(array) => array,
        Err(v) => {
            let vec_len = v.len();
            error!("Failed to convert voxel_types vector to array, got {} items, expected {}", vec_len, VoxelType::COUNT);
            // Create default array with proper size
            std::array::from_fn(|_| materials.add(StandardMaterial::default()))
        }
    };

    let scalar_array: [Handle<StandardMaterial>; 256] = match scalar.try_into() {
        Ok(array) => array,
        Err(v) => {
            let vec_len = v.len();
            error!("Failed to convert scalar vector to array, got {} items, expected 256", vec_len);
            // Create default array with proper size
            std::array::from_fn(|_| materials.add(StandardMaterial::default()))
        }
    };

    let atlas = MaterialAtlas {
        voxel_types: voxel_types_array,
        scalar: scalar_array,
    };
    commands.insert_resource(atlas);
}

#[tracing::instrument(level = "trace", skip_all)]
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
    voxels: &Query<(Entity, &VoxelData)>,
) {
    debug!("Running system to initialize voxel components.");
    // Despawn current voxels
    for (entity, _) in voxels.iter() {
        commands.entity(entity).despawn();
    }
    let Some(data) = scenario.data.as_ref() else {
        error!("No scenario data available for voxel initialization");
        return;
    };
    let model = &data.simulation.model;
    let voxels = &model.spatial_description.voxels;
    let voxel_count = model.spatial_description.voxels.count_xyz();
    info!("Voxel count: {voxel_count:?}");
    let size = voxels.size_mm;
    info!("Voxel size: {size:?}");

    let half_size = Vec3::new(
        voxels.size_mm / 2.0,
        voxels.size_mm / 2.0,
        voxels.size_mm / 2.0,
    );

    meshes.remove(&mesh_atlas.voxels);

    let mesh = meshes.add(Mesh::from(Cuboid { half_size }));
    mesh_atlas.voxels = mesh.clone();
    for x in 0..voxel_count[0] {
        for y in 0..voxel_count[1] {
            for z in 0..voxel_count[2] {
                let voxel_type = voxels.types[(x, y, z)];
                if !voxel_type.is_connectable() {
                    continue;
                }
                let position = voxels.positions_mm.slice(s!(x, y, z, ..));
                commands.spawn((
                    Mesh3d(mesh.clone()),
                    MeshMaterial3d(materials.voxel_types[voxel_type as usize].clone()),
                    Transform::from_xyz(position[0], position[1], position[2]),
                    VoxelData {
                        index: match voxels.numbers[(x, y, z)] {
                            Some(num) => num,
                            None => {
                                error!("No voxel number assigned at position ({}, {}, {})", x, y, z);
                                continue;
                            }
                        },
                        colors: Array1::from_elem(
                            sample_tracker.max_sample,
                            materials.voxel_types[voxel_type as usize].clone(),
                        ),
                        position_xyz: arr1(&[x, y, z]),
                        posision_mm: Vec3 {
                            x: position[0],
                            y: position[1],
                            z: position[2],
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
    mut query: Query<(&mut MeshMaterial3d<StandardMaterial>, &VoxelData)>,
) {
    trace!("Running system to update heart voxel colors.");
    query.par_iter_mut().for_each(|(mut material, data)| {
        material.0 = data.colors[sample_tracker.current_sample].clone();
    });
}

#[allow(clippy::needless_pass_by_value)]
#[tracing::instrument(level = "trace", skip_all)]
pub(crate) fn update_heart_voxel_visibility(
    mut voxels: Query<(&mut Visibility, &VoxelData)>,
    cutting_plane: Res<CuttingPlaneSettings>,
    options: Res<VisibilityOptions>,
) {
    if cutting_plane.is_changed() || options.is_changed() {
        for (mut visibility, data) in &mut voxels {
            if options.heart && voxel_is_visible(data.posision_mm, &cutting_plane) {
                *visibility = Visibility::Visible;
            } else {
                *visibility = Visibility::Hidden;
            }
        }
    }
}

#[tracing::instrument(level = "trace", skip_all)]
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
pub fn on_color_mode_changed(
    color_options: Res<ColorOptions>,
    query: Query<&mut VoxelData>,
    scenario_list: Res<ScenarioList>,
    selected_scenario: Res<SelectedSenario>,
    materials: Res<MaterialAtlas>,
) {
    trace!("Running system to change visualization mode.");
    if selected_scenario.index.is_none() {
        return;
    }
    if !color_options.is_changed() {
        return;
    }
    debug!("Visualization mode changed to {:?}.", color_options.mode);
    let Some(index) = selected_scenario.index else {
        error!("No scenario selected for color mode change");
        return;
    };
    let Some(entry) = scenario_list.entries.get(index) else {
        error!("Selected scenario index {} is out of bounds", index);
        return;
    };
    let scenario = &entry.scenario;

    match color_options.mode {
        ColorMode::EstimationVoxelTypes => {
            set_heart_voxel_colors_to_types(query, materials, scenario, false);
        }
        ColorMode::SimulationVoxelTypes => {
            set_heart_voxel_colors_to_types(query, materials, scenario, true);
        }
        ColorMode::EstimatedCdeNorm => {
            set_heart_voxel_colors_to_norm(query, materials, scenario, false);
        }
        ColorMode::SimulatedCdeNorm => {
            set_heart_voxel_colors_to_norm(query, materials, scenario, true);
        }
        ColorMode::EstimatedCdeMax => {
            set_heart_voxel_colors_to_max(
                query,
                materials,
                scenario,
                ColorSource::Estimation,
                color_options.relative_coloring,
            );
        }
        ColorMode::SimulatedCdeMax => {
            set_heart_voxel_colors_to_max(
                query,
                materials,
                scenario,
                ColorSource::Simulation,
                color_options.relative_coloring,
            );
        }
        ColorMode::DeltaCdeMax => {
            set_heart_voxel_colors_to_max(
                query,
                materials,
                scenario,
                ColorSource::Delta,
                color_options.relative_coloring,
            );
        }
        ColorMode::EstimatedActivationTime => {
            set_heart_voxel_colors_to_activation_time(
                query,
                materials,
                scenario,
                ColorSource::Estimation,
                color_options.relative_coloring,
            );
        }
        ColorMode::SimulatedActivationTime => {
            set_heart_voxel_colors_to_activation_time(
                query,
                materials,
                scenario,
                ColorSource::Simulation,
                color_options.relative_coloring,
            );
        }
        ColorMode::DeltaActivationTime => {
            set_heart_voxel_colors_to_activation_time(
                query,
                materials,
                scenario,
                ColorSource::Delta,
                color_options.relative_coloring,
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
        match scenario.data.as_ref() {
            Some(data) => &data.simulation.model.spatial_description.voxels.types,
            None => {
                error!("No simulation data available for voxel type visualization");
                return;
            }
        }
    } else {
        match scenario.results.as_ref().and_then(|r| r.model.as_ref()) {
            Some(model) => &model.spatial_description.voxels.types,
            None => {
                error!("No estimation model available for voxel type visualization");
                return;
            }
        }
    };

    query.par_iter_mut().for_each(|mut data| {
        for sample in 0..data.colors.shape()[0] {
            let x = data.position_xyz[0];
            let y = data.position_xyz[1];
            let z = data.position_xyz[2];
            data.colors[sample] = materials.voxel_types[voxel_types[(x, y, z)] as usize].clone();
        }
    });
}

/// Maps `VoxelType` enum variants to RGBA colors. Used to colorize voxels in the visualization based on voxel type.
#[must_use]
pub const fn type_to_color(voxel_type: VoxelType) -> Color {
    let alpha = 1.0;
    match voxel_type {
        VoxelType::None => Color::Srgba(Srgba {
            red: 1.0,
            green: 1.0,
            blue: 1.0,
            alpha: 0.0,
        }),
        VoxelType::Sinoatrial => Color::Srgba(Srgba {
            red: 1.0,
            green: 0.776,
            blue: 0.118,
            alpha,
        }),
        VoxelType::Atrium => Color::Srgba(Srgba {
            red: 0.686,
            green: 0.345,
            blue: 0.541,
            alpha,
        }),
        VoxelType::Atrioventricular | VoxelType::Vessel => Color::Srgba(Srgba {
            red: 0.0,
            green: 0.804,
            blue: 0.424,
            alpha,
        }),
        VoxelType::HPS => Color::Srgba(Srgba {
            red: 0.0,
            green: 0.604,
            blue: 0.871,
            alpha,
        }),
        VoxelType::Ventricle => Color::Srgba(Srgba {
            red: 1.0,
            green: 0.122,
            blue: 0.357,
            alpha,
        }),
        VoxelType::Pathological => Color::Srgba(Srgba {
            red: 0.651,
            green: 0.463,
            blue: 0.114,
            alpha,
        }),
        VoxelType::Torso => Color::Srgba(Srgba {
            red: 0.63,
            green: 0.69,
            blue: 0.73,
            alpha,
        }),
        VoxelType::Chamber => Color::Srgba(Srgba {
            red: 0.12,
            green: 0.35,
            blue: 0.54,
            alpha,
        }),
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
        match scenario.data.as_ref() {
            Some(data) => &data.simulation.system_states_spherical.magnitude,
            None => {
                error!("No simulation data available for norm visualization");
                return;
            }
        }
    } else {
        match scenario.results.as_ref() {
            Some(results) => &results.estimations.system_states_spherical.magnitude,
            None => {
                error!("No estimation results available for norm visualization");
                return;
            }
        }
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
    source: ColorSource,
    relative_coloring: bool,
) {
    debug!("Setting heart voxel colors to max.");
    let system_states = match source {
        ColorSource::Simulation => {
            match scenario.data.as_ref() {
                Some(data) => &data.simulation.system_states_spherical_max.magnitude,
                None => {
                    error!("No simulation data available for max magnitude visualization");
                    return;
                }
            }
        }
        ColorSource::Estimation => {
            match scenario.results.as_ref() {
                Some(results) => &results.estimations.system_states_spherical_max.magnitude,
                None => {
                    error!("No estimation results available for max magnitude visualization");
                    return;
                }
            }
        }
        ColorSource::Delta => {
            match scenario.results.as_ref() {
                Some(results) => &results.estimations.system_states_spherical_max_delta.magnitude,
                None => {
                    error!("No estimation delta results available for max magnitude visualization");
                    return;
                }
            }
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
    source: ColorSource,
    relative_coloring: bool,
) {
    debug!("Setting heart voxel colors to max.");
    let activation_time_ms = match source {
        ColorSource::Simulation => {
            match scenario.data.as_ref() {
                Some(data) => &data.simulation.activation_times,
                None => {
                    error!("No simulation data available for activation time visualization");
                    return;
                }
            }
        }
        ColorSource::Estimation => {
            match scenario.results.as_ref() {
                Some(results) => &results.estimations.activation_times,
                None => {
                    error!("No estimation results available for activation time visualization");
                    return;
                }
            }
        }
        ColorSource::Delta => {
            match scenario.results.as_ref() {
                Some(results) => &results.estimations.activation_times_delta,
                None => {
                    error!("No estimation delta results available for activation time visualization");
                    return;
                }
            }
        }
    };

    let mut offset = 0.0;
    let scaling = if relative_coloring {
        let max: f32 = *activation_time_ms.max_skipnan();
        let min: f32 = *activation_time_ms.min_skipnan();
        offset = -min;
        1.0 / (max - min)
    } else {
        1.0
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
