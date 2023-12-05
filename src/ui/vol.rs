use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::{
    vis::{setup_heart_voxels, SampleTracker, VisOptions},
    ScenarioList, SelectedSenario,
};

#[allow(clippy::needless_pass_by_value)]
pub fn draw_ui_volumetric(
    mut contexts: EguiContexts,
    mut commands: Commands,
    mut sample_tracker: ResMut<SampleTracker>,
    _vis_options: ResMut<VisOptions>,
    selected_scenario: Res<SelectedSenario>,
    scenario_list: Res<ScenarioList>,
) {
    egui::SidePanel::left("volumetric_left_panel").show(contexts.ctx_mut(), |ui| {
        ui.label("Volumetric");
        if ui
            .add_enabled(
                selected_scenario.index.is_some(),
                egui::Button::new("Init Voxels"),
            )
            .clicked()
        {
            setup_heart_voxels(
                &mut commands,
                &mut sample_tracker,
                &scenario_list
                    .entries
                    .get(selected_scenario.index.expect("Scenario to be selected."))
                    .as_ref()
                    .expect("Scenario to exist.")
                    .scenario,
            );
        };
    });
}
