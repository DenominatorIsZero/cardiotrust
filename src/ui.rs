pub mod colors;
mod explorer;
mod results;
mod scenario;
mod topbar;
mod vol;

use bevy::prelude::*;
use bevy_editor_cam::prelude::{EditorCam, EnabledMotion};
use bevy_egui::{EguiPlugin, EguiPrimaryContextPass};

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
            .init_state::<UiType>()
            .init_resource::<ResultImages>()
            .init_resource::<SelectedResultImage>()
            .init_resource::<PlaybackSpeed>()
            .add_plugins(EguiPlugin::default())
            .add_systems(Update, enable_camera_motion)
            .add_systems(Update, toggle_ui_type_on_f2)
            .add_systems(
                EguiPrimaryContextPass,
                draw_ui_topbar.run_if(in_state(UiType::EGui)),
            )
            .add_systems(
                EguiPrimaryContextPass,
                draw_ui_explorer
                    .run_if(in_state(UiState::Explorer).and(in_state(UiType::EGui)))
                    .after(draw_ui_topbar),
            )
            .add_systems(
                EguiPrimaryContextPass,
                draw_ui_scenario
                    .run_if(in_state(UiState::Scenario).and(in_state(UiType::EGui)))
                    .after(draw_ui_topbar),
            )
            .add_systems(
                EguiPrimaryContextPass,
                draw_ui_results
                    .run_if(in_state(UiState::Results).and(in_state(UiType::EGui)))
                    .after(draw_ui_topbar),
            )
            .add_systems(
                EguiPrimaryContextPass,
                draw_ui_volumetric
                    .run_if(in_state(UiState::Volumetric).and(in_state(UiType::EGui)))
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
    #[tracing::instrument(level = "trace")]
    fn default() -> Self {
        Self::Explorer
    }
}

/// Selects which UI backend is active.
///
/// `EGui` (default) runs the existing EGUI-based UI systems. `Bevy` will run
/// the Bevy-native UI systems once they are implemented. Toggle at runtime
/// with the **F2** key.
#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash)]
#[allow(clippy::module_name_repetitions)]
pub enum UiType {
    EGui,
    Bevy,
}

impl Default for UiType {
    #[tracing::instrument(level = "trace")]
    fn default() -> Self {
        Self::EGui
    }
}

/// Toggles [`UiType`] between `EGui` and `Bevy` each time **F2** is pressed.
#[tracing::instrument(skip_all, level = "trace")]
pub fn toggle_ui_type_on_f2(
    keys: Res<ButtonInput<KeyCode>>,
    ui_type: Res<State<UiType>>,
    mut next: ResMut<NextState<UiType>>,
) {
    if keys.just_pressed(KeyCode::F2) {
        next.set(match ui_type.get() {
            UiType::EGui => UiType::Bevy,
            UiType::Bevy => UiType::EGui,
        });
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
