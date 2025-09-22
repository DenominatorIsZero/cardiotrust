use std::path::PathBuf;

use egui_extras::{Column, TableBuilder};
use tracing::{error, trace};

use super::{FIRST_COLUMN_WIDTH, PADDING, ROW_HEIGHT, SECOND_COLUMN_WIDTH};
use crate::core::config::model::{ControlFunction, Handcrafted, Model, Mri};

/// Draws ui for settings common to data generation and optimization.
#[allow(clippy::too_many_lines, clippy::module_name_repetitions)]
#[tracing::instrument(skip(ui), level = "trace")]
pub fn draw_ui_scenario_common(ui: &mut egui::Ui, model: &mut Model) {
    trace!("Running system to draw scenario common UI.");
    draw_measurement_settings(ui, model);
    draw_functional_settings(ui, model);
    draw_velocity_settings(ui, model);
    if let Some(handcrafted) = model.handcrafted.as_mut() {
        draw_handcrafted_settings(ui, handcrafted, model.common.pathological);
    }
    if let Some(mri) = model.mri.as_mut() {
        draw_mri_settings(ui, mri, model.common.pathological);
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

#[allow(clippy::too_many_lines)]
#[tracing::instrument(skip_all, level = "trace")]
fn draw_velocity_settings(ui: &mut egui::Ui, model: &mut Model) {
    ui.label(egui::RichText::new("Velocity Settings").underline());
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
                // SA
                body.row(ROW_HEIGHT, |mut row| {
                    row.col(|ui| {
                        ui.label("Sinoatrial Node");
                    });
                    row.col(|ui| {
                        ui.add(
                            egui::Slider::new(
                                &mut model.common.propagation_velocities.sinoatrial,
                                0.01..=10.0,
                            )
                            .suffix(" m/s"),
                        );
                    });
                    row.col(|ui| {
                        ui.add(
                            egui::Label::new(
                                "Desired propagation velocity in the \
                                    sinoatrial node in m/s. Note that the \
                                    maximum propagation velocity is limited \
                                    by the voxel size and sample rate.",
                            )
                            .truncate(),
                        );
                    });
                });
                // Atrium
                body.row(ROW_HEIGHT, |mut row| {
                    row.col(|ui| {
                        ui.label("Atrium");
                    });
                    row.col(|ui| {
                        ui.add(
                            egui::Slider::new(
                                &mut model.common.propagation_velocities.atrium,
                                0.01..=10.0,
                            )
                            .suffix(" m/s"),
                        );
                    });
                    row.col(|ui| {
                        ui.add(
                            egui::Label::new(
                                "Desired propagation velocity in the \
                                    atrium in m/s. Note that the \
                                    maximum propagation velocity is limited \
                                    by the voxel size and sample rate.",
                            )
                            .truncate(),
                        );
                    });
                });
                // AV
                body.row(ROW_HEIGHT, |mut row| {
                    row.col(|ui| {
                        ui.label("Atrioventricular node");
                    });
                    row.col(|ui| {
                        ui.add(
                            egui::Slider::new(
                                &mut model.common.propagation_velocities.atrioventricular,
                                0.01..=10.0,
                            )
                            .suffix(" m/s"),
                        );
                    });
                    row.col(|ui| {
                        ui.add(
                            egui::Label::new(
                                "Desired propagation velocity in the \
                                    atrioventricular node in m/s. Note that the \
                                    maximum propagation velocity is limited \
                                    by the voxel size and sample rate.",
                            )
                            .truncate(),
                        );
                    });
                });
                // HPS
                body.row(ROW_HEIGHT, |mut row| {
                    row.col(|ui| {
                        ui.label("His-Purkinje S.");
                    });
                    row.col(|ui| {
                        ui.add(
                            egui::Slider::new(
                                &mut model.common.propagation_velocities.hps,
                                0.01..=10.0,
                            )
                            .suffix(" m/s"),
                        );
                    });
                    row.col(|ui| {
                        ui.add(
                            egui::Label::new(
                                "Desired propagation velocity in the \
                                    His-Purkinje system node in m/s. Note that the \
                                    maximum propagation velocity is limited \
                                    by the voxel size and sample rate.",
                            )
                            .truncate(),
                        );
                    });
                });
                // Ventricle
                body.row(ROW_HEIGHT, |mut row| {
                    row.col(|ui| {
                        ui.label("Ventricle");
                    });
                    row.col(|ui| {
                        ui.add(
                            egui::Slider::new(
                                &mut model.common.propagation_velocities.ventricle,
                                0.01..=10.0,
                            )
                            .suffix(" m/s"),
                        );
                    });
                    row.col(|ui| {
                        ui.add(
                            egui::Label::new(
                                "Desired propagation velocity in the \
                                    ventricle in m/s. Note that the \
                                    maximum propagation velocity is limited \
                                    by the voxel size and sample rate.",
                            )
                            .truncate(),
                        );
                    });
                });
                // Pathological
                if model.common.pathological {
                    body.row(ROW_HEIGHT, |mut row| {
                        row.col(|ui| {
                            ui.label("Pathological");
                        });
                        row.col(|ui| {
                            ui.add(
                                egui::Slider::new(
                                    &mut model.common.propagation_velocities.pathological,
                                    0.01..=10.0,
                                )
                                .suffix(" m/s"),
                            );
                        });
                        row.col(|ui| {
                            ui.add(
                                egui::Label::new(
                                    "Desired propagation velocity in the \
                                    pathological tissue in m/s. Note that the \
                                    maximum propagation velocity is limited \
                                    by the voxel size and sample rate.",
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
fn draw_handcrafted_settings(ui: &mut egui::Ui, handcrafted: &mut Handcrafted, patholoical: bool) {
    ui.label(egui::RichText::new("Handcrafted Model Settings").underline());
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
                // sa x center
                body.row(ROW_HEIGHT, |mut row| {
                    row.col(|ui| {
                        ui.label("X Center SA");
                    });
                    row.col(|ui| {
                        ui.add(egui::Slider::new(
                            &mut handcrafted.sa_x_center_percentage,
                            0.0..=1.0,
                        ));
                    });
                    row.col(|ui| {
                        ui.add(
                            egui::Label::new(
                                "The center of the sinoatrial node \
                                    in x-direction in percent.",
                            )
                            .truncate(),
                        );
                    });
                });
                // sa y center
                body.row(ROW_HEIGHT, |mut row| {
                    row.col(|ui| {
                        ui.label("Y Center SA");
                    });
                    row.col(|ui| {
                        ui.add(egui::Slider::new(
                            &mut handcrafted.sa_y_center_percentage,
                            0.0..=1.0,
                        ));
                    });
                    row.col(|ui| {
                        ui.add(
                            egui::Label::new(
                                "The center of the sinoatrial node \
                                    in y-direction in percent.",
                            )
                            .truncate(),
                        );
                    });
                });
                // include atrium
                body.row(ROW_HEIGHT, |mut row| {
                    row.col(|ui| {
                        ui.label("Include Atrium");
                    });
                    row.col(|ui| {
                        ui.checkbox(&mut handcrafted.include_atrium, "");
                    });
                    row.col(|ui| {
                        ui.add(
                            egui::Label::new("Wether to include atrial tissue or not.").truncate(),
                        );
                    });
                });
                if handcrafted.include_atrium {
                    // atrium y stop
                    body.row(ROW_HEIGHT, |mut row| {
                        row.col(|ui| {
                            ui.label("Y Stop Atrium");
                        });
                        row.col(|ui| {
                            ui.add(egui::Slider::new(
                                &mut handcrafted.atrium_y_start_percentage,
                                0.0..=1.0,
                            ));
                        });
                        row.col(|ui| {
                            ui.add(
                                egui::Label::new(
                                    "The end of the atrium \
                                    / start of the ventricles
                                    in y-direction in percent.",
                                )
                                .truncate(),
                            );
                        });
                    });
                }
                // include av
                body.row(ROW_HEIGHT, |mut row| {
                    row.col(|ui| {
                        ui.label("Include AV");
                    });
                    row.col(|ui| {
                        ui.checkbox(&mut handcrafted.include_av, "");
                    });
                    row.col(|ui| {
                        ui.add(egui::Label::new("Wether to include av tissue or not.").truncate());
                    });
                });
                if handcrafted.include_av {
                    // av x center
                    body.row(ROW_HEIGHT, |mut row| {
                        row.col(|ui| {
                            ui.label("X Center AV");
                        });
                        row.col(|ui| {
                            ui.add(egui::Slider::new(
                                &mut handcrafted.av_x_center_percentage,
                                0.0..=1.0,
                            ));
                        });
                        row.col(|ui| {
                            ui.add(
                                egui::Label::new(
                                    "The center of the atrioventricular node \
                                    in x-direction in percent.",
                                )
                                .truncate(),
                            );
                        });
                    });
                }

                // include hps
                body.row(ROW_HEIGHT, |mut row| {
                    row.col(|ui| {
                        ui.label("Include HPS");
                    });
                    row.col(|ui| {
                        ui.checkbox(&mut handcrafted.include_hps, "");
                    });
                    row.col(|ui| {
                        ui.add(egui::Label::new("Wether to include hps tissue or not.").truncate());
                    });
                });
                if handcrafted.include_hps {
                    // hps y stop
                    body.row(ROW_HEIGHT, |mut row| {
                        row.col(|ui| {
                            ui.label("Y Stop HPS");
                        });
                        row.col(|ui| {
                            ui.add(egui::Slider::new(
                                &mut handcrafted.hps_y_stop_percentage,
                                0.0..=1.0,
                            ));
                        });
                        row.col(|ui| {
                            ui.add(
                                egui::Label::new(
                                    "The end of the His-Purkinje-System \
                                    in y-direction in percent.",
                                )
                                .truncate(),
                            );
                        });
                    });
                    // hps x start
                    body.row(ROW_HEIGHT, |mut row| {
                        row.col(|ui| {
                            ui.label("X Start HPS");
                        });
                        row.col(|ui| {
                            ui.add(egui::Slider::new(
                                &mut handcrafted.hps_x_start_percentage,
                                0.0..=1.0,
                            ));
                        });
                        row.col(|ui| {
                            ui.add(
                                egui::Label::new(
                                    "The start of the His-Purkinje-System \
                                    in x-direction in percent.",
                                )
                                .truncate(),
                            );
                        });
                    });
                    // hps x stop
                    body.row(ROW_HEIGHT, |mut row| {
                        row.col(|ui| {
                            ui.label("X Stop HPS");
                        });
                        row.col(|ui| {
                            ui.add(egui::Slider::new(
                                &mut handcrafted.hps_x_stop_percentage,
                                0.0..=1.0,
                            ));
                        });
                        row.col(|ui| {
                            ui.add(
                                egui::Label::new(
                                    "The end of the His-Purkinje-System \
                                    in x-direction in percent.",
                                )
                                .truncate(),
                            );
                        });
                    });
                    // hps y up
                    body.row(ROW_HEIGHT, |mut row| {
                        row.col(|ui| {
                            ui.label("Y Up HPS");
                        });
                        row.col(|ui| {
                            ui.add(egui::Slider::new(
                                &mut handcrafted.hps_y_up_percentage,
                                0.0..=1.0,
                            ));
                        });
                        row.col(|ui| {
                            ui.add(
                                egui::Label::new(
                                    "The end of the upwards portion \
                                    of the His-Purkinje-System \
                                    in x-direction in percent.",
                                )
                                .truncate(),
                            );
                        });
                    });
                }
                if patholoical {
                    // pathology x start
                    body.row(ROW_HEIGHT, |mut row| {
                        row.col(|ui| {
                            ui.label("X Start Pathology");
                        });
                        row.col(|ui| {
                            ui.add(egui::Slider::new(
                                &mut handcrafted.pathology_x_start_percentage,
                                0.0..=1.0,
                            ));
                        });
                        row.col(|ui| {
                            ui.add(
                                egui::Label::new(
                                    "The start of the pathology \
                                    in x-direction in percent.",
                                )
                                .truncate(),
                            );
                        });
                    });
                    // pathology x stop
                    body.row(ROW_HEIGHT, |mut row| {
                        row.col(|ui| {
                            ui.label("X Stop Pathology");
                        });
                        row.col(|ui| {
                            ui.add(egui::Slider::new(
                                &mut handcrafted.pathology_x_stop_percentage,
                                0.0..=1.0,
                            ));
                        });
                        row.col(|ui| {
                            ui.add(
                                egui::Label::new(
                                    "The end of the pathology \
                                    in x-direction in percent.",
                                )
                                .truncate(),
                            );
                        });
                    });
                    // pathology y start
                    body.row(ROW_HEIGHT, |mut row| {
                        row.col(|ui| {
                            ui.label("Y Start Pathology");
                        });
                        row.col(|ui| {
                            ui.add(egui::Slider::new(
                                &mut handcrafted.pathology_y_start_percentage,
                                0.0..=1.0,
                            ));
                        });
                        row.col(|ui| {
                            ui.add(
                                egui::Label::new(
                                    "The start of the pathology \
                                    in y-direction in percent.",
                                )
                                .truncate(),
                            );
                        });
                    });
                    // pathology y start
                    body.row(ROW_HEIGHT, |mut row| {
                        row.col(|ui| {
                            ui.label("Y Stop Pathology");
                        });
                        row.col(|ui| {
                            ui.add(egui::Slider::new(
                                &mut handcrafted.pathology_y_stop_percentage,
                                0.0..=1.0,
                            ));
                        });
                        row.col(|ui| {
                            ui.add(
                                egui::Label::new(
                                    "The end of the pathology \
                                    in y-direction in percent.",
                                )
                                .truncate(),
                            );
                        });
                    });
                }
            });
        if patholoical {
            ui.add_space(7.0 * ROW_HEIGHT);
        } else {
            ui.add_space(2.0 * ROW_HEIGHT);
        }
    });
}

#[allow(clippy::too_many_lines)]
#[tracing::instrument(skip_all, level = "trace")]
fn draw_mri_settings(ui: &mut egui::Ui, mri: &mut Mri, _patholoical: bool) {
    ui.label(egui::RichText::new("MRI Model Settings").underline());
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
                // Path
                body.row(ROW_HEIGHT, |mut row| {
                    row.col(|ui| {
                        ui.label("Path");
                    });
                    row.col(|ui| {
                        let mut path = mri
                            .path
                            .to_str()
                            .unwrap_or_else(|| {
                                error!("MRI path contains invalid UTF-8: {:?}", mri.path);
                                "<invalid path>"
                            })
                            .to_string();
                        ui.add(egui::TextEdit::singleline(&mut path));
                        mri.path = PathBuf::from(path);
                    });
                    row.col(|ui| {
                        ui.add(egui::Label::new("The path to the .nii file.").truncate());
                    });
                });
            });
    });
}
