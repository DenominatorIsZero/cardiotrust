use egui::Align;
use egui_extras::{Column, TableBuilder};

use crate::core::{
    config::model::ControlFunction,
    model::spatial::voxels::VoxelType,
    scenario::{Scenario, Status},
};

use super::common::draw_ui_scenario_common;

pub fn draw_ui_scenario_data(parent: &mut egui::Ui, scenario: &mut Scenario) {
    parent.set_enabled(*scenario.get_status() == Status::Planning);
    let simulation = scenario.get_config_mut().simulation.as_mut().unwrap();
    egui::ScrollArea::vertical()
        .id_source("simulation")
        .show(parent, |ui| {
            ui.heading("Simulation");
            ui.separator();
            ui.push_id("simulation_parameter_table", |ui| {
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
                        // Sample Rate
                        body.row(30.0, |mut row| {
                            row.col(|ui| {
                                ui.label("Sample rate");
                            });
                            row.col(|ui| {
                                ui.add(
                                    egui::Slider::new(
                                        &mut simulation.sample_rate_hz,
                                        1.0..=48000.0,
                                    )
                                    .suffix(" Hz"),
                                );
                            });
                            row.col(|ui| {
                                ui.label(
                                    "The sample rate of the simulation in Hz. Default: 2000.0 Hz.",
                                );
                            });
                        });
                        // Duration
                        body.row(30.0, |mut row| {
                            row.col(|ui| {
                                ui.label("Duration");
                            });
                            row.col(|ui| {
                                ui.add(
                                    egui::Slider::new(&mut simulation.duration_s, 0.1..=60.0)
                                        .suffix(" s"),
                                );
                            });
                            row.col(|ui| {
                                ui.label(
                                    "The duration of the simulation in seconds. Default: 1.0 s.",
                                );
                            });
                        });
                        // Sensors per axis
                        let sensors_per_axis = &mut simulation.model.sensors_per_axis;
                        body.row(30.0, |mut row| {
                            row.col(|ui| {
                                ui.label("Sensors per axis");
                            });
                            row.col(|ui| {
                                ui.with_layout(egui::Layout::left_to_right(Align::TOP), |ui| {
                                    ui.add(
                                        egui::DragValue::new(&mut sensors_per_axis[0])
                                            .prefix("x: "),
                                    );
                                    ui.add(
                                        egui::DragValue::new(&mut sensors_per_axis[1])
                                            .prefix("y: "),
                                    );
                                    ui.add(
                                        egui::DragValue::new(&mut sensors_per_axis[2])
                                            .prefix("z: "),
                                    );
                                });
                            });
                            row.col(|ui| {
                                ui.label("The number of sensors used per axis.");
                            });
                        });
                        // Sensor array size
                        let sensor_array_size_mm = &mut simulation.model.sensor_array_size_mm;
                        body.row(30.0, |mut row| {
                            row.col(|ui| {
                                ui.label("Sensor array size");
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
                                ui.label("The overall size of the sensor array in mm.");
                            });
                        });
                        // Sensor array origin
                        let sensor_array_origin_mm = &mut simulation.model.sensor_array_origin_mm;
                        body.row(30.0, |mut row| {
                            row.col(|ui| {
                                ui.label("Sensor array origin");
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
                                ui.label(
                                    "The origin of the sensor array with \
                                regard to the body coordinate system in mm.",
                                );
                            });
                        });
                        // voxel size mm
                        body.row(30.0, |mut row| {
                            row.col(|ui| {
                                ui.label("Voxel size");
                            });
                            row.col(|ui| {
                                ui.add(
                                    egui::Slider::new(
                                        &mut simulation.model.voxel_size_mm,
                                        1.0..=10.0,
                                    )
                                    .suffix(" mm"),
                                );
                            });
                            row.col(|ui| {
                                ui.label(
                                    "The desired size of the voxels in mm. \
                                    Might be rounded to the closest fit depending \
                                    on the choosen heart size.",
                                );
                            });
                        });
                        // Heart size
                        let heart_size_mm = &mut simulation.model.heart_size_mm;
                        body.row(30.0, |mut row| {
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
                                ui.label("The overall size of the heart in mm.");
                            });
                        });
                        // Heart origin
                        let heart_origin_mm = &mut simulation.model.heart_origin_mm;
                        body.row(30.0, |mut row| {
                            row.col(|ui| {
                                ui.label("Heart origin");
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
                                ui.label(
                                    "The origin of the sensor array with \
                                regard to the body coordinate system in mm.",
                                );
                            });
                        });
                        draw_ui_scenario_common(&mut body, &mut simulation.model);
                    });
            });
        });
}
