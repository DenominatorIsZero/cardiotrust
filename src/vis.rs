use bevy::{math::vec3, prelude::*};
use bevy_aabb_instancing::{Cuboid, CuboidMaterialId, Cuboids, VertexPullingRenderPlugin};
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use ndarray::{arr1, s, Array1, Array2};
use num_traits::Pow;

use crate::{
    core::{model::spatial::voxels::VoxelType, scenario::Scenario},
    ui::UiState,
};

mod body;
mod heart;
pub mod plotting;
mod sensors;

#[allow(clippy::module_name_repetitions)]
pub struct VisPlugin;

impl Plugin for VisPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PanOrbitCameraPlugin)
            .init_resource::<SampleTracker>()
            .init_resource::<VisOptions>()
            .add_plugins(VertexPullingRenderPlugin { outlines: true })
            .add_systems(Startup, setup)
            .add_systems(
                Update,
                update_sample_index.run_if(in_state(UiState::Volumetric)),
            )
            .add_systems(
                Update,
                update_heart_voxel_colors
                    .run_if(in_state(UiState::Volumetric))
                    .after(update_sample_index),
            );
    }
}

#[derive(Resource, Debug)]
pub struct SampleTracker {
    pub current_sample: usize,
    pub max_sample: usize,
    pub sample_rate: f32,
}

impl Default for SampleTracker {
    fn default() -> Self {
        Self {
            current_sample: 1,
            max_sample: 1,
            sample_rate: 1.0,
        }
    }
}

#[allow(clippy::module_name_repetitions)]
#[derive(Resource, Debug)]
pub struct VisOptions {
    pub playbackspeed: f32,
    pub mode: VisMode,
}

impl Default for VisOptions {
    fn default() -> Self {
        Self {
            playbackspeed: 1.0,
            mode: VisMode::EstimatedCdeNorm,
        }
    }
}

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub enum VisMode {
    EstimatedCdeNorm,
    SimulatedCdeNorm,
}

#[derive(Component)]
struct VoxelData {
    indices: Array1<usize>,
    colors: Array2<u32>,
    positions: Array2<usize>,
}

pub fn setup(mut commands: Commands) {
    setup_light_and_camera(&mut commands);
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
                set_heart_voxel_colors_to_types(&cuboids, &mut voxel_data, scenario);
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

fn init_sample_tracker(sample_tracker: &mut SampleTracker, scenario: &Scenario) {
    sample_tracker.current_sample = 0;
    sample_tracker.max_sample = scenario
        .data
        .as_ref()
        .expect("Data to be some")
        .get_measurements()
        .values
        .shape()[0];
    sample_tracker.sample_rate = scenario
        .config
        .simulation
        .as_ref()
        .expect("Simultaion to be some")
        .sample_rate_hz;
}

pub fn setup_light_and_camera(commands: &mut Commands) {
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });

    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        PanOrbitCamera::default(),
    ));
}

#[allow(clippy::needless_pass_by_value)]
fn set_heart_voxel_colors_to_types(
    cuboids: &Cuboids,
    voxel_data: &mut VoxelData,
    scenario: &Scenario,
) {
    let voxels_types = scenario
        .data
        .as_ref()
        .expect("Data to be some")
        .get_voxel_types();

    for index in 0..cuboids.instances.len() {
        for sample in 0..voxel_data.colors.shape()[1] {
            let x = voxel_data.positions[(index, 0)];
            let y = voxel_data.positions[(index, 1)];
            let z = voxel_data.positions[(index, 2)];
            voxel_data.colors[(index, sample)] = type_to_color(voxels_types.values[(x, y, z)]);
        }
    }
}

fn type_to_color(voxel_type: VoxelType) -> u32 {
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

#[allow(clippy::needless_pass_by_value)]
fn update_heart_voxel_colors(
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

// might want to add a accum delta time to sampletracker, so that I can also change
// the current sample manually.
#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::needless_pass_by_value
)]
fn update_sample_index(
    mut sample_tracker: ResMut<SampleTracker>,
    time: Res<Time>,
    vis_options: Res<VisOptions>,
) {
    sample_tracker.current_sample = ((time.elapsed_seconds()
        * sample_tracker.sample_rate
        * vis_options.playbackspeed) as usize)
        % sample_tracker.max_sample;
}
