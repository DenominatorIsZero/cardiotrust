use egui_extras::{Column, TableBuilder};
use tracing::trace;

use super::{
    common::draw_ui_scenario_common, FIRST_COLUMN_WIDTH, PADDING, ROW_HEIGHT, SECOND_COLUMN_WIDTH,
};
use crate::core::{
    algorithm::refinement::Optimizer,
    config::algorithm::{Algorithm, AlgorithmType},
    scenario::{Scenario, Status},
};

/// Draws the UI elements for the algorithm.
#[allow(clippy::too_many_lines)]
#[tracing::instrument(skip(parent), level = "trace")]
pub fn draw_ui_scenario_algoriothm(parent: &mut egui::Ui, scenario: &mut Scenario) {
    trace!("Running system to draw scenario algorithm UI.");
    if *scenario.get_status() != Status::Planning {
        parent.disable();
    }
    let algorithm = &mut scenario.config.algorithm;
    egui::ScrollArea::vertical()
        .id_source("algorithm")
        .show(parent, |ui| {
            ui.heading("Algorithm");
            ui.separator();
            draw_algorithm_settings(ui, algorithm);
            if algorithm.algorithm_type == AlgorithmType::ModelBased {
                draw_optimizer_settings(ui, algorithm);
                draw_estimation_settings(ui, algorithm);
                draw_regularization_settings(ui, algorithm);
                draw_metrics_settings(ui, algorithm);
                draw_ui_scenario_common(ui, &mut algorithm.model);
            }
        });
}

#[allow(clippy::too_many_lines)]
#[tracing::instrument(skip_all, level = "trace")]
fn draw_estimation_settings(ui: &mut egui::Ui, algorithm: &mut Algorithm) {
    ui.label(egui::RichText::new("Estimation Settings").underline());
    ui.group(|ui| {
        let width = ui.available_width();
        TableBuilder::new(ui)
            .column(Column::exact(FIRST_COLUMN_WIDTH))
            .column(Column::exact(SECOND_COLUMN_WIDTH))
            .column(Column::exact(
                width - FIRST_COLUMN_WIDTH - SECOND_COLUMN_WIDTH - PADDING,
            ))
            .striped(true)
            .header(ROW_HEIGHT, |mut header| {
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
                // Apply system update
                body.row(ROW_HEIGHT, |mut row| {
                    row.col(|ui| {
                        ui.label("Apply\nsystem update");
                    });
                    row.col(|ui| {
                        ui.checkbox(&mut algorithm.model.common.apply_system_update, "");
                    });
                    row.col(|ui| {
                        ui.add(
                            egui::Label::new(
                                "Wether or not to apply the system\
                                update step during the state estimation phase\
                                of the algorithm.",
                            )
                            .truncate(),
                        );
                    });
                });
                if algorithm.model.common.apply_system_update {
                    // Update Kalman Gain
                    body.row(ROW_HEIGHT, |mut row| {
                        row.col(|ui| {
                            ui.label("Update\nKalman Gain");
                        });
                        row.col(|ui| {
                            ui.checkbox(&mut algorithm.update_kalman_gain, "");
                        });
                        row.col(|ui| {
                            ui.add(
                                egui::Label::new(
                                    "Wether or not to update\
                                    the Kalman gain. If set to false a\
                                    simplified gain will be calculated once\
                                    at initialization.",
                                )
                                .truncate(),
                            );
                        });
                    });
                    // Process covariance mean
                    body.row(ROW_HEIGHT, |mut row| {
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
                            ui.add(
                                egui::Label::new(
                                    "The mean value of the process\
                                 noise covariance matrix.",
                                )
                                .truncate(),
                            );
                        });
                    });
                    // Process covariance std
                    body.row(ROW_HEIGHT, |mut row| {
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
                            ui.add(
                                egui::Label::new(
                                    "The standard deviation of the\
                                process noise covariance matrix.\
                                If this is zero, all diagonal values will\
                                be choosen as the mean.\
                                Otherwise they will be drawn from a normal\
                                distribution according\
                                to the mean value and standard deviation.",
                                )
                                .truncate(),
                            );
                        });
                    });
                }
            });
    });
}

