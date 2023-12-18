use bevy::{math::vec3, prelude::*};
use bevy_aabb_instancing::{Cuboid, CuboidMaterialId, Cuboids};
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
    indices: Array1<usize>,
    colors: Array2<u32>,
    positions: Array2<usize>,
}
/// .
///
/// # Panics
///
/// Panics if data is missing in sceario.
#[allow(clippy::cast_precision_loss)]
pub fn setup_heart_voxels(
    commands: &mut Commands,
    sample_tracker: &mut SampleTracker,
    scenario: &Scenario,
) {
    init_sample_tracker(sample_tracker, scenario);
    init_voxels(commands, scenario, sample_tracker);
}

#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
fn init_voxels(commands: &mut Commands, scenario: &Scenario, sample_tracker: &mut SampleTracker) {
    const PATCH_SIZE: usize = 15;
    const SCENE_RADIUS: f32 = 1000.0;
    let data = scenario.data.as_ref().expect("Data to be some");
    let model = data.get_model();
    let voxels = &model.spatial_description.voxels;
    let voxel_count = model.spatial_description.voxels.count_xyz();
    let x_batches: usize = (voxel_count[0] as f32 / PATCH_SIZE as f32).ceil() as usize;
    let y_batches: usize = (voxel_count[1] as f32 / PATCH_SIZE as f32).ceil() as usize;
    let z_batches: usize = (voxel_count[2] as f32 / PATCH_SIZE as f32).ceil() as usize;
    let offset = arr1(&[voxels.size_mm, voxels.size_mm, voxels.size_mm]) / 2.0;

    for x_batch in 0..x_batches {
        for y_batch in 0..y_batches {
            for z_batch in 0..z_batches {
                let mut instances = Vec::with_capacity(PATCH_SIZE * PATCH_SIZE * PATCH_SIZE);
                let mut indices = Array1::zeros(PATCH_SIZE.pow(3));
                let mut positions_in_grid = Array2::zeros((PATCH_SIZE.pow(3), 3));
                let mut running_index = 0;
                for x_offset in 0..PATCH_SIZE {
                    let x_index = x_batch * PATCH_SIZE + x_offset;
                    if x_index >= voxel_count[0] {
                        break;
                    }
                    for y_offset in 0..PATCH_SIZE {
                        let y_index = y_batch * PATCH_SIZE + y_offset;
                        if y_index >= voxel_count[1] {
                            break;
                        }
                        for z_offset in 0..PATCH_SIZE {
                            let z_index = z_batch * PATCH_SIZE + z_offset;
                            if z_index >= voxel_count[2] {
                                break;
                            }
                            if voxels.types.values[(x_index, y_index, z_index)] == VoxelType::None {
                                break;
                            }
                            let position =
                                voxels
                                    .positions_mm
                                    .values
                                    .slice(s!(x_index, y_index, z_index, ..));
                            let min = &position - &offset;
                            let min = vec3(min[0], min[2], min[1]);
                            let max = &position + &offset;
                            let max = vec3(max[0], max[2], max[1]);
                            let starting_color = Color::as_rgba_u32(Color::Rgba {
                                red: min.x / SCENE_RADIUS,
                                green: min.y / SCENE_RADIUS,
                                blue: min.z / SCENE_RADIUS,
                                alpha: 1.0,
                            });
                            let mut cuboid = Cuboid::new(min, max, starting_color);
                            cuboid.set_depth_bias(0);
                            instances.push(cuboid);
                            indices[running_index] = voxels.numbers.values
                                [(x_index, y_index, z_index)]
                                .expect("Index to be some");
                            positions_in_grid[(running_index, 0)] = x_index;
                            positions_in_grid[(running_index, 1)] = y_index;
                            positions_in_grid[(running_index, 2)] = z_index;
                            running_index += 1;
                        }
                    }
                }
                let cuboids = Cuboids::new(instances);
                let aabb = cuboids.aabb();
                let colors = Array2::zeros((PATCH_SIZE.pow(2), sample_tracker.max_sample));
                let mut voxel_data = VoxelData {
                    indices,
                    colors,
                    positions: positions_in_grid,
                };
                set_heart_voxel_colors_to_types(&cuboids, &mut voxel_data, scenario, true);
                commands.spawn(SpatialBundle::default()).insert((
                    cuboids,
                    aabb,
                    CuboidMaterialId(0),
                    voxel_data,
                ));
            }
        }
    }
}

#[allow(clippy::needless_pass_by_value)]
pub fn update_heart_voxel_colors(
    sample_tracker: Res<SampleTracker>,
    mut query: Query<(&mut Cuboids, &VoxelData)>,
) {
    // read out current sample in color vector and set that to cube
    // maybe use emissive for activation time?
    query
        .par_iter_mut()
        .for_each_mut(|(mut cuboids, voxel_data)| {
            for index in 0..cuboids.instances.len() {
                unsafe {
                    cuboids.instances.get_unchecked_mut(index).color = *voxel_data
                        .colors
                        .uget((index, sample_tracker.current_sample));
                }
            }
        });
}

