use bevy::prelude::*;
use bevy_editor_cam::prelude::{EditorCam, EnabledMotion};
use bevy_egui::{egui, EguiContexts};
use egui::Separator;

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
    egui::TopBottomPanel::top("menu_panel").show(contexts.ctx_mut().expect("EGUI context available"), |ui| {
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
                        && scenario_list.entries
                            [selected_scenario.index.expect("Index to be some.")]
                        .scenario
                        .get_status()
                            == &Status::Done,
                    egui::Button::new("Results"),
                )
                .clicked()
            {
                let index = selected_scenario.index.expect("Index to be some.");
                let scenario = &mut scenario_list.entries[index].scenario;
                scenario.load_data();
                scenario.load_results();
                commands.insert_resource(NextState::Pending(UiState::Results));
            }
            if ui
                .add_enabled(
                    ui_state.get() != &UiState::Volumetric
                        && selected_scenario.index.is_some()
                        && scenario_list.entries
                            [selected_scenario.index.expect("Index to be some.")]
                        .scenario
                        .get_status()
                            == &Status::Done,
                    egui::Button::new("Volumetric"),
                )
                .clicked()
            {
                let index = selected_scenario.index.expect("Index to be some.");
                let scenario = &mut scenario_list.entries[index].scenario;
                scenario.load_data();
                scenario.load_results();
                commands.insert_resource(NextState::Pending(UiState::Volumetric));
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
