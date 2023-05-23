use bevy::prelude::*;

use bevy_egui::{egui, EguiContexts};
use egui::Align;
use egui_extras::{Column, TableBuilder};

use crate::{
    core::config::{simulation::ControlFunction, ModelPreset},
    core::scenario::Scenario,
    Scenarios, SelectedSenario,
};

pub fn draw_ui_scenario(
    _commands: Commands,
    mut contexts: EguiContexts,
    mut scenarios: ResMut<Scenarios>,
    selected_scenario: ResMut<SelectedSenario>,
) {
    let index = selected_scenario.index.unwrap();
    let scenario = &mut scenarios.scenarios[index];
    let context = contexts.ctx_mut();

    draw_ui_scenario_topbar(context, scenario);
    draw_ui_scenario_central_panel(context, scenario);
}

fn draw_ui_scenario_topbar(context: &mut egui::Context, scenario: &mut Scenario) {
    egui::TopBottomPanel::top("scenario_status").show(context, |ui| {
        ui.with_layout(egui::Layout::left_to_right(Align::TOP), |ui| {
            ui.label(format!("Scenario with ID: {}", scenario.get_id()));
            ui.separator();
            ui.label(format!("Status: {}", scenario.get_status_str()));
            ui.separator();
            if ui.button("Back").clicked() {
                todo!();
            }
            if ui.button("Forward").clicked() {
                todo!();
            }
            if ui.button("Delete").clicked() {
                todo!();
            }
        });
    });
}

fn draw_ui_scenario_central_panel(context: &mut egui::Context, scenario: &mut Scenario) {
    egui::CentralPanel::default().show(context, |ui| {
        ui.columns(2, |columns| {
            draw_ui_scenario_data(&mut columns[0], scenario);
            draw_ui_scenario_algoriothm(&mut columns[1], scenario);
        });
    });
}

fn draw_ui_scenario_algoriothm(parent: &mut egui::Ui, scenario: &mut Scenario) {
    egui::ScrollArea::vertical()
    .id_source("algorithm")
    .show(parent, |ui| {
        ui.heading("Algorithm");
        ui.separator();
        ui.push_id("algorithm_parameter_table", |ui| {
            TableBuilder::new(ui)
                .column(Column::auto().resizable(true))
                .column(Column::auto().resizable(true))
                .column(Column::remainder())
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
                    body.row(30.0, |mut row| {
                        row.col(|ui| {
                            ui.label("Parameter 1");
                        });
                        row.col(|ui| {
                            if ui.button("Change").clicked() {
                                todo!();
                            };
                        });
                        row.col(|ui| {
                            ui.label("Some long-winded explanation of how this parameter works or what not. I don't know yet how long I can make this but I guess I will find out. One way or another.");
                        });
                    });
                });
        });
    });
}

fn draw_ui_scenario_data(parent: &mut egui::Ui, scenario: &mut Scenario) {
    let config = scenario.get_config_mut();
    egui::ScrollArea::vertical()
        .id_source("simulation")
        .show(parent, |ui| {
            ui.heading("Simulation");
            ui.separator();
            ui.push_id("simulation_parameter_table", |ui| {
                TableBuilder::new(ui)
                    .column(Column::auto().resizable(true))
                    .column(Column::auto().resizable(true))
                    .column(Column::remainder())
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
                        draw_ui_scenario_row(
                            &mut body,
                            "Sample rate",
                            egui::Slider::new(
                                &mut config.simulation.as_mut().unwrap().sample_rate_hz,
                                1.0..=48000.0,
                            )
                            .suffix(" Hz"),
                            "The sample rate of the simulation in Hz. Default: 2000.0 Hz.",
                        );
                        // Duration
                        draw_ui_scenario_row(
                            &mut body,
                            "Duration",
                            egui::Slider::new(
                                &mut config.simulation.as_mut().unwrap().duration_s,
                                0.1..=60.0,
                            )
                            .suffix(" s"),
                            "The duration of the simulation in seconds. Default: 1.0 s.",
                        );
                        // Control function
                        let control_function = &mut config.simulation.as_mut().unwrap().control_function;
                        body.row(30.0, |mut row| {
                            row.col(|ui| {
                                ui.label("Contorl function");
                            });
                            row.col(|ui| {
                                egui::ComboBox::new("cb_control_function", "").selected_text(format!("{:?}", control_function)).show_ui(ui, |ui| {
                                    ui.selectable_value(control_function, ControlFunction::Sinosodal, "Sinosodal");
                                    ui.selectable_value(control_function, ControlFunction::Ohara, "Ohara");
                                });
                            });
                            row.col(|ui| {
                                ui.label("The control function used as the input to the system / The shape of the assumed current density curve.");
                            });
                        });
                        // Model preset
                        let model_preset = &mut config.simulation.as_mut().unwrap().model;
                        body.row(30.0, |mut row| {
                            row.col(|ui| {
                                ui.label("Model preset");
                            });
                            row.col(|ui| {
                                egui::ComboBox::new("cb_model_preset", "").selected_text(format!("{:?}", model_preset)).show_ui(ui, |ui| {
                                    ui.selectable_value(model_preset, ModelPreset::Healthy, "Healthy");
                                    ui.selectable_value(model_preset, ModelPreset::Scarred, "Scarred");
                                });
                            });
                            row.col(|ui| {
                                ui.label("The model preset to be used for the simulation.");
                            });
                        });
                        // Sensors per axis
                        let sensors_per_axis = &mut config.simulation.as_mut().unwrap().sensors_per_axis;
                        body.row(30.0, |mut row| {
                            row.col(|ui| {
                                ui.label("Sensors per axis");
                            });
                            row.col(|ui| {
                                ui.with_layout(egui::Layout::left_to_right(Align::TOP), |ui| {
                                    ui.add(egui::DragValue::new(
                                        &mut sensors_per_axis[0],
                                    ).prefix("x: "));
                                    ui.add(egui::DragValue::new(
                                        &mut sensors_per_axis[1],
                                    ).prefix("y: "));
                                    ui.add(egui::DragValue::new(
                                        &mut sensors_per_axis[2],
                                    ).prefix("z: "));
                                });
                            });
                            row.col(|ui| {
                                ui.label("The number of sensors used per axis.");
                            });
                        });
                });
            });
        });
}

fn draw_ui_scenario_row<T: egui::Widget>(
    body: &mut egui_extras::TableBody,
    name: &str,
    widget: T,
    description: &str,
) {
    body.row(30.0, |mut row| {
        row.col(|ui| {
            ui.label(name);
        });
        row.col(|ui| {
            ui.add(widget);
        });
        row.col(|ui| {
            ui.label(description);
        });
    });
}
