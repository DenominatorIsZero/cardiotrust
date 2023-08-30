use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use egui::Separator;

use crate::{
    core::scenario::Status,
    scheduler::{NumberOfJobs, SchedulerState},
    ScenarioList, SelectedSenario,
};

use super::UiState;

#[allow(clippy::module_name_repetitions, clippy::needless_pass_by_value)]
pub fn draw_ui_topbar(
    mut commands: Commands,
    mut contexts: EguiContexts,
    ui_state: Res<State<UiState>>,
    scheduler_state: Res<State<SchedulerState>>,
    mut scenario_list: ResMut<ScenarioList>,
    selected_scenario: Res<SelectedSenario>,
    mut number_of_jobs: ResMut<NumberOfJobs>,
) {
    egui::TopBottomPanel::top("menu_panel").show(contexts.ctx_mut(), |ui| {
        ui.horizontal(|ui| {
            if ui
                .add_enabled(
                    ui_state.get() != &UiState::Explorer,
                    egui::Button::new("Explorer"),
                )
                .clicked()
            {
                println!("Opening Explorer UI");
                commands.insert_resource(NextState(Some(UiState::Explorer)));
            };
            if ui
                .add_enabled(
                    ui_state.get() != &UiState::Scenario,
                    egui::Button::new("Scenario"),
                )
                .clicked()
            {
                println!("Opening Scenario UI");
                commands.insert_resource(NextState(Some(UiState::Scenario)));
            };
            if ui
                .add_enabled(
                    ui_state.get() != &UiState::Results
                        && selected_scenario.index.is_some()
                        && scenario_list.entries[selected_scenario.index.unwrap()]
                            .scenario
                            .get_status()
                            == &Status::Done,
                    egui::Button::new("Results"),
                )
                .clicked()
            {
                let index = selected_scenario.index.unwrap();
                let scenario = &mut scenario_list.entries[index].scenario;
                scenario.load_data();
                scenario.load_results();
                println!("Opening Results UI");
                commands.insert_resource(NextState(Some(UiState::Results)));
            };
            if ui
                .add_enabled(
                    ui_state.get() != &UiState::Volumetric,
                    egui::Button::new("Volumetric"),
                )
                .clicked()
            {
                println!("Opening Volumetric UI");
                commands.insert_resource(NextState(Some(UiState::Volumetric)));
            };
            ui.add(Separator::default().spacing(200.0));
            if ui
                .add_enabled(
                    scheduler_state.get() == &SchedulerState::Paused,
                    egui::Button::new("Start"),
                )
                .clicked()
            {
                println!("Starting Scheduler");
                commands.insert_resource(NextState(Some(SchedulerState::Available)));
            };
            if ui
                .add_enabled(
                    scheduler_state.get() != &SchedulerState::Paused,
                    egui::Button::new("Stop"),
                )
                .clicked()
            {
                println!("Stopping Scheduler");
                commands.insert_resource(NextState(Some(SchedulerState::Paused)));
            };
            ui.label("Number of jobs:");
            ui.add(egui::Slider::new(&mut number_of_jobs.value, 1..=32));
        });
    });
}
