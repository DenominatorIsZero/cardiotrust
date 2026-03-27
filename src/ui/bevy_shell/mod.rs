//! Bevy-native navigation shell.
//!
//! Provides the persistent sidebar rail, breadcrumb bar, content area, and
//! keyboard-shortcut routing that run while `UiType::Bevy` is active.

pub mod breadcrumb;
pub mod content_area;
pub mod explorer;
pub mod home;
pub mod project;
pub mod routing;
pub mod sidebar;

use bevy::prelude::*;

use self::{
    breadcrumb::update_breadcrumb,
    content_area::{despawn_root_layout, spawn_root_layout},
    explorer::ExplorerViewPlugin,
    home::{despawn_home_view, spawn_home_view, FolderDialogReceiver},
    project::load_project_on_path_change,
    routing::handle_keyboard_shortcuts,
    sidebar::{
        apply_nav_item_preconditions, apply_sidebar_width, auto_collapse_on_narrow_viewport,
        handle_chevron_click, handle_nav_item_click, spawn_sidebar, update_nav_item_visual_states,
    },
};
use crate::ui::{UiState, UiType};

/// Plugin that registers all Bevy-shell systems.
#[derive(Debug)]
pub struct BevyShellPlugin;

impl Plugin for BevyShellPlugin {
    #[tracing::instrument(level = "info", skip(app))]
    fn build(&self, app: &mut App) {
        app.init_resource::<FolderDialogReceiver>();

        // Explorer view — Bevy-native card grid (UiType::Bevy only).
        app.add_plugins(ExplorerViewPlugin);

        // Spawn / despawn the root layout when entering / exiting Bevy mode.
        app.add_systems(
            OnEnter(UiType::Bevy),
            (spawn_root_layout, spawn_sidebar).chain(),
        )
        .add_systems(OnExit(UiType::Bevy), despawn_root_layout);

        // Home view — spawn on enter, despawn on exit.
        app.add_systems(OnEnter(UiState::Home), spawn_home_view)
            .add_systems(OnExit(UiState::Home), despawn_home_view);

        // Per-frame systems — only run while Bevy UI is active.
        app.add_systems(
            Update,
            (
                update_nav_item_visual_states,
                apply_nav_item_preconditions,
                handle_nav_item_click,
                handle_chevron_click,
                apply_sidebar_width,
                update_breadcrumb,
                handle_keyboard_shortcuts,
                load_project_on_path_change,
            )
                .run_if(in_state(UiType::Bevy)),
        )
        .add_systems(
            Update,
            home::poll_folder_dialog.run_if(in_state(UiType::Bevy)),
        )
        .add_systems(
            PreUpdate,
            auto_collapse_on_narrow_viewport.run_if(in_state(UiType::Bevy)),
        );

        // Home view button handlers — only when in Bevy mode and on the Home state.
        app.add_systems(
            Update,
            (
                home::handle_open_project_button,
                home::handle_recent_project_click,
            )
                .run_if(in_state(UiType::Bevy).and(in_state(UiState::Home))),
        );
    }
}
