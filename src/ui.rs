mod explorer;
mod results;
mod scenario;
mod topbar;
mod vol;

use bevy::prelude::*;
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
    #[tracing::instrument(skip(app))]
    fn build(&self, app: &mut App) {
        app.init_state::<UiState>()
            .init_resource::<ResultImages>()
            .init_resource::<SelectedResultImage>()
            .init_resource::<PlaybackSpeed>()
            .add_plugins(EguiPlugin)
            .add_systems(Update, draw_ui_topbar)
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
#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
#[allow(clippy::module_name_repetitions)]
pub enum UiState {
    #[default]
    Explorer,
    Scenario,
    Results,
    Volumetric,
}

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub struct ClientUiPlugin;

impl Plugin for ClientUiPlugin {
    #[tracing::instrument(skip(app))]
    fn build(&self, app: &mut App) {
        app.init_state::<UiState>()
            .init_resource::<PlaybackSpeed>()
            .add_plugins(EguiPlugin)
            .add_systems(Update, draw_ui_volumetric);
    }
}