#[tracing::instrument(skip_all, level = "trace")]
fn draw_regularization_settings(ui: &mut egui::Ui, algorithm: &mut Algorithm) {
    ui.label(egui::RichText::new("Regulariztion Settings").underline());
    ui.group(|ui| {
        let width = ui.available_width();
        TableBuilder::new(ui)
            .column(Column::exact(FIRST_COLUMN_WIDTH))
            .column(Column::exact(SECOND_COLUMN_WIDTH))
            .column(Column::exact(
                width - FIRST_COLUMN_WIDTH - SECOND_COLUMN_WIDTH - PADDING,
            ))
            .striped(true)
            .header(ROW_HEIGHT, |mut header| {
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
                if algorithm.algorithm_type == AlgorithmType::ModelBased {
                    // Regularization threshold
                    body.row(ROW_HEIGHT, |mut row| {
                        row.col(|ui| {
                            ui.label("Regularization\nthreshold");
                        });
                        row.col(|ui| {
                            ui.add(egui::Slider::new(
                                &mut algorithm.maximum_regularization_threshold,
                                0.5..=1.5,
                            ));
                        });
                        row.col(|ui| {
                            ui.add(
                                egui::Label::new(
                                    "The absolute value of\
                                    current density that has to be\
                                    exceeded before the regularization\
                                    starts havin an effect. Default: 1.1.",
                                )
                                .truncate(),
                            );
                        });
                    });
                    // Regularization strength
                    body.row(ROW_HEIGHT, |mut row| {
                        row.col(|ui| {
                            ui.label("Regularization\nstrength");
                        });
                        row.col(|ui| {
                            ui.add(egui::Slider::new(
                                &mut algorithm.maximum_regularization_strength,
                                0.0..=1000.0,
                            ));
                        });
                        row.col(|ui| {
                            ui.add(
                                egui::Label::new("The weighting of the regularization term.")
                                    .truncate(),
                            );
                        });
                    });
                }
            });
    });
}

#[allow(clippy::too_many_lines)]
#[tracing::instrument(skip_all, level = "trace")]
fn draw_algorithm_settings(ui: &mut egui::Ui, algorithm: &mut Algorithm) {
    ui.label(egui::RichText::new("Algorithm Settings").underline());
    ui.group(|ui| {
        let width = ui.available_width();
        TableBuilder::new(ui)
            .column(Column::exact(FIRST_COLUMN_WIDTH))
            .column(Column::exact(SECOND_COLUMN_WIDTH))
            .column(Column::exact(
                width - FIRST_COLUMN_WIDTH - SECOND_COLUMN_WIDTH - PADDING,
            ))
            .striped(true)
            .header(ROW_HEIGHT, |mut header| {
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
                body.row(ROW_HEIGHT, |mut row| {
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
                        ui.add(
                            egui::Label::new(
                                "The algorhim used for estimating the \
                                     current densities.",
                            )
                            .truncate(),
                        );
                    });
                });
                if algorithm_type == &AlgorithmType::ModelBased {
                    // Epochs
                    body.row(ROW_HEIGHT, |mut row| {
                        row.col(|ui| {
                            ui.label("Epochs");
                        });
                        row.col(|ui| {
                            ui.add(egui::Slider::new(&mut algorithm.epochs, 1..=50000));
                        });
                        row.col(|ui| {
                            ui.add(
                                egui::Label::new("The number of epochs to run the algorithm for.")
                                    .truncate(),
                            );
                        });
                    });
                    // Batch size
                    body.row(ROW_HEIGHT, |mut row| {
                        row.col(|ui| {
                            ui.label("Batch size");
                        });
                        row.col(|ui| {
                            ui.add(egui::Slider::new(&mut algorithm.batch_size, 0..=50000));
                        });
                        row.col(|ui| {
                            ui.add(
                                egui::Label::new(
                                    "The batch size to use for the algorithm.\
                                After how many samples to update the weights.\
                                Default: 0 - one update per Epoch.",
                                )
                                .truncate(),
                            );
                        });
                    });
                    // Freeze gains
                    body.row(ROW_HEIGHT, |mut row| {
                        row.col(|ui| {
                            ui.label("Freeze gains");
                        });
                        row.col(|ui| {
                            ui.checkbox(&mut algorithm.freeze_gains, "");
                        });
                        row.col(|ui| {
                            ui.add(
                                egui::Label::new(
                                    "Wether or not to freeze the gains\
                                    preventing them from being changed.",
                                )
                                .truncate(),
                            );
                        });
                    });
                    // Freeze delays
                    body.row(ROW_HEIGHT, |mut row| {
                        row.col(|ui| {
                            ui.label("Freeze delays");
                        });
                        row.col(|ui| {
                            ui.checkbox(&mut algorithm.freeze_delays, "");
                        });
                        row.col(|ui| {
                            ui.add(
                                egui::Label::new(
                                    "Wether or not to freeze the delays\
                                    preventing them from being changed.",
                                )
                                .truncate(),
                            );
                        });
                    });
                }
            });
    });
}

