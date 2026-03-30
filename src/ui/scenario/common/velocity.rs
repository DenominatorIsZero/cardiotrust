use egui_extras::{Column, TableBuilder};

use super::super::{FIRST_COLUMN_WIDTH, PADDING, ROW_HEIGHT, SECOND_COLUMN_WIDTH};
use crate::core::config::model::Model;

#[allow(clippy::too_many_lines)]
#[tracing::instrument(skip_all, level = "trace")]
pub fn draw_velocity_settings(ui: &mut egui::Ui, model: &mut Model) {
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
