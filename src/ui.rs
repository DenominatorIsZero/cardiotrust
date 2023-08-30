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
pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<UiState>()
            .init_resource::<ResultImages>()
            .init_resource::<SelectedResultImage>()
            .init_resource::<PlaybackSpeed>()
            .add_plugins(EguiPlugin)
            .add_systems(Update, draw_ui_topbar)
            .add_systems(Update, draw_ui_explorer.run_if(in_state(UiState::Explorer)))
            .add_systems(Update, draw_ui_scenario.run_if(in_state(UiState::Scenario)))
            .add_systems(Update, draw_ui_results.run_if(in_state(UiState::Results)))
            .add_systems(
                Update,
                draw_ui_volumetric.run_if(in_state(UiState::Volumetric)),
            )
            .add_systems(Update, reset_result_images);
    }
}

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
#[allow(clippy::module_name_repetitions)]
pub enum UiState {
    #[default]
    Explorer,
    Scenario,
    Results,
    Volumetric,
}
