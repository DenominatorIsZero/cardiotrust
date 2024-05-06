use egui::Align;
use egui_extras::{Column, TableBuilder};
use tracing::trace;

use super::{common::draw_ui_scenario_common, ROW_HEIGHT};
use crate::{
    core::{
        config::{
            model::{SensorArrayGeometry, SensorArrayMotion},
            simulation::Simulation,
        },
        scenario::{Scenario, Status},
    },
    ui::scenario::{FIRST_COLUMN_WIDTH, PADDING, SECOND_COLUMN_WIDTH},
};

/// Draws the data section of the scenario UI.
#[allow(clippy::too_many_lines, clippy::module_name_repetitions)]
#[tracing::instrument(skip(parent), level = "trace")]
pub fn draw_ui_scenario_data(parent: &mut egui::Ui, scenario: &mut Scenario) {
    trace!("Running system to draw scenario data UI.");
    parent.set_enabled(*scenario.get_status() == Status::Planning);
    let simulation = scenario.config.simulation.as_mut().unwrap();
    egui::ScrollArea::vertical()
        .id_source("simulation")
        .vscroll(true)
        .hscroll(false)
        .show(parent, |ui| {
            ui.heading("Simulation");
            ui.separator();
            draw_basic_settings(ui, simulation);
            draw_sensor_settings(ui, simulation);
            draw_general_heart_settings(ui, simulation);
            draw_ui_scenario_common(ui, &mut simulation.model);
        });
}

#[tracing::instrument(skip_all, level = "trace")]
fn draw_basic_settings(ui: &mut egui::Ui, simulation: &mut Simulation) {
    ui.label(egui::RichText::new("Basic Settings").underline());
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
                body.row(ROW_HEIGHT, |mut row| {
                    row.col(|ui| {
                        ui.label("Sample Rate");
                    });
                    row.col(|ui| {
                        ui.add(
                            egui::Slider::new(&mut simulation.sample_rate_hz, 1.0..=48000.0)
                                .suffix(" Hz"),
                        );
                    });
                    row.col(|ui| {
                        ui.add(
                            egui::Label::new(
                                "The sample rate of the simulation in Hz. Default: 2000.0 Hz.",
                            )
                            .truncate(true),
                        );
                    });
                });
                // Duration
                body.row(ROW_HEIGHT, |mut row| {
                    row.col(|ui| {
                        ui.label("Duration");
                    });
                    row.col(|ui| {
                        ui.add(
                            egui::Slider::new(&mut simulation.duration_s, 0.1..=60.0).suffix(" s"),
                        );
                    });
                    row.col(|ui| {
                        ui.add(
                            egui::Label::new(
                                "The duration of the simulation in seconds. Default: 1.0 s.",
                            )
                            .truncate(true),
                        );
                    });
                });
            });
    });
}

