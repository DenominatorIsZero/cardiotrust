mod explorer;
mod results;
mod scenario;
mod topbar;
mod vol;

use bevy::prelude::*;
use bevy_editor_cam::prelude::{EditorCam, EnabledMotion};
use bevy_egui::EguiPlugin;

use self::{
    explorer::draw_ui_explorer,
    results::{
        draw_ui_results, reset_result_images, PlaybackSpeed, ResultImages, SelectedResultImage,
    },
    scenario::draw_ui_scenario,
    topbar::draw_ui_topbar,
    vol::draw_ui_volumetric,
};

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub struct UiPlugin;

impl Plugin for UiPlugin {
    #[tracing::instrument(level = "info", skip(app))]
    fn build(&self, app: &mut App) {
        info!("Initializing UI plugin.");
        app.init_state::<UiState>()
            .init_resource::<ResultImages>()
            .init_resource::<SelectedResultImage>()
            .init_resource::<PlaybackSpeed>()
            .add_plugins(EguiPlugin)
            .add_systems(Update, enable_camera_motion)
            .add_systems(Update, draw_ui_topbar.after(enable_camera_motion))
            .add_systems(
                Update,
                draw_ui_explorer
                    .run_if(in_state(UiState::Explorer))
                    .after(draw_ui_topbar),
            )
            .add_systems(
                Update,
                draw_ui_scenario
                    .run_if(in_state(UiState::Scenario))
                    .after(draw_ui_topbar),
            )
            .add_systems(
                Update,
                draw_ui_results
                    .run_if(in_state(UiState::Results))
                    .after(draw_ui_topbar),
            )
            .add_systems(
                Update,
                draw_ui_volumetric
                    .run_if(in_state(UiState::Volumetric))
                    .after(draw_ui_topbar),
            )
            .add_systems(Update, reset_result_images);
    }
}

/// An enum representing the different UI states of the application.
///
/// The default state is `Explorer`. The other states are `Scenario`,
/// `Results`, and `Volumetric`.
///
/// This allows conditional rendering of different UI components
/// depending on the current state.
#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash)]
#[allow(clippy::module_name_repetitions)]
pub enum UiState {
    Explorer,
    Scenario,
    Results,
    Volumetric,
}

impl Default for UiState {
    #[cfg(target_arch = "wasm32")]
    fn default() -> Self {
        Self::Volumetric
    }
    #[cfg(not(target_arch = "wasm32"))]
    fn default() -> Self {
        Self::Explorer
    }
}

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub struct ClientUiPlugin;

impl Plugin for ClientUiPlugin {
    #[tracing::instrument(level = "info", skip(app))]
    fn build(&self, app: &mut App) {
        info!("Initializing client UI plugin.");
        app.init_state::<UiState>()
            .init_resource::<PlaybackSpeed>()
            .add_plugins(EguiPlugin)
            .add_systems(Update, draw_ui_volumetric);
    }
}

#[tracing::instrument(skip_all, level = "trace")]
pub fn enable_camera_motion(mut cameras: Query<&mut EditorCam, With<Camera>>) {
    for mut camera in &mut cameras {
        camera.enabled_motion = EnabledMotion {
            pan: true,
            orbit: true,
            zoom: true,
        };
    }
}
