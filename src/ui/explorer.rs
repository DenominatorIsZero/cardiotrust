use std::mem::discriminant;

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use egui::ProgressBar;
use egui_extras::{Column, TableBuilder};

use crate::core::scenario::{Scenario, Status};
use crate::{ScenarioBundle, ScenarioList, SelectedSenario};

use super::UiState;

#[allow(clippy::module_name_repetitions)]
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
            .column(Column::initial(100.0).resizable(true))
            .column(Column::auto().resizable(true))
            .column(Column::auto().resizable(true))
            .column(Column::auto().resizable(true))
            .column(Column::auto().resizable(true))
            .column(Column::auto().resizable(true))
            .column(Column::auto().resizable(true))
            .column(Column::auto().resizable(true))
            .column(Column::auto().resizable(true))
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
                header.col(|ui| {
                    ui.heading("Loss");
                });
                header.col(|ui| {
                    ui.heading("Delta\nStates\nMean");
                });
                header.col(|ui| {
                    ui.heading("Delta\nStates\nMax");
                });
                header.col(|ui| {
                    ui.heading("Delta\nMeas.\nMean");
                });
                header.col(|ui| {
                    ui.heading("Delta\nMeas.\nMax");
                });
                header.col(|ui| {
                    ui.heading("Delta\nGains\nMean");
                });
                header.col(|ui| {
                    ui.heading("Delta\nGains\nMax");
                });
                header.col(|ui| {
                    ui.heading("Delta\nDelays\nMean");
                });
                header.col(|ui| {
                    ui.heading("Delta\nDelays\nMax");
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
                                epoch_rx: None,
                                summary_rx: None,
                            });
                            selected_scenario.index = Some(scenario_list.entries.len() - 1);
                            commands.insert_resource(NextState(Some(UiState::Scenario)));
                        };
                    });
                    row.col(|_ui| {});
                    row.col(|_ui| {});
                    row.col(|_ui| {});
                    row.col(|_ui| {});
                    row.col(|_ui| {});
                    row.col(|_ui| {});
                    row.col(|_ui| {});
                    row.col(|_ui| {});
                    row.col(|_ui| {});
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
            if discriminant(scenario.get_status()) == discriminant(&Status::Running(1)) {
                ui.add(ProgressBar::new(scenario.get_progress()));
            } else {
                ui.label(scenario.get_status_str());
            }
        });
        row.col(|ui| {
            match &scenario.summary {
                Some(summary) => ui.label(summary.loss.to_string()),
                None => ui.label("-"),
            };
        });
        row.col(|ui| {
            match &scenario.summary {
                Some(summary) => ui.label(summary.delta_states_mean.to_string()),
                None => ui.label("-"),
            };
        });
        row.col(|ui| {
            match &scenario.summary {
                Some(summary) => ui.label(summary.delta_states_max.to_string()),
                None => ui.label("-"),
            };
        });
        row.col(|ui| {
            match &scenario.summary {
                Some(summary) => ui.label(summary.delta_measurements_mean.to_string()),
                None => ui.label("-"),
            };
        });
        row.col(|ui| {
            match &scenario.summary {
                Some(summary) => ui.label(summary.delta_measurements_max.to_string()),
                None => ui.label("-"),
            };
        });
        row.col(|ui| {
            match &scenario.summary {
                Some(summary) => ui.label(summary.delta_gains_mean.to_string()),
                None => ui.label("-"),
            };
        });
        row.col(|ui| {
            match &scenario.summary {
                Some(summary) => ui.label(summary.delta_gains_max.to_string()),
                None => ui.label("-"),
            };
        });
        row.col(|ui| {
            match &scenario.summary {
                Some(summary) => ui.label(summary.delta_delays_mean.to_string()),
                None => ui.label("-"),
            };
        });
        row.col(|ui| {
            match &scenario.summary {
                Some(summary) => ui.label(summary.delta_delays_max.to_string()),
                None => ui.label("-"),
            };
        });
    });
}
