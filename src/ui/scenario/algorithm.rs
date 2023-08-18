use egui_extras::{Column, TableBuilder};

use crate::core::scenario::{Scenario, Status};

use super::common::draw_ui_scenario_common;

#[allow(clippy::too_many_lines)]
pub fn draw_ui_scenario_algoriothm(parent: &mut egui::Ui, scenario: &mut Scenario) {
    parent.set_enabled(*scenario.get_status() == Status::Planning);
    let algorithm = &mut scenario.get_config_mut().algorithm;
    egui::ScrollArea::vertical()
        .id_source("algorithm")
        .show(parent, |ui| {
            ui.heading("Algorithm");
            ui.separator();
            ui.push_id("algorithm_parameter_table", |ui| {
                TableBuilder::new(ui)
                    .column(Column::initial(125.0).resizable(true))
                    .column(Column::auto().resizable(true))
                    .column(Column::initial(600.0).resizable(true))
                    .header(20.0, |mut header| {
                        header.col(|ui| {
                            ui.heading("Parameter");
                        });
                        header.col(|ui| {
                            ui.heading("Value");
                        });
                        header.col(|ui| {
                            ui.heading("Description");
                        });
                    })
                    .body(|mut body| {
                        // Epochs
                        body.row(30.0, |mut row| {
                            row.col(|ui| {
                                ui.label("Epochs");
                            });
                            row.col(|ui| {
                                ui.add(egui::Slider::new(&mut algorithm.epochs, 1..=50000));
                            });
                            row.col(|ui| {
                                ui.label("The number of epochs to run the algorithm for.");
                            });
                        });
                        // Snapshot interval
                        body.row(60.0, |mut row| {
                            row.col(|ui| {
                                ui.label("Snapshot interval");
                            });
                            row.col(|ui| {
                                ui.add(
                                    egui::Slider::new(&mut algorithm.snapshots_interval, 0..=10000)
                                        .suffix(" Epochs"),
                                );
                            });
                            row.col(|ui| {
                                ui.label(
                                    "How often to take snapshots during the\
                                optimization of the model.\
                                Default: 0 - no snapshots are taken, only the final\
                                result is stored.",
                                );
                            });
                        });
                        // Learning rate
                        body.row(30.0, |mut row| {
                            row.col(|ui| {
                                ui.label("Learning rate");
                            });
                            row.col(|ui| {
                                ui.add(
                                    egui::Slider::new(&mut algorithm.learning_rate, 1e-10..=1e10)
                                        .logarithmic(true)
                                        .custom_formatter(|n, _| format!("{n:+.4e}")),
                                );
                            });
                            row.col(|ui| {
                                ui.label(
                                    "The learning rate used in the model refinement\
                                step of the algorithm.",
                                );
                            });
                        });
                        // Regularization Strength
                        body.row(30.0, |mut row| {
                            row.col(|ui| {
                                ui.label("Regularization\nstrength");
                            });
                            row.col(|ui| {
                                ui.add(egui::Slider::new(
                                    &mut algorithm.regularization_strength,
                                    0.0..=1.0,
                                ));
                            });
                            row.col(|ui| {
                                ui.label(
                                    "The weighting of the regularization term.\
                                    The rest of the mse loss get's multiplied by one\
                                    minus this term.",
                                );
                            });
                        });
                        // Process covariance mean
                        body.row(30.0, |mut row| {
                            row.col(|ui| {
                                ui.label("Process\ncovariance mean");
                            });
                            row.col(|ui| {
                                ui.add(
                                    egui::Slider::new(
                                        &mut algorithm.model.process_covariance_mean,
                                        1e-10..=1e10,
                                    )
                                    .logarithmic(true)
                                    .custom_formatter(|n, _| format!("{n:+.4e}")),
                                );
                            });
                            row.col(|ui| {
                                ui.label(
                                    "The mean value of the process\
                                 noise covariance matrix.",
                                );
                            });
                        });
                        // Process covariance std
                        body.row(80.0, |mut row| {
                            row.col(|ui| {
                                ui.label("Process\ncovariance std");
                            });
                            row.col(|ui| {
                                ui.add(egui::Slider::new(
                                    &mut algorithm.model.process_covariance_std,
                                    0.0..=1.0,
                                ));
                            });
                            row.col(|ui| {
                                ui.label(
                                    "The standard deviation of the\
                                process noise covariance matrix.\
                                If this is zero, all diagonal values will\
                                be choosen as the mean.\
                                Otherwise they will be drawn from a normal\
                                distribution according\
                                to the mean value and standard deviation.",
                                );
                            });
                        });
                        // Apply system update
                        body.row(30.0, |mut row| {
                            row.col(|ui| {
                                ui.label("Apply\nsystem update");
                            });
                            row.col(|ui| {
                                ui.checkbox(&mut algorithm.model.apply_system_update, "");
                            });
                            row.col(|ui| {
                                ui.label(
                                    "Wether or not to apply the system\
                                update step during the state estimation phase\
                                of the algorithm.",
                                );
                            });
                        });
                        // Apply system update
                        body.row(30.0, |mut row| {
                            row.col(|ui| {
                                ui.label("Constrain\ncurrent density");
                            });
                            row.col(|ui| {
                                ui.checkbox(&mut algorithm.model.apply_system_update, "");
                            });
                            row.col(|ui| {
                                ui.label(
                                    "Wether or not to constrain the current\
                                    density to a maximum value. Doing so\
                                    prevents fast divergence of the system.",
                                );
                            });
                        });
                        // Freeze Gains
                        body.row(30.0, |mut row| {
                            row.col(|ui| {
                                ui.label("Freeze\nGains");
                            });
                            row.col(|ui| {
                                ui.checkbox(&mut algorithm.freeze_gains, "");
                            });
                            row.col(|ui| {
                                ui.label(
                                    "Wether or not to freeze the gains\
                                    preventing them from being changed.",
                                );
                            });
                        });
                        // Freeze Delays
                        body.row(30.0, |mut row| {
                            row.col(|ui| {
                                ui.label("Freeze\nDelays");
                            });
                            row.col(|ui| {
                                ui.checkbox(&mut algorithm.freeze_delays, "");
                            });
                            row.col(|ui| {
                                ui.label(
                                    "Wether or not to freeze the delays\
                                    preventing them from being changed",
                                );
                            });
                        });
                        // calculate Kalman Gain
                        body.row(30.0, |mut row| {
                            row.col(|ui| {
                                ui.label("Update\nKalman Gain");
                            });
                            row.col(|ui| {
                                ui.checkbox(&mut algorithm.calculate_kalman_gain, "");
                            });
                            row.col(|ui| {
                                ui.label(
                                    "Wether or not to update\
                                    the Kalman gain. If set to false a\
                                    simplified gain will be calculated once\
                                    at initialization.",
                                );
                            });
                        });
                        draw_ui_scenario_common(&mut body, &mut algorithm.model);
                    });
            });
        });
}
