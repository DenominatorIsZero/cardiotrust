use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use super::UiState;

pub fn draw_ui_topbar(
    mut commands: Commands,
    mut contexts: EguiContexts,
    ui_state: Res<State<UiState>>,
) {
    egui::TopBottomPanel::top("menu_panel").show(contexts.ctx_mut(), |ui| {
        ui.horizontal(|ui| {
            if ui
                .add_enabled(
                    ui_state.0 != UiState::Explorer,
                    egui::Button::new("Explorer"),
                )
                .clicked()
            {
                println!("Opening Explorer UI");
                commands.insert_resource(NextState(Some(UiState::Explorer)));
            };
            if ui
                .add_enabled(
                    ui_state.0 != UiState::Scenario,
                    egui::Button::new("Scenario"),
                )
                .clicked()
            {
                println!("Opening Scenario UI");
                commands.insert_resource(NextState(Some(UiState::Scenario)));
            };
            if ui
                .add_enabled(ui_state.0 != UiState::Results, egui::Button::new("Results"))
                .clicked()
            {
                println!("Opening Results UI");
                commands.insert_resource(NextState(Some(UiState::Results)));
            };
            if ui
                .add_enabled(
                    ui_state.0 != UiState::Volumetric,
                    egui::Button::new("Volumetric"),
                )
                .clicked()
            {
                println!("Opening Volumetric UI");
                commands.insert_resource(NextState(Some(UiState::Volumetric)));
            };
        });
    });
}
