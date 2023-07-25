mod explorer;
mod results;
mod scenario;
mod topbar;
mod vol;

use bevy::prelude::*;
use bevy_egui::EguiPlugin;

use self::{
    explorer::draw_ui_explorer,
    results::{draw_ui_results, load_example_image, ResultImages, SelectedResultImage},
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
            .add_plugin(EguiPlugin)
            .add_system(draw_ui_topbar)
            .add_system(draw_ui_explorer.run_if(in_state(UiState::Explorer)))
            .add_system(draw_ui_scenario.run_if(in_state(UiState::Scenario)))
            .add_system(draw_ui_results.run_if(in_state(UiState::Results)))
            .add_startup_system(load_example_image)
            .add_system(draw_ui_volumetric.run_if(in_state(UiState::Volumetric)));
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
