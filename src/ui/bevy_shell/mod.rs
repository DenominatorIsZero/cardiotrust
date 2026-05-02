//! Bevy-native navigation shell.
//!
//! Provides the persistent sidebar rail, breadcrumb bar, content area, and
//! keyboard-shortcut routing for the application's supported UI.

pub mod breadcrumb;
pub mod content_area;
pub mod explorer;
pub mod home;
pub mod project;
pub mod results;
pub mod routing;
pub mod scenario;
pub mod scheduler;
pub mod scroll;
pub mod sidebar;
pub mod volumetric;

use bevy::prelude::*;
use bevy_ui_widgets::ScrollbarPlugin;

use self::{
    breadcrumb::update_breadcrumb,
    content_area::spawn_root_layout,
    explorer::ExplorerViewPlugin,
    home::{
        despawn_home_view, spawn_home_view, sync_home_project_switch_guard, FolderDialogReceiver,
    },
    project::load_project_on_path_change,
    results::ResultsViewPlugin,
    routing::handle_keyboard_shortcuts,
    scenario::ScenarioViewPlugin,
    scheduler::SchedulerViewPlugin,
    scroll::{on_scroll_handler, send_scroll_events},
    sidebar::{
        apply_nav_item_preconditions, apply_sidebar_width, auto_collapse_on_narrow_viewport,
        handle_chevron_click, handle_nav_item_click, spawn_sidebar, update_nav_item_visual_states,
    },
    volumetric::VolumetricViewPlugin,
};
use crate::ui::UiState;

/// Plugin that registers all Bevy-shell systems.
#[derive(Debug)]
pub struct BevyShellPlugin;

impl Plugin for BevyShellPlugin {
    #[tracing::instrument(level = "info", skip(app))]
    fn build(&self, app: &mut App) {
        app.init_resource::<FolderDialogReceiver>();
        app.add_plugins(ScrollbarPlugin);
        app.add_observer(on_scroll_handler);

        // Explorer view — Bevy-native card grid.
        app.add_plugins(ExplorerViewPlugin);

        // Scenario editor view — Bevy-native tabbed layout.
        app.add_plugins(ScenarioViewPlugin);

        // Results gallery view — Bevy-native.
        app.add_plugins(ResultsViewPlugin);

        // Scheduler dashboard view — Bevy-native.
        app.add_plugins(SchedulerViewPlugin);

        // Volumetric view — Bevy-native workspace with scoped egui overlays.
        app.add_plugins(VolumetricViewPlugin);

        // Spawn the root layout once at startup.
        app.add_systems(
            Startup,
            (spawn_root_layout, spawn_sidebar, spawn_home_view).chain(),
        );

        // Home view — spawn on enter, despawn on exit.
        app.add_systems(OnEnter(UiState::Home), spawn_home_view)
            .add_systems(OnExit(UiState::Home), despawn_home_view);

        // Per-frame shell systems.
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
                sync_home_project_switch_guard,
                send_scroll_events,
            ),
        )
        .add_systems(Update, home::poll_folder_dialog)
        .add_systems(PreUpdate, auto_collapse_on_narrow_viewport);

        // Home view button handlers.
        app.add_systems(
            Update,
            (
                home::handle_open_project_button,
                home::handle_recent_project_click,
            )
                .run_if(in_state(UiState::Home)),
        );
    }
}
