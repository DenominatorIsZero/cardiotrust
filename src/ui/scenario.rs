mod algorithm;
pub mod common;
mod data;

use bevy::prelude::*;

use bevy_egui::{egui, EguiContexts};
use egui::Align;

use self::{algorithm::draw_ui_scenario_algoriothm, data::draw_ui_scenario_data};
use crate::{
    core::scenario::{Scenario, Status},
    ScenarioList, SelectedSenario,
};

#[allow(clippy::module_name_repetitions)]
pub fn draw_ui_scenario(
    mut contexts: EguiContexts,
    mut scenarios: ResMut<ScenarioList>,
    mut selected_scenario: ResMut<SelectedSenario>,
) {
    let context = contexts.ctx_mut();

    draw_ui_scenario_topbar(context, &mut scenarios, &mut selected_scenario);

    let index = selected_scenario.index.unwrap();
    let scenario = &mut scenarios.entries[index].scenario;
    draw_ui_scenario_central_panel(context, scenario);
}

fn draw_ui_scenario_topbar(
    context: &mut egui::Context,
    scenarios: &mut ResMut<ScenarioList>,
    selected_scenario: &mut ResMut<SelectedSenario>,
) {
    egui::TopBottomPanel::top("scenario_status").show(context, |ui| {
        ui.with_layout(egui::Layout::left_to_right(Align::TOP), |ui| {
            let index = selected_scenario.index.unwrap();
            let scenario = &mut scenarios.entries[index].scenario;
            ui.label(format!("Scenario with ID: {}", scenario.get_id()));
            ui.separator();
            ui.label(format!("Status: {}", scenario.get_status_str()));
            ui.separator();
            match scenario.get_status() {
                Status::Planning => {
                    if ui.button("Schedule").clicked() {
                        scenario.schedule().unwrap();
                    }
                }
                Status::Scheduled => {
                    if ui.button("Unschedule").clicked() {
                        scenario.unschedule().unwrap();
                    }
                }
                _ => (),
            }
            if ui.button("Save").clicked() {
                scenario.save().unwrap();
            }
            if ui.button("Delete").clicked() {
                scenario.delete().unwrap();
                scenarios.entries.remove(index);
                selected_scenario.index = Some(0);
            }
            ui.separator();
            let index = selected_scenario.index.unwrap();
            let scenario = &mut scenarios.entries[index].scenario;
            if ui
                .add(egui::TextEdit::multiline(&mut scenario.comment).desired_width(f32::INFINITY))
                .lost_focus()
            {
                scenario.save().unwrap();
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
