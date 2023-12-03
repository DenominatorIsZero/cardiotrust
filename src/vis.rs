use std::default;

use bevy::prelude::*;
use bevy_aabb_instancing::{Cuboid, CuboidMaterialId, Cuboids, VertexPullingRenderPlugin};
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use ndarray::{Array1, Array2};
use num_traits::Pow;

use crate::{core::scenario::Scenario, ui::UiState};

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
}

pub fn setup(mut commands: Commands) {
    setup_light_and_camera(&mut commands);
}

// this should be run on button press instead using the selected scenario
#[allow(clippy::cast_precision_loss)]
pub fn setup_heart_voxels(
    commands: &mut Commands,
    sample_tracker: &mut SampleTracker,
    scenario: &Scenario,
    vis_mode: &VisMode,
) {
    const PATCHES_PER_DIM: usize = 20;
    const PATCH_SIZE: usize = 15;
    const SCENE_RADIUS: f32 = 150.0;

    sample_tracker.current_sample = 0;
    sample_tracker.max_sample = 2000;
    sample_tracker.sample_rate = 2000.0;

    for x_batch in 0..PATCHES_PER_DIM {
        for z_batch in 0..PATCHES_PER_DIM {
            let mut instances = Vec::with_capacity(PATCH_SIZE * PATCH_SIZE);
            for x_index in 0..PATCH_SIZE {
                for z_index in 0..PATCH_SIZE {
                    let x_position = (x_batch * PATCH_SIZE) as f32 + x_index as f32 - SCENE_RADIUS;
                    let z_position = (z_batch * PATCH_SIZE) as f32 + z_index as f32 - SCENE_RADIUS;
                    let distance_from_origin = x_position.hypot(z_position);
                    let amplitude = 0.2 * distance_from_origin;
                    let y_position =
                        amplitude * ((0.05 * x_position).cos() * (0.05 * z_position).sin());
                    let position = Vec3::new(x_position, y_position, z_position);
                    let height = 0.01 * distance_from_origin;
                    let min = position - Vec3::new(0.5, height, 0.5);
                    let max = position + Vec3::new(0.5, height, 0.5);
                    let scalar_color = Color::as_rgba_u32(Color::Rgba {
                        red: min.x / SCENE_RADIUS,
                        green: min.z / SCENE_RADIUS,
                        blue: 0.0,
                        alpha: 1.0,
                    });
                    let mut cuboid = Cuboid::new(min, max, scalar_color);
                    cuboid.set_depth_bias(0);
                    instances.push(cuboid);
                }
            }
            let cuboids = Cuboids::new(instances);
            let aabb = cuboids.aabb();
            let indices = Array1::zeros(PATCH_SIZE.pow(2));
            let mut colors = Array2::zeros((PATCH_SIZE.pow(2), sample_tracker.max_sample));
            let color_mult = (x_batch * PATCHES_PER_DIM + z_batch) as f32
                / (PATCHES_PER_DIM * PATCHES_PER_DIM) as f32;
            colors.fill(Color::as_rgba_u32(Color::Rgba {
                red: color_mult,
                green: color_mult,
                blue: color_mult,
                alpha: 1.0,
            }));
            let voxel_data = VoxelData { indices, colors };
            commands.spawn(SpatialBundle::default()).insert((
                cuboids,
                aabb,
                CuboidMaterialId(0),
                voxel_data,
            ));
        }
    }
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
