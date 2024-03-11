pub mod body;
pub mod heart;
pub mod options;
pub mod plotting;
pub mod sample_tracker;
pub mod sensors;

use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};

use self::{
    heart::{on_vis_mode_changed, update_heart_voxel_colors},
    options::VisOptions,
    sample_tracker::{update_sample_index, SampleTracker},
};
use crate::ui::UiState;

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

#[allow(clippy::needless_pass_by_value)]
fn spawn_torso(
    mut commands: Commands,
    ass: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // note that we have to include the `Scene0` label
    let my_mesh: Handle<Mesh> = ass.load("torso.glb#Mesh0/Primitive0");

    // to position our 3d model, simply use the Transform
    // in the SceneBundlex
    commands.spawn(PbrBundle {
        mesh: my_mesh,
        // Notice how there is no need to set the `alpha_mode` explicitly here.
        // When converting a color to a material using `into()`, the alpha mode is
        // automatically set to `Blend` if the alpha channel is anything lower than 1.0.
        material: materials.add(Color::rgba(85.0 / 255.0, 79.0 / 255.0, 72.0 / 255.0, 0.25).into()),
        transform: Transform::from_xyz(0.0, 0.0, 0.0)
            .with_scale(Vec3::ONE * 1000.0)
            .with_rotation(Quat::from_euler(EulerRot::XYZ, PI / 2.0, PI, 0.0)),
        ..default()
    });
}

pub fn setup_light_and_camera(commands: &mut Commands) {
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 1.0,
    });

    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(-100.0, 200.0, 50.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        PanOrbitCamera::default(),
    ));
}
