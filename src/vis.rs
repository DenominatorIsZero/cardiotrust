pub mod body;
pub mod heart;
pub mod options;
pub mod plotting;
pub mod sample_tracker;
pub mod sensors;

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

pub fn setup_light_and_camera(commands: &mut Commands) {
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 1.0,
    });

    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        PanOrbitCamera::default(),
    ));
}
