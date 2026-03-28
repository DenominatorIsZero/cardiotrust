pub mod bevy_shell;
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
    bevy_shell::BevyShellPlugin,
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
            .init_resource::<SidebarState>()
            .add_plugins(EguiPlugin::default())
            .add_plugins(BevyShellPlugin)
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

/// An enum representing the different UI states (views) of the application.
///
/// The default state is `Explorer`. Under the Bevy backend all six views are
/// reachable; under `EGui` only `Explorer`, `Scenario`, `Results`, and
/// `Volumetric` are used.
///
/// This allows conditional rendering of different UI components
/// depending on the current state.
#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash)]
#[allow(clippy::module_name_repetitions)]
pub enum UiState {
    Home,
    Explorer,
    Scenario,
    Results,
    Volumetric,
    Scheduler,
}

impl Default for UiState {
    #[tracing::instrument(level = "trace")]
    fn default() -> Self {
        Self::Home
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
        Self::Bevy
    }
}

/// Tracks the sidebar expanded/collapsed state and current width.
///
/// When expanded the sidebar is 200 px wide (icon + label). When collapsed it
/// is 56 px wide (icon only). The `user_expanded` field remembers the user's
/// preference so auto-collapse can restore it when the viewport widens.
#[derive(Resource, Debug)]
pub struct SidebarState {
    pub expanded: bool,
    pub width: f32,
    /// The user's explicit preference (before auto-collapse overrides it).
    pub user_expanded: bool,
}

impl Default for SidebarState {
    #[tracing::instrument(level = "trace")]
    fn default() -> Self {
        Self {
            expanded: true,
            width: 200.0,
            user_expanded: true,
        }
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
