mod algorithm;
pub mod common;
mod data;

use bevy::prelude::*;
use bevy_editor_cam::prelude::{EditorCam, EnabledMotion};
use bevy_egui::{egui, EguiContexts};
use egui::Align;
use tracing::error;

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
#[tracing::instrument(skip_all, level = "trace")]
pub fn draw_ui_scenario(
    mut contexts: EguiContexts,
    mut scenarios: ResMut<ScenarioList>,
    mut selected_scenario: ResMut<SelectedSenario>,
    mut cameras: Query<&mut EditorCam, With<Camera>>,
) {
    trace!("Running system to draw scenario UI.");
    let context = match contexts.ctx_mut() {
        Ok(ctx) => ctx,
        Err(e) => {
            error!("EGUI context not available for scenario UI: {}", e);
            return;
        }
    };

    draw_ui_scenario_topbar(
        context,
        &mut scenarios,
        &mut selected_scenario,
        &mut cameras,
    );

    let Some(index) = selected_scenario.index else {
        error!("No scenario selected for scenario UI");
        return;
    };
    let Some(entry) = scenarios.entries.get_mut(index) else {
        error!("Selected scenario index {} is out of bounds", index);
        return;
    };
    let scenario = &mut entry.scenario;
    draw_ui_scenario_central_panel(context, scenario, &mut cameras);
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
    cameras: &mut Query<&mut EditorCam, With<Camera>>,
) {
    trace!("Running system to draw scenario topbar.");
    egui::TopBottomPanel::top("scenario_status").show(context, |ui| {
        for mut camera in cameras {
            if ui.ui_contains_pointer() {
                camera.enabled_motion = EnabledMotion {
                    pan: false,
                    orbit: false,
                    zoom: false,
                };
            }
        }
        ui.with_layout(egui::Layout::left_to_right(Align::TOP), |ui| {
            let Some(index) = selected_scenario.index else {
                error!("No scenario selected for topbar operations");
                return;
            };
            let Some(entry) = scenarios.entries.get_mut(index) else {
                error!(
                    "Selected scenario index {} is out of bounds in topbar",
                    index
                );
                return;
            };
            let scenario = &mut entry.scenario;
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
                        if let Err(e) = scenario.schedule() {
                            error!("Failed to schedule scenario: {}", e);
                        }
                    }
                }
                Status::Scheduled => {
                    if ui.button("Unschedule").clicked() {
                        if let Err(e) = scenario.unschedule() {
                            error!("Failed to unschedule scenario: {}", e);
                        }
                    }
                }
                _ => (),
            }
            if ui.button("Save").clicked() {
                if let Err(e) = scenario.save() {
                    error!("Failed to save scenario: {}", e);
                }
            } else if ui.button("Delete").clicked() {
                if let Err(e) = scenario.delete() {
                    error!("Failed to delete scenario: {}", e);
                } else {
                    scenarios.entries.remove(index);
                    selected_scenario.index = Some(0);
                }
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
            let Some(index) = selected_scenario.index else {
                error!("No scenario selected for comment editing");
                return;
            };
            let Some(entry) = scenarios.entries.get_mut(index) else {
                error!(
                    "Selected scenario index {} is out of bounds for comment editing",
                    index
                );
                return;
            };
            let scenario = &mut entry.scenario;
            if ui
                .add(egui::TextEdit::multiline(&mut scenario.comment).desired_width(f32::INFINITY))
                .lost_focus()
            {
                if let Err(e) = scenario.save() {
                    error!("Failed to save scenario: {}", e);
                }
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
fn draw_ui_scenario_central_panel(
    context: &egui::Context,
    scenario: &mut Scenario,
    cameras: &mut Query<&mut EditorCam, With<Camera>>,
) {
    trace!("Running system to draw scenario central panel");
    egui::CentralPanel::default().show(context, |ui| {
        for mut camera in cameras {
            if ui.ui_contains_pointer() {
                camera.enabled_motion = EnabledMotion {
                    pan: false,
                    orbit: false,
                    zoom: false,
                };
            }
        }
        ui.columns(2, |columns| {
            draw_ui_scenario_data(&mut columns[0], scenario);
            draw_ui_scenario_algoriothm(&mut columns[1], scenario);
        });
    });
}