#[allow(clippy::too_many_lines)]
#[tracing::instrument(skip_all, level = "trace")]
fn draw_sensor_settings(ui: &mut egui::Ui, simulation: &mut Simulation) {
    ui.label(egui::RichText::new("Sensor Settings").underline());
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
                // sensor_type
                let sensor_geometry = &mut simulation.model.common.sensor_array_geometry;
                body.row(ROW_HEIGHT, |mut row| {
                    row.col(|ui| {
                        ui.label("Sensor Geometry");
                    });
                    row.col(|ui| {
                        egui::ComboBox::new("cb_sensor_geometry", "")
                            .selected_text(format!("{sensor_geometry:?}"))
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    sensor_geometry,
                                    SensorArrayGeometry::Cube,
                                    "Cube",
                                );
                                ui.selectable_value(
                                    sensor_geometry,
                                    SensorArrayGeometry::Cylinder,
                                    "Cylinder",
                                );
                            });
                    });
                    row.col(|ui| {
                        ui.add(
                            egui::Label::new(
                                "The spatial geometry of the sensor array. Default: Cylinder.",
                            )
                            .truncate(true),
                        );
                    });
                });// end row
                // sensor_motion
                let sensor_motion = &mut simulation.model.common.sensor_array_motion;
                body.row(ROW_HEIGHT, |mut row| {
                    row.col(|ui| {
                        ui.label("Sensor Motion");
                    });
                    row.col(|ui| {
                        egui::ComboBox::new("cb_sensor_motion", "")
                            .selected_text(format!("{sensor_motion:?}"))
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    sensor_motion,
                                    SensorArrayMotion::Static,
                                    "Static",
                                );
                                ui.selectable_value(
                                    sensor_motion,
                                    SensorArrayMotion::Grid,
                                    "Grid",
                                );
                            });
                    });
                    row.col(|ui| {
                        ui.add(
                            egui::Label::new(
                                "Whether the sensor array is static or moving along a grid. Default: Grid.",
                            )
                            .truncate(true),
                        );
                    });
                });// end row
                    // 3D sensors?
                body.row(ROW_HEIGHT, |mut row| {
                    row.col(|ui| {
                        ui.label("3D Sensors");
                    });
                    row.col(|ui| {
                            ui.add(egui::Checkbox::new(&mut simulation.model.common.three_d_sensors, ""));
                    });
                    row.col(|ui| {
                        ui.add(
                            egui::Label::new(
                                "Whether to use 3D sensors or not. Default: true.",
                            )
                            .truncate(true),
                        );
                    });
                }); // end row
                    // Sensor array origin
                let sensor_array_origin_mm = &mut simulation.model.common.sensor_array_origin_mm;
                body.row(ROW_HEIGHT, |mut row| {
                    row.col(|ui| {
                        ui.label("Sensors array origin");
                    });
                    row.col(|ui| {
                        ui.with_layout(egui::Layout::left_to_right(Align::TOP), |ui| {
                            ui.add(
                                egui::DragValue::new(&mut sensor_array_origin_mm[0])
                                    .prefix("x: ")
                                    .suffix(" mm"),
                            );
                            ui.add(
                                egui::DragValue::new(&mut sensor_array_origin_mm[1])
                                    .prefix("y: ")
                                    .suffix(" mm"),
                            );
                            ui.add(
                                egui::DragValue::new(&mut sensor_array_origin_mm[2])
                                    .prefix("z: ")
                                    .suffix(" mm"),
                            );
                        });
                    });
                    row.col(|ui| {
                        ui.add(
                            egui::Label::new(
                                "The origin of the sensor array with regard to the body coordinate system in mm.",
                            )
                            .truncate(true),
                        );
                    });
                }); // end row
                match sensor_geometry {
                    SensorArrayGeometry::Cube => {
                // Sensors per axis
                let sensors_per_axis = &mut simulation.model.common.sensors_per_axis;
                body.row(ROW_HEIGHT, |mut row| {
                    row.col(|ui| {
                        ui.label("Sensors per axis");
                    });
                    row.col(|ui| {
                        ui.with_layout(egui::Layout::left_to_right(Align::TOP), |ui| {
                            ui.add(egui::DragValue::new(&mut sensors_per_axis[0]).prefix("x: "));
                            ui.add(egui::DragValue::new(&mut sensors_per_axis[1]).prefix("y: "));
                            ui.add(egui::DragValue::new(&mut sensors_per_axis[2]).prefix("z: "));
                        });
                    });
                    row.col(|ui| {
                        ui.add(
                            egui::Label::new("The number of sensors used per axis.").truncate(true),
                        );
                    });
                }); // end row
                    // Sensor array size
                let sensor_array_size_mm = &mut simulation.model.common.sensor_array_size_mm;
                body.row(ROW_HEIGHT, |mut row| {
                    row.col(|ui| {
                        ui.label("Sensors array size");
                    });
                    row.col(|ui| {
                        ui.with_layout(egui::Layout::left_to_right(Align::TOP), |ui| {
                            ui.add(
                                egui::DragValue::new(&mut sensor_array_size_mm[0])
                                    .prefix("x: ")
                                    .suffix(" mm"),
                            );
                            ui.add(
                                egui::DragValue::new(&mut sensor_array_size_mm[1])
                                    .prefix("y: ")
                                    .suffix(" mm"),
                            );
                            ui.add(
                                egui::DragValue::new(&mut sensor_array_size_mm[2])
                                    .prefix("z: ")
                                    .suffix(" mm"),
                            );
                        });
                    });
                    row.col(|ui| {
                        ui.add(
                            egui::Label::new("The overall size of the sensor array in mm.")
                                .truncate(true),
                        );
                    });
                }); // end row
                    }
                    SensorArrayGeometry::Cylinder => {
                        // number of sensors
                let number_of_sensors = &mut simulation.model.common.number_of_sensors;
                body.row(ROW_HEIGHT, |mut row| {
                    row.col(|ui| {
                        ui.label("Number of sensors");
                    });
                    row.col(|ui| {
                            ui.add(egui::DragValue::new(number_of_sensors));
                    });
                    row.col(|ui| {
                        ui.add(
                            egui::Label::new("The number of sensors used.").truncate(true),
                        );
                    });
                }); // end row
                        // array radius
                let array_radius = &mut simulation.model.common.sensor_array_radius_mm;
                body.row(ROW_HEIGHT, |mut row| {
                    row.col(|ui| {
                        ui.label("Sensor array radius");
                    });
                    row.col(|ui| {
                            ui.add(egui::DragValue::new(array_radius).suffix(" mm"));
                    });
                    row.col(|ui| {
                        ui.add(
                            egui::Label::new("The radius of the sensor array.").truncate(true),
                        );
                    });
                }); // end row
                    }
                }
                if sensor_motion == &SensorArrayMotion::Grid {
                let motion_range = &mut simulation.model.common.sensor_array_motion_range_mm;
                body.row(ROW_HEIGHT, |mut row| {
                    row.col(|ui| {
                        ui.label("Motion range");
                    });
                    row.col(|ui| {
                        ui.with_layout(egui::Layout::left_to_right(Align::TOP), |ui| {
                            ui.add(egui::DragValue::new(&mut motion_range[0]).prefix("x: "));
                            ui.add(egui::DragValue::new(&mut motion_range[1]).prefix("y: "));
                            ui.add(egui::DragValue::new(&mut motion_range[2]).prefix("z: "));
                        });
                    });
                    row.col(|ui| {
                        ui.add(
                            egui::Label::new("The maximum offset of the grid to the sensor origin.").truncate(true),
                        );
                    });
                }); // end row
                let motion_steps = &mut simulation.model.common.sensor_array_motion_steps;
                body.row(ROW_HEIGHT, |mut row| {
                    row.col(|ui| {
                        ui.label("Motion steps");
                    });
                    row.col(|ui| {
                        ui.with_layout(egui::Layout::left_to_right(Align::TOP), |ui| {
                            ui.add(egui::DragValue::new(&mut motion_steps[0]).prefix("x: "));
                            ui.add(egui::DragValue::new(&mut motion_steps[1]).prefix("y: "));
                            ui.add(egui::DragValue::new(&mut motion_steps[2]).prefix("z: "));
                        });
                    });
                    row.col(|ui| {
                        ui.add(
                            egui::Label::new("The number of steps in the grid along each axis.").truncate(true),
                        );
                    });
                }); // end row
                }
            });
    });
}