/// .
///
/// # Panics
///
/// Panics if selected scenario is corrupted.
#[allow(clippy::needless_pass_by_value)]
pub fn on_vis_mode_changed(
    vis_options: Res<VisOptions>,
    mut query: Query<(&Cuboids, &mut VoxelData)>,
    scenario_list: Res<ScenarioList>,
    selected_scenario: Res<SelectedSenario>,
) {
    if selected_scenario.index.is_none() {
        return;
    }
    if !vis_options.is_changed() {
        return;
    }
    let scenario =
        &scenario_list.entries[selected_scenario.index.expect("index to be some.")].scenario;
    query
        .iter_mut()
        .for_each(|(cuboids, mut voxel_data)| match vis_options.mode {
            VisMode::EstimationVoxelTypes => {
                set_heart_voxel_colors_to_types(cuboids, voxel_data.as_mut(), scenario, false);
            }
            VisMode::SimulationVoxelTypes => {
                set_heart_voxel_colors_to_types(cuboids, voxel_data.as_mut(), scenario, true);
            }
            VisMode::EstimatedCdeNorm => {
                set_heart_voxel_colors_to_norm(cuboids, voxel_data.as_mut(), scenario, false);
            }
            VisMode::SimulatedCdeNorm => {
                set_heart_voxel_colors_to_norm(cuboids, voxel_data.as_mut(), scenario, true);
            }
            VisMode::EstimatedCdeMax => {
                set_heart_voxel_colors_to_max(cuboids, voxel_data.as_mut(), scenario, false);
            }
            VisMode::SimulatedCdeMax => {
                set_heart_voxel_colors_to_max(cuboids, voxel_data.as_mut(), scenario, true);
            }
        });
}
#[allow(clippy::needless_pass_by_value)]
fn set_heart_voxel_colors_to_types(
    cuboids: &Cuboids,
    voxel_data: &mut VoxelData,
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

    for index in 0..cuboids.instances.len() {
        for sample in 0..voxel_data.colors.shape()[1] {
            let x = voxel_data.positions[(index, 0)];
            let y = voxel_data.positions[(index, 1)];
            let z = voxel_data.positions[(index, 2)];
            voxel_data.colors[(index, sample)] = type_to_color(voxel_types.values[(x, y, z)]);
        }
    }
}

#[must_use]
pub fn type_to_color(voxel_type: VoxelType) -> u32 {
    let alpha = 1.0;
    match voxel_type {
        VoxelType::None => Color::as_rgba_u32(Color::Rgba {
            red: 1.0,
            green: 1.0,
            blue: 1.0,
            alpha: 0.0,
        }),
        VoxelType::Sinoatrial => Color::as_rgba_u32(Color::Rgba {
            red: 1.0,
            green: 0.776,
            blue: 0.118,
            alpha,
        }),
        VoxelType::Atrium => Color::as_rgba_u32(Color::Rgba {
            red: 0.686,
            green: 0.345,
            blue: 0.541,
            alpha,
        }),
        VoxelType::Atrioventricular => Color::as_rgba_u32(Color::Rgba {
            red: 0.0,
            green: 0.804,
            blue: 0.424,
            alpha,
        }),
        VoxelType::HPS => Color::as_rgba_u32(Color::Rgba {
            red: 0.0,
            green: 0.604,
            blue: 0.871,
            alpha,
        }),
        VoxelType::Ventricle => Color::as_rgba_u32(Color::Rgba {
            red: 1.0,
            green: 0.122,
            blue: 0.357,
            alpha,
        }),
        VoxelType::Pathological => Color::as_rgba_u32(Color::Rgba {
            red: 0.651,
            green: 0.463,
            blue: 0.114,
            alpha,
        }),
    }
}

#[allow(clippy::cast_possible_truncation)]
fn set_heart_voxel_colors_to_norm(
    cuboids: &Cuboids,
    voxel_data: &mut VoxelData,
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

    for index in 0..cuboids.instances.len() {
        let state_index = voxel_data.indices[index];
        for sample in 0..voxel_data.colors.shape()[1] {
            let norm = system_states.values[[sample, state_index]].abs()
                + system_states.values[[sample, state_index + 1]].abs()
                + system_states.values[[sample, state_index + 2]].abs();
            let color: RGBColor = color_map.transform_single(f64::from(norm));
            voxel_data.colors[[index, sample]] = Color::as_rgba_u32(Color::Rgba {
                red: color.r as f32,
                green: color.g as f32,
                blue: color.b as f32,
                alpha: 1.0,
            });
        }
    }
}

#[allow(clippy::cast_possible_truncation)]
fn set_heart_voxel_colors_to_max(
    cuboids: &Cuboids,
    voxel_data: &mut VoxelData,
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
    let mut norm = Array1::zeros(voxel_data.colors.shape()[1]);
    let mut max = 0.0;
    for index in 0..cuboids.instances.len() {
        let state_index = voxel_data.indices[index];
        for sample in 0..voxel_data.colors.shape()[1] {
            norm[sample] = system_states.values[[sample, state_index]].abs()
                + system_states.values[[sample, state_index + 1]].abs()
                + system_states.values[[sample, state_index + 2]].abs();
            max = *norm.max_skipnan();
        }
        let color: RGBColor = color_map.transform_single(f64::from(max));
        let color = Color::as_rgba_u32(Color::Rgba {
            red: color.r as f32,
            green: color.g as f32,
            blue: color.b as f32,
            alpha: 1.0,
        });
        for sample in 0..voxel_data.colors.shape()[1] {
            voxel_data.colors[[index, sample]] = color;
        }
    }
}
