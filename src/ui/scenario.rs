mod algorithm;
mod data;

use bevy::prelude::*;

use bevy_egui::{egui, EguiContexts};
use egui::Align;

use self::{algorithm::draw_ui_scenario_algoriothm, data::draw_ui_scenario_data};
use crate::{core::scenario::Scenario, Scenarios, SelectedSenario};

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
