use std::mem::discriminant;

use bevy::prelude::*;
use bevy_editor_cam::prelude::{EditorCam, EnabledMotion};
use bevy_egui::{egui, EguiContexts};
use egui::ProgressBar;
use egui_extras::{Column, TableBuilder};

use super::UiState;
use crate::{
    core::scenario::{Scenario, Status},
    ScenarioBundle, ScenarioList, SelectedSenario,
};

/// Draws the UI for the scenario explorer.
///
/// This displays a table with columns for scenario ID, status, losses, metrics,
/// and allows creating new scenarios and selecting one to view/edit details.
///
/// Uses egui to create the table and columns. Loops through the scenarios
/// from the `ScenarioList` resource to populate the rows. Inserts a new row
/// when the New button is clicked.
#[allow(clippy::module_name_repetitions, clippy::too_many_lines)]
#[tracing::instrument(skip_all, level = "trace")]
pub fn draw_ui_explorer(
    mut commands: Commands,
    mut contexts: EguiContexts,
    mut scenario_list: ResMut<ScenarioList>,
    mut selected_scenario: ResMut<SelectedSenario>,
    mut cameras: Query<&mut EditorCam, With<Camera>>,
) {
    trace!("Drawing UI for explorer tab");
    egui::CentralPanel::default().show(contexts.ctx_mut().expect("EGUI context available"), |ui| {
            for mut camera in &mut cameras {
                if ui.ui_contains_pointer() {
                    camera.enabled_motion = EnabledMotion {
                        pan: false,
                        orbit: false,
                        zoom: false,
                    };
                }
            }
            TableBuilder::new(ui)
                .column(Column::auto().resizable(true))
                .column(Column::initial(150.0).resizable(true))
                .column(Column::initial(100.0).resizable(true))
                .column(Column::initial(75.0).resizable(true))
                .column(Column::initial(75.0).resizable(true))
                .column(Column::initial(75.0).resizable(true))
                .column(Column::initial(75.0).resizable(true))
                .column(Column::initial(75.0).resizable(true))
                .column(Column::initial(75.0).resizable(true))
                .column(Column::initial(75.0).resizable(true))
                .column(Column::remainder())
                .header(20.0, |mut header| {
                    header.col(|ui| {
                        ui.heading("\nID\n");
                    });
                    header.col(|ui| {
                        ui.heading("\nStatus\n");
                    });
                    header.col(|ui| {
                        ui.heading("\nLoss\n");
                    });
                    header.col(|ui| {
                        ui.heading("\nMSE Loss\n");
                    });
                    header.col(|ui| {
                        ui.heading("\nM. R. Loss\n");
                    });
                    header.col(|ui| {
                        ui.heading("\nThreshold");
                    });
                    header.col(|ui| {
                        ui.heading("\nDice");
                    });
                    header.col(|ui| {
                        ui.heading("\nIoU");
                    });
                    header.col(|ui| {
                        ui.heading("\nRecall");
                    });
                    header.col(|ui| {
                        ui.heading("\nPrecision");
                    });
                    header.col(|ui| {
                        ui.heading("\nComment");
                    });
                })
                .body(|mut body| {
                    for index in 0..scenario_list.entries.len() {
                        draw_row(
                            &mut commands,
                            &mut body,
                            index,
                            &mut scenario_list,
                            &mut selected_scenario,
                        );
                    }
                    body.row(30.0, |mut row| {
                        row.col(|ui| {
                            if ui.button("New").clicked() {
                                scenario_list.entries.push(ScenarioBundle {
                                    scenario: Scenario::build(None),
                                    join_handle: None,
                                    epoch_rx: None,
                                    summary_rx: None,
                                });
                                selected_scenario.index = Some(scenario_list.entries.len() - 1);
                                commands.insert_resource(NextState::Pending(UiState::Scenario));
                            }
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
                    });
                });
        },
    );
}

/// Draws a row in the scenario list table.
///
/// For the scenario at the given index, this renders UI elements to show the
/// scenario's status, metrics, comment text box, etc. It is called in a loop
/// to draw each row.
#[allow(clippy::too_many_lines)]
#[tracing::instrument(skip(commands, body), level = "trace")]
fn draw_row(
    commands: &mut Commands,
    body: &mut egui_extras::TableBody,
    index: usize,
    scenario_list: &mut ResMut<ScenarioList>,
    selected_scenario: &mut ResMut<SelectedSenario>,
) {
    trace!("Drawing row in scenario list table");
    body.row(30.0, |mut row| {
        row.col(|ui| {
            if ui
                .button(scenario_list.entries[index].scenario.get_id())
                .clicked()
            {
                selected_scenario.index = Some(index);
                commands.insert_resource(NextState::Pending(UiState::Scenario));
            }
        });
        row.col(|ui| {
            if discriminant(scenario_list.entries[index].scenario.get_status())
                == discriminant(&Status::Running(1))
            {
                ui.add(
                    ProgressBar::new(scenario_list.entries[index].scenario.get_progress())
                        .show_percentage()
                        .text(scenario_list.entries[index].scenario.get_etc()),
                );
            } else {
                ui.label(scenario_list.entries[index].scenario.get_status_str());
            }
        });
        row.col(|ui| {
            match &scenario_list.entries[index].scenario.summary {
                Some(summary) => ui.label(format!("{:.3e}", summary.loss)),
                None => ui.label("-"),
            };
        });
        row.col(|ui| {
            match &scenario_list.entries[index].scenario.summary {
                Some(summary) => ui.label(format!("{:.3e}", summary.loss_mse)),
                None => ui.label("-"),
            };
        });
        row.col(|ui| {
            match &scenario_list.entries[index].scenario.summary {
                Some(summary) => ui.label(format!("{:.3e}", summary.loss_maximum_regularization)),
                None => ui.label("-"),
            };
        });
        row.col(|ui| {
            match &scenario_list.entries[index].scenario.summary {
                Some(summary) => ui.label(format!("{:.3e}", summary.threshold)),
                None => ui.label("-"),
            };
        });
        row.col(|ui| {
            match &scenario_list.entries[index].scenario.summary {
                Some(summary) => ui.label(format!("{:.3e}", summary.dice)),
                None => ui.label("-"),
            };
        });
        row.col(|ui| {
            match &scenario_list.entries[index].scenario.summary {
                Some(summary) => ui.label(format!("{:.3e}", summary.iou)),
                None => ui.label("-"),
            };
        });
        row.col(|ui| {
            match &scenario_list.entries[index].scenario.summary {
                Some(summary) => ui.label(format!("{:.3e}", summary.recall)),
                None => ui.label("-"),
            };
        });
        row.col(|ui| {
            match &scenario_list.entries[index].scenario.summary {
                Some(summary) => ui.label(format!("{:.3e}", summary.precision)),
                None => ui.label("-"),
            };
        });
        row.col(|ui| {
            if ui
                .add(
                    egui::TextEdit::multiline(&mut scenario_list.entries[index].scenario.comment)
                        .desired_width(f32::INFINITY)
                        .desired_rows(2),
                )
                .lost_focus()
            {
                scenario_list.entries[index].scenario.save().unwrap();
            }
        });
    });
}
