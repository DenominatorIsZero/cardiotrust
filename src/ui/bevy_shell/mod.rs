//! Bevy-native navigation shell.
//!
//! Provides the persistent sidebar rail, breadcrumb bar, content area, and
//! keyboard-shortcut routing that run while `UiType::Bevy` is active.

pub mod breadcrumb;
pub mod content_area;
pub mod routing;
pub mod sidebar;

use bevy::prelude::*;

use self::{
    breadcrumb::update_breadcrumb,
    content_area::{despawn_root_layout, spawn_root_layout},
    routing::handle_keyboard_shortcuts,
    sidebar::{
        apply_nav_item_preconditions, apply_sidebar_width, auto_collapse_on_narrow_viewport,
        handle_chevron_click, handle_nav_item_click, spawn_sidebar, update_nav_item_visual_states,
    },
};
use crate::ui::UiType;

/// Plugin that registers all Bevy-shell systems.
#[derive(Debug)]
pub struct BevyShellPlugin;

impl Plugin for BevyShellPlugin {
    #[tracing::instrument(level = "info", skip(app))]
    fn build(&self, app: &mut App) {
        // Spawn / despawn the root layout when entering / exiting Bevy mode.
        app.add_systems(
            OnEnter(UiType::Bevy),
            (spawn_root_layout, spawn_sidebar).chain(),
        )
        .add_systems(OnExit(UiType::Bevy), despawn_root_layout);

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
            )
                .run_if(in_state(UiType::Bevy)),
        )
        .add_systems(
            PreUpdate,
            auto_collapse_on_narrow_viewport.run_if(in_state(UiType::Bevy)),
        );
    }
}