#[allow(clippy::too_many_lines)]
#[tracing::instrument(skip_all, level = "trace")]
fn draw_general_heart_settings(ui: &mut egui::Ui, simulation: &mut Simulation) {
    ui.label(egui::RichText::new("General Heart Settings").underline());
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
                // Voxel Size
                body.row(ROW_HEIGHT, |mut row| {
                    row.col(|ui| {
                        ui.label("Voxel Size");
                    });
                    row.col(|ui| {
                        ui.add(
                            egui::Slider::new(
                                &mut simulation.model.common.voxel_size_mm,
                                1.0..=10.0,
                            )
                            .suffix(" mm"),
                        );
                    });
                    row.col(|ui| {
                        ui.add(
                            egui::Label::new(
                                "The desired size of the voxels in mm. \
                                    Might be rounded to the closest fit depending \
                                    on the choosen heart size.",
                            )
                            .truncate(true),
                        );
                    });
                }); // end row
                    // Heart Offset
                let heart_origin_mm = &mut simulation.model.common.heart_offset_mm;
                body.row(ROW_HEIGHT, |mut row| {
                    row.col(|ui| {
                        ui.label("Heart offset");
                    });
                    row.col(|ui| {
                        ui.with_layout(egui::Layout::left_to_right(Align::TOP), |ui| {
                            ui.add(
                                egui::DragValue::new(&mut heart_origin_mm[0])
                                    .prefix("x: ")
                                    .suffix(" mm"),
                            );
                            ui.add(
                                egui::DragValue::new(&mut heart_origin_mm[1])
                                    .prefix("y: ")
                                    .suffix(" mm"),
                            );
                            ui.add(
                                egui::DragValue::new(&mut heart_origin_mm[2])
                                    .prefix("z: ")
                                    .suffix(" mm"),
                            );
                        });
                    });
                    row.col(|ui| {
                        ui.add(
                            egui::Label::new(
                                "The offset of the heart with \
                                regard to the body coordinate system in mm.",
                            )
                            .truncate(true),
                        );
                    });
                }); // end row
                    // Heart size
                if let Some(handcrafted) = simulation.model.handcrafted.as_mut() {
                    let heart_size_mm = &mut handcrafted.heart_size_mm;
                    body.row(ROW_HEIGHT, |mut row| {
                        row.col(|ui| {
                            ui.label("Heart size");
                        });
                        row.col(|ui| {
                            ui.with_layout(egui::Layout::left_to_right(Align::TOP), |ui| {
                                ui.add(
                                    egui::DragValue::new(&mut heart_size_mm[0])
                                        .prefix("x: ")
                                        .suffix(" mm"),
                                );
                                ui.add(
                                    egui::DragValue::new(&mut heart_size_mm[1])
                                        .prefix("y: ")
                                        .suffix(" mm"),
                                );
                                ui.add(
                                    egui::DragValue::new(&mut heart_size_mm[2])
                                        .prefix("z: ")
                                        .suffix(" mm"),
                                );
                            });
                        });
                        row.col(|ui| {
                            ui.add(
                                egui::Label::new("The overall size of the heart in mm.")
                                    .truncate(true),
                            );
                        });
                    }); // end row
                }
            });
    });
}
