use egui_extras::{Column, TableBuilder};
use tracing::trace;

use super::common::draw_ui_scenario_common;
use crate::core::{
    config::algorithm::AlgorithmType,
    scenario::{Scenario, Status},
};

/// Draws the UI elements for the algorithm.
#[allow(clippy::too_many_lines)]
#[tracing::instrument(skip(parent), level = "trace")]
pub fn draw_ui_scenario_algoriothm(parent: &mut egui::Ui, scenario: &mut Scenario) {
    trace!("Running system to draw scenario algorithm UI.");
    parent.set_enabled(*scenario.get_status() == Status::Planning);
    let algorithm = &mut scenario.config.algorithm;
    egui::ScrollArea::vertical()
        .id_source("algorithm")
        .show(parent, |ui| {
            ui.heading("Algorithm");
            ui.separator();
            ui.push_id("algorithm_parameter_table", |ui| {
                TableBuilder::new(ui)
                    .column(Column::initial(125.0).resizable(true))
                    .column(Column::auto().resizable(true))
                    .column(Column::initial(200.0).resizable(true))
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
                        // algorithm type
                        let algorithm_type = &mut algorithm.algorithm_type;
                        body.row(30.0, |mut row| {
                            row.col(|ui| {
                                ui.label("Algorithm Type");
                            });
                            row.col(|ui| {
                                egui::ComboBox::new("cb_algorithm_type", "")
                                    .selected_text(format!("{algorithm_type:?}"))
                                    .show_ui(ui, |ui| {
                                        ui.selectable_value(
                                            algorithm_type,
                                            AlgorithmType::ModelBased,
                                            "Model Based",
                                        );
                                        ui.selectable_value(
                                            algorithm_type,
                                            AlgorithmType::PseudoInverse,
                                            "Pseudo Inverse",
                                        );
                                    });
                            });
                            row.col(|ui| {
                                ui.label(
                                    "The control function used as the input tthe system \
                                    / The shape of the assumed current density curve.",
                                );
                            });
                        });
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
                        // Batch size
                        body.row(30.0, |mut row| {
                            row.col(|ui| {
                                ui.label("Batch Size");
                            });
                            row.col(|ui| {
                                ui.add(egui::Slider::new(&mut algorithm.batch_size, 0..=50000));
                            });
                            row.col(|ui| {
                                ui.label(
                                    "The batch size to use for the algorithm.\
                                After how many samples to update the weights.\
                                Default: 0 - one update per Epoch.",
                                );
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
                        // Gradient Clamping Threshold
                        body.row(30.0, |mut row| {
                            row.col(|ui| {
                                ui.label("Gradient clamping\nthreshold");
                            });
                            row.col(|ui| {
                                ui.add(
                                    egui::Slider::new(
                                        &mut algorithm.gradient_clamping_threshold,
                                        1e-6..=1e3,
                                    )
                                    .logarithmic(true)
                                    .custom_formatter(|n, _| format!("{n:+.4e}")),
                                );
                            });
                            row.col(|ui| {
                                ui.label(
                                    "The maximum value to which the gradients/
                                    are clamped",
                                );
                            });
                        });
                        // Learning rate reduction factor
                        body.row(30.0, |mut row| {
                            row.col(|ui| {
                                ui.label("Learning rate reduction factor");
                            });
                            row.col(|ui| {
                                ui.add(
                                    egui::Slider::new(
                                        &mut algorithm.learning_rate_reduction_factor,
                                        1e-10..=1e10,
                                    )
                                    .logarithmic(true)
                                    .custom_formatter(|n, _| format!("{n:+.4e}")),
                                );
                            });
                            row.col(|ui| {
                                ui.label(
                                    "The factor with which to multiply the learning rate\
                                    every n epochs.",
                                );
                            });
                        });
                        // Learning rate reduction interval
                        body.row(30.0, |mut row| {
                            row.col(|ui| {
                                ui.label("Learning rate reduction interval");
                            });
                            row.col(|ui| {
                                ui.add(
                                    egui::Slider::new(
                                        &mut algorithm.learning_rate_reduction_interval,
                                        0..=50000,
                                    )
                                    .logarithmic(true)
                                    .custom_formatter(|n, _| format!("{n:+.4e}")),
                                );
                            });
                            row.col(|ui| {
                                ui.label(
                                    "The interval between which to reduce the learning rate.\
                                    a value of 0 means no learning rate reduction is done.",
                                );
                            });
                        });
                        // Regularization Threshold
                        body.row(30.0, |mut row| {
                            row.col(|ui| {
                                ui.label("Regularization\nthreshold");
                            });
                            row.col(|ui| {
                                ui.add(egui::Slider::new(
                                    &mut algorithm.regularization_threshold,
                                    0.5..=1.5,
                                ));
                            });
                            row.col(|ui| {
                                ui.label(
                                    "The absolute value of\
                                    current density that has to be\
                                    exceeded before the regularization\
                                    starts havin an effect. Default: 1.1.",
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
                                    0.0..=1000.0,
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
                                        &mut algorithm.model.common.process_covariance_mean,
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
                                    &mut algorithm.model.common.process_covariance_std,
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
                                ui.checkbox(&mut algorithm.model.common.apply_system_update, "");
                            });
                            row.col(|ui| {
                                ui.label(
                                    "Wether or not to apply the system\
                                update step during the state estimation phase\
                                of the algorithm.",
                                );
                            });
                        });
                        // Constrain Current Density
                        body.row(30.0, |mut row| {
                            row.col(|ui| {
                                ui.label("Constrain\ncurrent density");
                            });
                            row.col(|ui| {
                                ui.checkbox(&mut algorithm.constrain_system_states, "");
                            });
                            row.col(|ui| {
                                ui.label(
                                    "Wether or not to constrain the current\
                                    density to a maximum value. Doing so\
                                    prevents fast divergence of the system.",
                                );
                            });
                        });
                        // State Clamping Threshold
                        body.row(30.0, |mut row| {
                            row.col(|ui| {
                                ui.label("State Clamping\nthreshold");
                            });
                            row.col(|ui| {
                                ui.add(egui::Slider::new(
                                    &mut algorithm.state_clamping_threshold,
                                    1.0..=10.0,
                                ));
                            });
                            row.col(|ui| {
                                ui.label(
                                    "The absolute value of\
                                    current density that has to be\
                                    exceeded before the constrain\
                                    starts havin an effect. Default: 1.5.",
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
