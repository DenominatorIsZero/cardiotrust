use egui_extras::{Column, TableBuilder};

use crate::core::scenario::Scenario;

pub fn draw_ui_scenario_algoriothm(parent: &mut egui::Ui, _scenario: &mut Scenario) {
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
                                ui.label(
                                    "Some long-winded explanation of how this\
                            parameter works or what not. I don't know yet how long I\
                            can make this but I guess I will find out. One way or another.",
                                );
                            });
                        });
                    });
            });
        });
}
