//! Bevy-native volumetric view with scoped egui overlays.

mod helpers;
mod spawn;
mod systems;
mod types;

use bevy::prelude::*;
use bevy_egui::EguiPrimaryContextPass;
use spawn::{despawn_volumetric_view, spawn_volumetric_view};
use systems::{
    collapse_overlay_on_outside_click, disable_camera_motion_over_volumetric_ui,
    draw_volumetric_overlays, handle_color_mode_button, handle_color_mode_option_click,
    handle_control_action_buttons, handle_plot_collapse_button, handle_plot_resize,
    handle_section_tab_click, handle_toolbar_buttons, log_camera_pose_on_f3,
    poll_screenshot_dialog, sync_overlay_panel_contents, sync_volumetric_layout,
    update_color_mode_dropdown, update_control_value_labels, update_plot_labels,
    update_section_tab_visuals, update_toolbar_visuals,
};
use types::{ScreenshotDialogReceiver, VolumetricViewState};

use crate::ui::{UiState, UiType};

#[derive(Debug)]
pub struct VolumetricViewPlugin;

impl Plugin for VolumetricViewPlugin {
    #[tracing::instrument(level = "info", skip(app))]
    fn build(&self, app: &mut App) {
        let volumetric_condition = in_state(UiType::Bevy).and(in_state(UiState::Volumetric));

        app.init_resource::<VolumetricViewState>()
            .init_resource::<ScreenshotDialogReceiver>()
            .add_systems(
                PreUpdate,
                systems::disable_camera_motion_over_volumetric_ui_preupdate
                    .before(bevy_editor_cam::input::default_camera_inputs)
                    .run_if(volumetric_condition.clone()),
            )
            .add_systems(
                OnEnter(UiState::Volumetric),
                spawn_volumetric_view.run_if(in_state(UiType::Bevy)),
            )
            .add_systems(
                OnExit(UiState::Volumetric),
                despawn_volumetric_view.run_if(in_state(UiType::Bevy)),
            )
            .add_systems(
                Update,
                (
                    sync_volumetric_layout,
                    sync_overlay_panel_contents,
                    update_section_tab_visuals,
                    update_toolbar_visuals,
                    update_plot_labels,
                    update_control_value_labels,
                    update_color_mode_dropdown,
                    poll_screenshot_dialog,
                    log_camera_pose_on_f3,
                    handle_section_tab_click,
                    collapse_overlay_on_outside_click,
                    handle_toolbar_buttons,
                    handle_color_mode_button,
                    handle_color_mode_option_click,
                    handle_control_action_buttons,
                    handle_plot_collapse_button,
                    handle_plot_resize,
                    disable_camera_motion_over_volumetric_ui,
                )
                    .run_if(volumetric_condition.clone()),
            )
            .add_systems(
                EguiPrimaryContextPass,
                draw_volumetric_overlays.run_if(volumetric_condition),
            );
    }
}
