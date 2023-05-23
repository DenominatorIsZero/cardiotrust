use bevy_egui::{egui, EguiContexts};

pub fn draw_ui_results(mut contexts: EguiContexts) {
    egui::CentralPanel::default().show(contexts.ctx_mut(), |ui| {
        ui.label("Results");
    });
}
