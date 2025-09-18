use bevy::prelude::*;
use bevy_editor_cam::prelude::{EditorCam, EnabledMotion};
use bevy_egui::{egui, EguiContexts};
use egui::Separator;
use tracing::error;

use super::UiState;
use crate::{
    core::scenario::Status,
    scheduler::{NumberOfJobs, SchedulerState},
    ScenarioList, SelectedSenario,
};

/// Draws the UI for the top bar, containing buttons to switch between UI states
/// and start/stop the scheduler. Also contains a slider to control the number
/// of scheduler jobs.
#[allow(clippy::module_name_repetitions, clippy::needless_pass_by_value)]
#[tracing::instrument(skip_all, level = "trace")]
pub fn draw_ui_topbar(
    mut commands: Commands,
    mut contexts: EguiContexts,
    ui_state: Res<State<UiState>>,
    scheduler_state: Res<State<SchedulerState>>,
    mut scenario_list: ResMut<ScenarioList>,
    selected_scenario: Res<SelectedSenario>,
    mut number_of_jobs: ResMut<NumberOfJobs>,
    mut cameras: Query<&mut EditorCam, With<Camera>>,
) {
    trace!("Running system to draw topbar.");
    let ctx = match contexts.ctx_mut() {
        Ok(ctx) => ctx,
        Err(e) => {
            error!("EGUI context not available for topbar: {}", e);
            return;
        }
    };
    egui::TopBottomPanel::top("menu_panel").show(ctx, |ui| {
        for mut camera in &mut cameras {
            if ui.ui_contains_pointer() {
                camera.enabled_motion = EnabledMotion {
                    pan: false,
                    orbit: false,
                    zoom: false,
                };
            }
        }
        ui.horizontal(|ui| {
            if ui
                .add_enabled(
                    ui_state.get() != &UiState::Explorer,
                    egui::Button::new("Explorer"),
                )
                .clicked()
            {
                commands.insert_resource(NextState::Pending(UiState::Explorer));
            }
            if ui
                .add_enabled(
                    ui_state.get() != &UiState::Scenario && selected_scenario.index.is_some(),
                    egui::Button::new("Scenario"),
                )
                .clicked()
            {
                commands.insert_resource(NextState::Pending(UiState::Scenario));
            }
            if ui
                .add_enabled(
                    ui_state.get() != &UiState::Results
                        && selected_scenario.index.is_some()
                        && selected_scenario.index
                            .map(|index| scenario_list.entries.get(index)
                                .map(|entry| entry.scenario.get_status() == &Status::Done)
                                .unwrap_or(false))
                            .unwrap_or(false),
                    egui::Button::new("Results"),
                )
                .clicked()
            {
                if let Some(index) = selected_scenario.index {
                    if let Some(entry) = scenario_list.entries.get_mut(index) {
                        let scenario = &mut entry.scenario;
                        if let Err(e) = scenario.load_data() {
                            error!("Failed to load scenario data: {}", e);
                        }
                        if let Err(e) = scenario.load_results() {
                            error!("Failed to load scenario results: {}", e);
                        }
                        commands.insert_resource(NextState::Pending(UiState::Results));
                    } else {
                        error!("Selected scenario index {} is out of bounds", index);
                    }
                } else {
                    error!("No scenario selected for Results view");
                }
            }
            if ui
                .add_enabled(
                    ui_state.get() != &UiState::Volumetric
                        && selected_scenario.index.is_some()
                        && selected_scenario.index
                            .map(|index| scenario_list.entries.get(index)
                                .map(|entry| entry.scenario.get_status() == &Status::Done)
                                .unwrap_or(false))
                            .unwrap_or(false),
                    egui::Button::new("Volumetric"),
                )
                .clicked()
            {
                if let Some(index) = selected_scenario.index {
                    if let Some(entry) = scenario_list.entries.get_mut(index) {
                        let scenario = &mut entry.scenario;
                        if let Err(e) = scenario.load_data() {
                            error!("Failed to load scenario data: {}", e);
                        }
                        if let Err(e) = scenario.load_results() {
                            error!("Failed to load scenario results: {}", e);
                        }
                        commands.insert_resource(NextState::Pending(UiState::Volumetric));
                    } else {
                        error!("Selected scenario index {} is out of bounds", index);
                    }
                } else {
                    error!("No scenario selected for Volumetric view");
                }
            }
            ui.add(Separator::default().spacing(200.0));
            if ui
                .add_enabled(
                    scheduler_state.get() == &SchedulerState::Paused,
                    egui::Button::new("Start"),
                )
                .clicked()
            {
                commands.insert_resource(NextState::Pending(SchedulerState::Available));
            }
            if ui
                .add_enabled(
                    scheduler_state.get() != &SchedulerState::Paused,
                    egui::Button::new("Stop"),
                )
                .clicked()
            {
                commands.insert_resource(NextState::Pending(SchedulerState::Paused));
            }
            ui.label("Number of jobs:");
            ui.add(egui::Slider::new(&mut number_of_jobs.value, 1..=32));
        });
    });
}
