use bevy_egui::{egui, EguiContexts};

pub fn draw_ui_volumetric(mut contexts: EguiContexts) {
    egui::SidePanel::left("volumetric_left_panel").show(contexts.ctx_mut(), |ui| {
        ui.label("Volumetric");
    });
}