#[tracing::instrument(skip_all, level = "trace")]
fn draw_metrics_settings(ui: &mut egui::Ui, algorithm: &mut Algorithm) {
    ui.label(egui::RichText::new("Metrics Settings").underline());
    ui.group(|ui| {
        let width = ui.available_width();
        TableBuilder::new(ui)
            .column(Column::exact(FIRST_COLUMN_WIDTH))
            .column(Column::exact(SECOND_COLUMN_WIDTH))
            .column(Column::exact(
                width - FIRST_COLUMN_WIDTH - SECOND_COLUMN_WIDTH - PADDING,
            ))
            .striped(true)
            .header(ROW_HEIGHT, |mut header| {
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
                if algorithm.algorithm_type == AlgorithmType::ModelBased {
                    // Snapshot interval
                    body.row(ROW_HEIGHT, |mut row| {
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
                            ui.add(
                                egui::Label::new(
                                    "How often to take snapshots during the\
                                optimization of the model.\
                                Default: 0 - no snapshots are taken, only the final\
                                result is stored.",
                                )
                                .truncate(),
                            );
                        });
                    });
                }
            });
    });
}

#[allow(clippy::too_many_lines)]
#[tracing::instrument(skip_all, level = "trace")]

fn draw_optimizer_settings(ui: &mut egui::Ui, algorithm: &mut Algorithm) {
    ui.label(egui::RichText::new("Optimizer Settings").underline());
    ui.group(|ui| {
        let width = ui.available_width();
        TableBuilder::new(ui)
            .column(Column::exact(FIRST_COLUMN_WIDTH))
            .column(Column::exact(SECOND_COLUMN_WIDTH))
            .column(Column::exact(
                width - FIRST_COLUMN_WIDTH - SECOND_COLUMN_WIDTH - PADDING,
            ))
            .striped(true)
            .header(ROW_HEIGHT, |mut header| {
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
                if algorithm.algorithm_type == AlgorithmType::ModelBased {
                    // Optimizer
                    body.row(ROW_HEIGHT, |mut row| {
                        row.col(|ui| {
                            ui.label("Optimizer");
                        });
                        row.col(|ui| {
                let optimzer = &mut algorithm.optimizer;
                        egui::ComboBox::new("cb_optimizer", "")
                            .selected_text(format!("{optimzer:?}"))
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    optimzer,
                                    Optimizer::Sgd,
                                    "SGD",
                                );
                                ui.selectable_value(
                                    optimzer,
                                    Optimizer::Adam,
                                    "ADAM",
                                );
                            });
                        });
                        row.col(|ui| {
                            ui.add(
                                egui::Label::new(
                                    "The optimization algorithm used to update the model parameters."
                                )
                                .truncate(),
                            );
                        });
                    });
                    // Learning rate
                    body.row(ROW_HEIGHT, |mut row| {
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
                            ui.add(
                                egui::Label::new(
                                    "The learning rate used in the model refinement\
                                step of the algorithm.",
                                )
                                .truncate(),
                            );
                        });
                    });
                    // Learning rate reduction interval
                    body.row(ROW_HEIGHT, |mut row| {
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
                            ui.add(
                                egui::Label::new(
                                    "The interval between which to reduce the learning rate.\
                                    a value of 0 means no learning rate reduction is done.",
                                )
                                .truncate(),
                            );
                        });
                    });
                    if algorithm.learning_rate_reduction_interval > 0 {
                        // Learning rate reduction factor
                        body.row(ROW_HEIGHT, |mut row| {
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
                                ui.add(
                                    egui::Label::new(
                                        "The factor with which to multiply the learning rate\
                                    every n epochs.",
                                    )
                                    .truncate(),
                                );
                            });
                        });
                    }
                }
            });
    });
}
