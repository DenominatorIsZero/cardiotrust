use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use egui_plot::{Line, Plot, PlotPoints};

use crate::{
    vis::{
        heart::setup_heart_voxels,
        options::{VisMode, VisOptions},
        sample_tracker::SampleTracker,
    },
    ScenarioList, SelectedSenario,
};

#[allow(clippy::needless_pass_by_value)]
pub fn draw_ui_volumetric(
    mut contexts: EguiContexts,
    mut commands: Commands,
    mut sample_tracker: ResMut<SampleTracker>,
    mut vis_options: ResMut<VisOptions>,
    selected_scenario: Res<SelectedSenario>,
    scenario_list: Res<ScenarioList>,
) {
    egui::SidePanel::left("volumetric_left_panel").show(contexts.ctx_mut(), |ui| {
        ui.label("Volumetric");
        if ui.button("Init Voxels").clicked() {
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
        let mut vis_mode = vis_options.mode.clone();
        egui::ComboBox::new("cb_vis_mode", "")
            .selected_text(format!("{vis_mode:?}"))
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    &mut vis_mode,
                    VisMode::EstimationVoxelTypes,
                    "Voxel types (estimation)",
                );
                ui.selectable_value(
                    &mut vis_mode,
                    VisMode::SimulationVoxelTypes,
                    "Voxel types (simulation)",
                );
                ui.selectable_value(
                    &mut vis_mode,
                    VisMode::EstimatedCdeNorm,
                    "Cde norm (estimation)",
                );
                ui.selectable_value(
                    &mut vis_mode,
                    VisMode::SimulatedCdeNorm,
                    "Cde norm (simulation)",
                );
                ui.selectable_value(
                    &mut vis_mode,
                    VisMode::EstimatedCdeMax,
                    "Cde max (estimation)",
                );
                ui.selectable_value(
                    &mut vis_mode,
                    VisMode::SimulatedCdeMax,
                    "Cde max (simulation)",
                );
            });
        if vis_mode != vis_options.mode {
            vis_options.mode = vis_mode;
        }
    });
    egui::TopBottomPanel::bottom("Volumetric bottom panel")
        .exact_height(400.0)
        .show(contexts.ctx_mut(), |ui| {
            let sin: PlotPoints = (0..1000)
                .map(|i| {
                    let x = i as f64 * 0.01;
                    [x, x.sin()]
                })
                .collect();
            let line = Line::new(sin);
            Plot::new("my_plot")..show(ui, |plot_ui| plot_ui.line(line));
        });
}
