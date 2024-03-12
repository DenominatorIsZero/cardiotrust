pub mod body;
pub mod heart;
pub mod options;
pub mod plotting;
pub mod sample_tracker;
pub mod sensors;

use bevy::prelude::*;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};

use self::{
    body::spawn_torso,
    heart::{init_voxels, on_vis_mode_changed, update_heart_voxel_colors},
    options::VisOptions,
    sample_tracker::{init_sample_tracker, update_sample_index, SampleTracker},
    sensors::spawn_sensors,
};
use crate::{core::scenario::Scenario, ui::UiState};

#[allow(clippy::module_name_repetitions)]
pub struct VisPlugin;

impl Plugin for VisPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PanOrbitCameraPlugin)
            .init_resource::<SampleTracker>()
            .init_resource::<VisOptions>()
            .add_systems(Startup, setup)
            .add_systems(Startup, spawn_torso)
            .add_systems(
                Update,
                update_sample_index.run_if(in_state(UiState::Volumetric)),
            )
            .add_systems(
                Update,
                update_heart_voxel_colors
                    .run_if(in_state(UiState::Volumetric))
                    .after(update_sample_index),
            )
            .add_systems(
                Update,
                on_vis_mode_changed.run_if(in_state(UiState::Volumetric)),
            );
    }
}

pub fn setup(mut commands: Commands) {
    setup_light_and_camera(&mut commands);
}

/// Creates an ambient light to illuminate the full scene.
/// Spawns a camera entity with default pan/orbit controls.
pub fn setup_light_and_camera(commands: &mut Commands) {
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 1000.0,
    });

    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(-100.0, 200.0, 50.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        PanOrbitCamera::default(),
    ));
}

/// Sets up the heart mesh, voxel grid, and sensor transforms according
/// to the provided scenario. Initializes the sample tracker based on the
/// scenario as well.
#[allow(clippy::cast_precision_loss)]
pub fn setup_heart_and_sensors(
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
