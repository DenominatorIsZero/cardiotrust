mod handcrafted;
mod mri;
mod velocity;

use egui_extras::{Column, TableBuilder};
use tracing::trace;

use super::{FIRST_COLUMN_WIDTH, PADDING, ROW_HEIGHT, SECOND_COLUMN_WIDTH};
use crate::core::config::model::{ControlFunction, Model};

/// Draws ui for settings common to data generation and optimization.
#[allow(clippy::too_many_lines, clippy::module_name_repetitions)]
#[tracing::instrument(skip(ui), level = "trace")]
pub fn draw_ui_scenario_common(ui: &mut egui::Ui, model: &mut Model) {
    trace!("Running system to draw scenario common UI.");
    draw_measurement_settings(ui, model);
    draw_functional_settings(ui, model);
    velocity::draw_velocity_settings(ui, model);
    if let Some(handcrafted) = model.handcrafted.as_mut() {
        handcrafted::draw_handcrafted_settings(ui, handcrafted, model.common.pathological);
    }
    if let Some(mri) = model.mri.as_mut() {
        mri::draw_mri_settings(ui, mri, model.common.pathological);
    }
}

#[tracing::instrument(skip_all, level = "trace")]
fn draw_measurement_settings(ui: &mut egui::Ui, model: &mut Model) {
    ui.label(egui::RichText::new("Measurement Settings").underline());
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
                // Measurment covariance mean
                body.row(ROW_HEIGHT, |mut row| {
                    row.col(|ui| {
                        ui.label("Measurement\ncovariance mean");
                    });
                    row.col(|ui| {
                        ui.add(
                            egui::Slider::new(
                                &mut model.common.measurement_covariance_mean,
                                1e-10..=1e10,
                            )
                            .logarithmic(true)
                            .custom_formatter(|n, _| format!("{n:+.4e}")),
                        );
                    });
                    row.col(|ui| {
                        ui.add(
                            egui::Label::new(
                                "The mean value of the measurement noise covariance matrix.",
                            )
                            .truncate(),
                        );
                    });
                });
                // Mearurment covariance standard deviation
                body.row(ROW_HEIGHT, |mut row| {
                    row.col(|ui| {
                        ui.label("Measurement\ncovariance std");
                    });
                    row.col(|ui| {
                        ui.add(egui::Slider::new(
                            &mut model.common.measurement_covariance_std,
                            0.0..=1.0,
                        ));
                    });
                    row.col(|ui| {
                        ui.add(
                            egui::Label::new(
                                "The standard deviation of the \
                                measurement noise covariance matrix. \
                                If this is zero, all diagonal values will \
                                be choosen as the mean. \
                                Otherwise they will be drawn from a normal \
                                distribution according \
                                to the mean value and standard deviation.",
                            )
                            .truncate(),
                        );
                    });
                });
            });
    });
}

#[tracing::instrument(skip_all, level = "trace")]
fn draw_functional_settings(ui: &mut egui::Ui, model: &mut Model) {
    ui.label(egui::RichText::new("Functional Settings").underline());
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
                // Control function
                let control_function = &mut model.common.control_function;
                body.row(ROW_HEIGHT, |mut row| {
                    row.col(|ui| {
                        ui.label("Control function");
                    });
                    row.col(|ui| {
                        egui::ComboBox::new("cb_control_function", "")
                            .selected_text(format!("{control_function:?}"))
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    control_function,
                                    ControlFunction::Triangle,
                                    "Triangle",
                                );
                                ui.selectable_value(
                                    control_function,
                                    ControlFunction::Ohara,
                                    "Ohara",
                                );
                            });
                    });
                    row.col(|ui| {
                        ui.add(
                            egui::Label::new(
                                "The control function used as the input tthe system \
                                    / The shape of the assumed current density curve.",
                            )
                            .truncate(),
                        );
                    });
                });
                // Pathological
                body.row(ROW_HEIGHT, |mut row| {
                    row.col(|ui| {
                        ui.label("Pathological");
                    });
                    row.col(|ui| {
                        ui.checkbox(&mut model.common.pathological, "");
                    });
                    row.col(|ui| {
                        ui.add(
                            egui::Label::new(
                                "Whether or not to place pathological tissue in the model.",
                            )
                            .truncate(),
                        );
                    });
                });
                // Current Factor in Pathology
                if model.common.pathological {
                    body.row(ROW_HEIGHT, |mut row| {
                        row.col(|ui| {
                            ui.label("Current Factor \nin pathology");
                        });
                        row.col(|ui| {
                            ui.add(egui::Slider::new(
                                &mut model.common.current_factor_in_pathology,
                                0.0..=1.0,
                            ));
                        });
                        row.col(|ui| {
                            ui.add(
                                egui::Label::new(
                                    "A factor describing how much to reduce the \
                                    current densities in pathological voxels.",
                                )
                                .truncate(),
                            );
                        });
                    });
                }
            });
    });
}
