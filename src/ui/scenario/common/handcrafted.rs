use egui_extras::{Column, TableBuilder};

use super::super::{FIRST_COLUMN_WIDTH, PADDING, ROW_HEIGHT, SECOND_COLUMN_WIDTH};
use crate::core::config::model::Handcrafted;

#[allow(clippy::too_many_lines)]
#[tracing::instrument(skip_all, level = "trace")]
pub fn draw_handcrafted_settings(
    ui: &mut egui::Ui,
    handcrafted: &mut Handcrafted,
    patholoical: bool,
) {
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
