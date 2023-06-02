use egui::Align;
use egui_extras::{Column, TableBuilder};

use crate::core::{
    config::model::ControlFunction,
    scenario::{Scenario, Status},
};

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
                    .column(Column::auto().resizable(true))
                    .column(Column::auto().resizable(true))
                    .column(Column::initial(300.0).resizable(true))
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
                        // Control function
                        let control_function = &mut simulation.model.control_function;
                        body.row(30.0, |mut row| {
                            row.col(|ui| {
                                ui.label("Contorl function");
                            });
                            row.col(|ui| {
                                egui::ComboBox::new("cb_control_function", "")
                                    .selected_text(format!("{:?}", control_function))
                                    .show_ui(ui, |ui| {
                                        ui.selectable_value(
                                            control_function,
                                            ControlFunction::Sinosodal,
                                            "Sinosodal",
                                        );
                                        ui.selectable_value(
                                            control_function,
                                            ControlFunction::Ohara,
                                            "Ohara",
                                        );
                                    });
                            });
                            row.col(|ui| {
                                ui.label(
                                    "The control function used as the input tthe system\
                                    / The shape of the assumed current density curve.",
                                );
                            });
                        });
                        // Pathological
                        let _model_preset = &mut simulation.model;
                        body.row(30.0, |mut row| {
                            row.col(|ui| {
                                ui.label("Pathological");
                            });
                            row.col(|ui| {
                                ui.checkbox(&mut simulation.model.pathological, "");
                            });
                            row.col(|ui| {
                                ui.label(
                                    "Whether or not to place pathological tissue in the model.",
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
                        // measurement covariance mean
                        body.row(30.0, |mut row| {
                            row.col(|ui| {
                                ui.label("Measurement\ncovariance mean");
                            });
                            row.col(|ui| {
                                ui.add(
                                    egui::Slider::new(
                                        &mut simulation.model.measurement_covariance_mean,
                                        1e-30..=1e-10,
                                    )
                                    .logarithmic(true)
                                    .custom_formatter(|n, _| format!("{:+.4e}", n)),
                                );
                            });
                            row.col(|ui| {
                                ui.label(
                                    "The mean value of the measurement\
                                 noise covariance matrix.",
                                );
                            });
                        });
                        // measurement covariance std
                        body.row(30.0, |mut row| {
                            row.col(|ui| {
                                ui.label("Measurement\ncovariance std");
                            });
                            row.col(|ui| {
                                ui.add(egui::Slider::new(
                                    &mut simulation.model.measurement_covariance_std,
                                    0.0..=1.0,
                                ));
                            });
                            row.col(|ui| {
                                ui.label(
                                    "The standard deviation of the\
                                measurement noise covariance matrix.\
                                If this is zero, all diagonal values will\
                                be choosen as the mean.\
                                Otherwise they will be drawn from a normal\
                                distribution according\
                                to the mean value and standard deviation.",
                                );
                            });
                        });
                    });
            });
        });
}
