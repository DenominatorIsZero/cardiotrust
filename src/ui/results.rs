use bevy_egui::{egui, EguiContexts};

#[allow(clippy::module_name_repetitions)]
pub fn draw_ui_results(mut contexts: EguiContexts) {
    egui::CentralPanel::default().show(contexts.ctx_mut(), |ui| {
        ui.label("Results");
    });
}
