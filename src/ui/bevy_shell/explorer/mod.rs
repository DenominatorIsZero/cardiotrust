//! Explorer view — Bevy-native card-grid layout for scenario browsing.
//!
//! Registered in `bevy_shell::BevyShellPlugin` behind the `UiType::Bevy` guard.
//!
//! # Module layout
//!
//! ```text
//! explorer/
//!   mod.rs          — ExplorerViewPlugin (registers all systems)
//!   card.rs         — ScenarioCard spawn / update systems
//!   context_menu.rs — ContextMenu overlay systems
//!   empty_state.rs  — EmptyState node
//!   thumbnail.rs    — ThumbnailCache resource + generation task
//!   toolbar.rs      — FilterBar, SortBar, SearchField systems
//! ```

pub mod card;
pub mod context_menu;
pub mod empty_state;
pub mod thumbnail;
pub mod toolbar;

use bevy::prelude::*;

use self::{
    card::{
        handle_card_click, handle_card_inline_edit, handle_card_quick_actions,
        handle_new_scenario_card_click, sync_cards_to_scenarios, update_active_card_border,
        update_card_hover, update_card_label_highlights, update_card_labels, CardEditMode,
        LastCardClick,
    },
    context_menu::{dismiss_context_menu, handle_context_menu_actions, spawn_context_menu},
    empty_state::{
        handle_empty_clear_search_click, handle_empty_new_scenario_click, spawn_empty_state,
        toggle_empty_state,
    },
    thumbnail::{poll_thumbnail_tasks, queue_thumbnail_generation, ThumbnailCache},
    toolbar::{
        apply_filter_and_sort, handle_new_scenario_toolbar_button, handle_search_clear_click,
        handle_search_field_click, handle_search_outside_click, handle_sort_click,
        handle_status_filter_click, handle_text_search_input, spawn_toolbar,
        update_search_display_text, update_search_field_visuals, update_toolbar_button_visuals,
        SearchFocused,
    },
};
use crate::ui::{bevy_shell::content_area::ContentSlot, UiState, UiType};

// ── Marker components ─────────────────────────────────────────────────────────

/// Marker for the root node of the entire Explorer view.
/// Despawning this (recursively) cleans up the view.
#[derive(Component, Debug)]
pub struct ExplorerViewRoot;

/// Marker for the grid root node that holds scenario cards.
#[derive(Component, Debug)]
pub struct ExplorerGridNode;

// ── Plugin ────────────────────────────────────────────────────────────────────

/// Plugin that owns the Bevy-native Explorer view.
#[derive(Debug)]
pub struct ExplorerViewPlugin;

impl Plugin for ExplorerViewPlugin {
    #[tracing::instrument(level = "info", skip(app))]
    fn build(&self, app: &mut App) {
        // Resources
        app.init_resource::<ThumbnailCache>();
        app.init_resource::<CardEditMode>();
        app.init_resource::<LastCardClick>();
        app.init_resource::<SearchFocused>();

        // Spawn / despawn the Explorer view when entering / exiting Explorer state
        // (only while the Bevy UI backend is active).
        // Order: root first, then toolbar and empty-state which parent into the root.
        app.add_systems(
            OnEnter(UiState::Explorer),
            (spawn_explorer_view, spawn_toolbar, spawn_empty_state)
                .chain()
                .run_if(in_state(UiType::Bevy)),
        )
        .add_systems(
            OnExit(UiState::Explorer),
            despawn_explorer_view.run_if(in_state(UiType::Bevy)),
        );

        // Per-frame systems — only run while Bevy UI + Explorer are active.
        // Nested sub-tuples work around the 20-element tuple limit.
        //
        // Ordering:
        //   handle_text_search_input → apply_filter_and_sort → update_card_label_highlights
        //   update_card_labels → update_card_label_highlights
        let explorer_condition = in_state(UiType::Bevy).and(in_state(UiState::Explorer));
        app.add_systems(
            Update,
            (
                (
                    update_grid_columns,
                    sync_cards_to_scenarios,
                    update_card_hover,
                    update_active_card_border,
                    toggle_empty_state,
                    // Search input must run before filter so query is already updated.
                    handle_text_search_input,
                    apply_filter_and_sort.after(handle_text_search_input),
                    queue_thumbnail_generation,
                    poll_thumbnail_tasks,
                    handle_card_click,
                    handle_card_inline_edit,
                ),
                (
                    handle_card_quick_actions,
                    // update_card_labels must run before update_card_label_highlights
                    update_card_labels,
                    update_card_label_highlights
                        .after(update_card_labels)
                        .after(apply_filter_and_sort),
                    handle_new_scenario_card_click,
                    handle_new_scenario_toolbar_button,
                    handle_status_filter_click,
                    handle_sort_click,
                    // Search-field UI systems
                    handle_search_field_click,
                    handle_search_outside_click,
                    handle_search_clear_click,
                    update_search_field_visuals,
                    update_search_display_text,
                    update_toolbar_button_visuals,
                    spawn_context_menu,
                    dismiss_context_menu,
                    handle_context_menu_actions,
                    handle_empty_new_scenario_click,
                    handle_empty_clear_search_click,
                ),
            )
                .run_if(explorer_condition),
        );
    }
}

// ── Spawn / despawn ───────────────────────────────────────────────────────────

/// Spawns the Explorer grid root node as a child of [`ContentSlot`].
#[tracing::instrument(skip_all)]
fn spawn_explorer_view(mut commands: Commands, content_slots: Query<Entity, With<ContentSlot>>) {
    let Ok(slot) = content_slots.single() else {
        return;
    };

    let root = commands
        .spawn((
            ExplorerViewRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                overflow: Overflow::scroll_y(),
                ..default()
            },
            BackgroundColor(crate::ui::colors::BG0),
        ))
        .with_children(|root| {
            // Grid node — cards are inserted here by sync_cards_to_scenarios
            root.spawn((
                ExplorerGridNode,
                Node {
                    width: Val::Percent(100.0),
                    display: Display::Grid,
                    grid_template_columns: RepeatedGridTrack::flex(3, 1.0),
                    column_gap: Val::Px(16.0),
                    row_gap: Val::Px(16.0),
                    padding: UiRect::all(Val::Px(16.0)),
                    ..default()
                },
            ));
        })
        .id();

    commands.entity(slot).add_child(root);
}

/// Despawns all [`ExplorerViewRoot`] entities.
#[tracing::instrument(skip_all)]
fn despawn_explorer_view(mut commands: Commands, roots: Query<Entity, With<ExplorerViewRoot>>) {
    for entity in &roots {
        commands.entity(entity).despawn();
    }
}

// ── Responsive columns ────────────────────────────────────────────────────────

/// Breakpoints (content area width → column count):
/// - ≥ 1400 px → 4 columns
/// - 1000–1399 px → 3 columns
/// - 700–999 px → 2 columns
/// - < 700 px → 1 column
#[tracing::instrument(skip_all)]
fn update_grid_columns(
    windows: Query<&Window>,
    mut grids: Query<&mut Node, With<ExplorerGridNode>>,
) {
    let Ok(window) = windows.single() else {
        return;
    };

    // Approximate the content area width by subtracting the sidebar (≤200 px).
    let content_width = window.resolution.width() - 200.0;

    let cols: u16 = if content_width >= 1400.0 {
        4
    } else if content_width >= 1000.0 {
        3
    } else if content_width >= 700.0 {
        2
    } else {
        1
    };

    for mut node in &mut grids {
        node.grid_template_columns = RepeatedGridTrack::flex(cols, 1.0);
    }
}
