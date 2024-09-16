mod algorithm;
pub mod common;
mod data;

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use egui::Align;

use self::{algorithm::draw_ui_scenario_algoriothm, data::draw_ui_scenario_data};
use crate::{
    core::{
        config::model::{
            Handcrafted, Mri, DEFAULT_HEART_OFFSET_HANDCRAFTED, DEFAULT_HEART_OFFSET_MRI,
        },
        scenario::{Scenario, Status},
    },
    ScenarioBundle, ScenarioList, SelectedSenario,
};

const FIRST_COLUMN_WIDTH: f32 = 150.0;
const SECOND_COLUMN_WIDTH: f32 = 200.0;
const PADDING: f32 = 20.0;
const ROW_HEIGHT: f32 = 30.0;

/// Draws the UI for the selected scenario.
///
/// This handles:
/// - The top bar with scenario list and controls
/// - The central panel showing details of the selected scenario
#[allow(clippy::module_name_repetitions)]
#[tracing::instrument(skip(contexts), level = "trace")]
pub fn draw_ui_scenario(
    mut contexts: EguiContexts,
    mut scenarios: ResMut<ScenarioList>,
    mut selected_scenario: ResMut<SelectedSenario>,
) {
    trace!("Running system to draw scenario UI.");
    let context = contexts.ctx_mut();

    draw_ui_scenario_topbar(context, &mut scenarios, &mut selected_scenario);

    let index = selected_scenario.index.unwrap();
    let scenario = &mut scenarios.entries[index].scenario;
    draw_ui_scenario_central_panel(context, scenario);
}

/// Draws the top bar UI for the scenario view.
///
/// This shows:
/// - The ID and status of the selected scenario
/// - Controls to change the status and save the scenario
/// - A text area to edit the scenario description
/// - Buttons to copy, delete or select a different scenario
#[tracing::instrument(skip(context), level = "trace")]
fn draw_ui_scenario_topbar(
    context: &egui::Context,
    scenarios: &mut ResMut<ScenarioList>,
    selected_scenario: &mut ResMut<SelectedSenario>,
) {
    trace!("Running system to draw scenario topbar.");
    egui::TopBottomPanel::top("scenario_status").show(context, |ui| {
        ui.with_layout(egui::Layout::left_to_right(Align::TOP), |ui| {
            let index = selected_scenario.index.unwrap();
            let scenario = &mut scenarios.entries[index].scenario;
            ui.label(format!("Scenario with ID: {}", scenario.get_id()));
            ui.separator();
            ui.label(format!("Status: {}", scenario.get_status_str()));
            ui.separator();
            ui.vertical(|ui| {
                let mut handcrafted = scenario.config.algorithm.model.handcrafted.is_some();
                let simulation = &mut scenario.config.simulation;
                let last_value = handcrafted;
                let model_type = if handcrafted { "Handcrafted" } else { "MRI" };
                ui.label("Model Type:");
                egui::ComboBox::new("cb_model_type", "")
                    .selected_text(model_type)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut handcrafted, true, "Handcrafted");
                        ui.selectable_value(&mut handcrafted, false, "MRI");
                    });
                if last_value != handcrafted {
                    if handcrafted {
                        scenario.config.algorithm.model.handcrafted = Some(Handcrafted::default());
                        scenario.config.algorithm.model.mri = None;
                        simulation.model.handcrafted = Some(Handcrafted::default());
                        simulation.model.mri = None;
                        simulation.model.common.heart_offset_mm = DEFAULT_HEART_OFFSET_HANDCRAFTED;
                    } else {
                        scenario.config.algorithm.model.handcrafted = None;
                        scenario.config.algorithm.model.mri = Some(Mri::default());
                        simulation.model.handcrafted = None;
                        simulation.model.mri = Some(Mri::default());
                        simulation.model.common.heart_offset_mm = DEFAULT_HEART_OFFSET_MRI;
                    }
                }
            });
            ui.separator();
            match scenario.get_status() {
                Status::Planning => {
                    if ui.button("Schedule").clicked() {
                        scenario.schedule().unwrap();
                    }
                }
                Status::Scheduled => {
                    if ui.button("Unschedule").clicked() {
                        scenario.unschedule().unwrap();
                    }
                }
                _ => (),
            }
            if ui.button("Save").clicked() {
                scenario.save().unwrap();
            } else if ui.button("Delete").clicked() {
                scenario.delete().unwrap();
                scenarios.entries.remove(index);
                selected_scenario.index = Some(0);
            } else if ui.button("Copy").clicked() {
                let mut new_scenario = Scenario::build(None);
                new_scenario.config = scenario.config.clone();
                new_scenario.comment.clone_from(&scenario.comment);
                scenarios.entries.push(ScenarioBundle {
                    scenario: new_scenario,
                    join_handle: None,
                    epoch_rx: None,
                    summary_rx: None,
                });
                selected_scenario.index = Some(scenarios.entries.len() - 1);
            }
            ui.separator();
            let index = selected_scenario.index.unwrap();
            let scenario = &mut scenarios.entries[index].scenario;
            if ui
                .add(egui::TextEdit::multiline(&mut scenario.comment).desired_width(f32::INFINITY))
                .lost_focus()
            {
                scenario.save().unwrap();
            }
        });
    });
}

/// Draws the UI for the central panel of the scenario screen.
///
/// Splits the panel into two columns using egui columns.
/// The left column calls `draw_ui_scenario_data` to show scenario data.
/// The right column calls `draw_ui_scenario_algorithm` to show algorithm settings.
#[tracing::instrument(skip(context), level = "trace")]
fn draw_ui_scenario_central_panel(context: &egui::Context, scenario: &mut Scenario) {
    trace!("Running system to draw scenario central panel");
    egui::CentralPanel::default().show(context, |ui| {
        ui.columns(2, |columns| {
            draw_ui_scenario_data(&mut columns[0], scenario);
            draw_ui_scenario_algoriothm(&mut columns[1], scenario);
        });
    });
}
