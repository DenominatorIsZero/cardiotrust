use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use egui_extras::{Column, TableBuilder};

use crate::core::scenario::Scenario;
use crate::{ScenarioBundle, ScenarioList, SelectedSenario};

use super::UiState;

pub fn draw_ui_explorer(
    mut commands: Commands,
    mut contexts: EguiContexts,
    mut scenario_list: ResMut<ScenarioList>,
    mut selected_scenario: ResMut<SelectedSenario>,
) {
    egui::CentralPanel::default().show(contexts.ctx_mut(), |ui| {
        TableBuilder::new(ui)
            .column(Column::auto().resizable(true))
            .column(Column::initial(150.0).resizable(true))
            .column(Column::remainder())
            .header(20.0, |mut header| {
                header.col(|ui| {
                    ui.heading("");
                });
                header.col(|ui| {
                    ui.heading("ID");
                });
                header.col(|ui| {
                    ui.heading("Status");
                });
            })
            .body(|mut body| {
                for (index, entry) in scenario_list.entries.iter().enumerate() {
                    draw_row(
                        &mut commands,
                        &mut body,
                        index,
                        &entry.scenario,
                        &mut selected_scenario,
                    );
                }
                body.row(30.0, |mut row| {
                    row.col(|_ui| {});
                    row.col(|ui| {
                        if ui.button("New").clicked() {
                            scenario_list.entries.push(ScenarioBundle {
                                scenario: Scenario::build(None),
                                join_handle: None,
                            });
                            selected_scenario.index = Some(scenario_list.entries.len() - 1);
                            commands.insert_resource(NextState(Some(UiState::Scenario)));
                        };
                    });
                    row.col(|_ui| {});
                });
            });
    });
}

fn draw_row(
    commands: &mut Commands,
    body: &mut egui_extras::TableBody,
    index: usize,
    scenario: &Scenario,
    selected_scenario: &mut ResMut<SelectedSenario>,
) {
    body.row(30.0, |mut row| {
        row.col(|_ui| {
            // Checkbox goes here later
        });
        row.col(|ui| {
            if ui.button(scenario.get_id()).clicked() {
                commands.insert_resource(NextState(Some(UiState::Scenario)));
                selected_scenario.index = Some(index);
            };
        });
        row.col(|ui| {
            ui.label(scenario.get_status_str());
        });
    });
}
