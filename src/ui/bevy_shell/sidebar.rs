//! Sidebar rail — spawning, visual states, collapse/expand, auto-collapse.

use bevy::prelude::*;

use super::content_area::ShellRoot;
use crate::{
    core::scenario::Status,
    ui::{colors, SidebarState, UiState},
    ProjectState, ScenarioList, SelectedSenario,
};

// ── Viewport threshold for auto-collapse ─────────────────────────────────────

const COLLAPSE_THRESHOLD_PX: f32 = 900.0;

// ── Components ────────────────────────────────────────────────────────────────

/// Marker for the sidebar root node (Column).
#[derive(Component, Debug)]
pub struct SidebarNode;

/// Marks a navigation button and its target view.
#[derive(Component, Debug, Clone)]
pub struct NavItem {
    pub target: UiState,
}

/// Marker placed on nav items whose preconditions are not satisfied.
#[derive(Component, Debug)]
pub struct NavItemDisabled;

/// Marker for the label text child inside a nav button (hidden when collapsed).
#[derive(Component, Debug)]
pub struct NavItemLabel;

/// Marker for the collapse/expand chevron button.
#[derive(Component, Debug)]
pub struct CollapseChevron;

/// Marker for the text node inside the chevron button so its label can be
/// flipped between `<` (expanded) and `>` (collapsed).
#[derive(Component, Debug)]
pub struct ChevronText;

// ── Spawn helpers ─────────────────────────────────────────────────────────────

/// Spawns the sidebar as the first child of `ShellRoot` and populates it.
/// Must run *after* `spawn_root_layout` so the root already exists.
#[tracing::instrument(skip_all)]
pub fn spawn_sidebar(
    mut commands: Commands,
    sidebar_state: Res<SidebarState>,
    roots: Query<Entity, With<ShellRoot>>,
) {
    let Ok(root) = roots.single() else {
        return;
    };

    let sidebar = commands
        .spawn((
            SidebarNode,
            Node {
                flex_direction: FlexDirection::Column,
                width: Val::Px(sidebar_state.width),
                height: Val::Percent(100.0),
                min_width: Val::Px(56.0),
                overflow: Overflow::clip(),
                ..default()
            },
            BackgroundColor(colors::BG0),
        ))
        .with_children(|sb| {
            // Logo area
            sb.spawn((
                Button,
                NavItem {
                    target: UiState::Home,
                },
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(56.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(colors::BG1),
            ))
            .with_children(|logo| {
                logo.spawn((
                    Text::new("CT"),
                    TextFont {
                        font_size: 20.0,
                        ..default()
                    },
                    TextColor(colors::ORANGE),
                ));
            });

            spawn_separator(sb);

            // Nav items (in order: Home, Explorer, Scenario, Results, Volumetric)
            // ASCII icon placeholders — no special font needed.
            for (icon, label, target) in [
                ("[H]", "Home", UiState::Home),
                ("[E]", "Explorer", UiState::Explorer),
                ("[S]", "Scenario", UiState::Scenario),
                ("[R]", "Results", UiState::Results),
                ("[V]", "Volumetric", UiState::Volumetric),
            ] {
                spawn_nav_button(sb, icon, label, target);
            }

            // Spacer
            sb.spawn(Node {
                flex_grow: 1.0,
                ..default()
            });

            spawn_separator(sb);

            // Scheduler nav item
            spawn_nav_button(sb, "[K]", "Scheduler", UiState::Scheduler);

            // Collapse chevron
            sb.spawn((
                CollapseChevron,
                Button,
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(40.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(colors::BG0),
            ))
            .with_children(|chev| {
                chev.spawn((
                    ChevronText,
                    Text::new("<"),
                    TextFont {
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(colors::GREY1),
                ));
            });
        })
        .id();

    // Insert sidebar as first child of the root row
    commands.entity(root).insert_children(0, &[sidebar]);
}

#[tracing::instrument(skip_all)]
fn spawn_separator(parent: &mut ChildSpawnerCommands) {
    parent.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(1.0),
            ..default()
        },
        BackgroundColor(colors::BG3),
    ));
}

#[tracing::instrument(skip_all)]
fn spawn_nav_button(parent: &mut ChildSpawnerCommands, icon: &str, label: &str, target: UiState) {
    parent
        .spawn((
            NavItem { target },
            Button,
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(48.0),
                align_items: AlignItems::Center,
                padding: UiRect::left(Val::Px(16.0)),
                column_gap: Val::Px(10.0),
                ..default()
            },
            BackgroundColor(Color::NONE),
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new(icon.to_string()),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(colors::GREY1),
            ));
            btn.spawn((
                NavItemLabel,
                Text::new(label.to_string()),
                TextFont {
                    font_size: 13.0,
                    ..default()
                },
                TextColor(colors::FG1),
            ));
        });
}

// ── Visual-state system ───────────────────────────────────────────────────────

/// Updates nav item background and text colours to match interaction/active/disabled state.
#[allow(clippy::type_complexity)]
#[tracing::instrument(skip_all)]
pub fn update_nav_item_visual_states(
    ui_state: Res<State<UiState>>,
    mut nav_items: Query<
        (
            Entity,
            &NavItem,
            &Interaction,
            &mut BackgroundColor,
            Option<&NavItemDisabled>,
        ),
        With<Button>,
    >,
    mut text_query: Query<(&ChildOf, &mut TextColor)>,
) {
    for (entity, nav_item, interaction, mut bg, disabled) in &mut nav_items {
        let is_active = nav_item.target == *ui_state.get() && disabled.is_none();
        let is_disabled = disabled.is_some();

        // Background
        bg.0 = if is_active {
            colors::BG1
        } else if is_disabled {
            Color::NONE
        } else {
            match interaction {
                Interaction::Hovered => colors::BG3,
                _ => Color::NONE,
            }
        };

        // Text colour for all direct text children
        for (child_of, mut text_color) in &mut text_query {
            if child_of.parent() == entity {
                text_color.0 = if is_disabled {
                    colors::BG3
                } else if is_active {
                    colors::ORANGE
                } else {
                    match interaction {
                        Interaction::Hovered => colors::FG0,
                        _ => colors::GREY1,
                    }
                };
            }
        }
    }
}

/// Adds/removes `NavItemDisabled` based on precondition guards.
///
/// This is the single authority over `NavItemDisabled`. It runs only when one
/// of the relevant resources changes, eliminating per-frame command churn that
/// was causing visual flickering.
#[tracing::instrument(skip_all)]
pub fn apply_nav_item_preconditions(
    mut commands: Commands,
    nav_items: Query<(Entity, &NavItem), With<Button>>,
    selected_scenario: Res<SelectedSenario>,
    scenario_list: Res<ScenarioList>,
    project_state: Res<ProjectState>,
) {
    // Only re-evaluate when something relevant actually changed.
    if !selected_scenario.is_changed()
        && !scenario_list.is_changed()
        && !project_state.is_changed()
    {
        return;
    }

    let no_project = project_state.current_path.is_none();

    for (entity, nav_item) in &nav_items {
        let disabled = if no_project {
            // Without a project, everything except Home is disabled.
            !matches!(nav_item.target, UiState::Home)
        } else {
            match nav_item.target {
                UiState::Scenario => selected_scenario.index.is_none(),
                UiState::Results | UiState::Volumetric => {
                    !selected_scenario.index.is_some_and(|i| {
                        scenario_list
                            .entries
                            .get(i)
                            .is_some_and(|e| e.scenario.get_status() == &Status::Done)
                    })
                }
                _ => false,
            }
        };
        if disabled {
            commands.entity(entity).insert(NavItemDisabled);
        } else {
            commands.entity(entity).remove::<NavItemDisabled>();
        }
    }
}

/// Navigates to a nav item's target when it is pressed and not disabled.
#[allow(clippy::type_complexity)]
#[tracing::instrument(skip_all)]
pub fn handle_nav_item_click(
    nav_items: Query<
        (&NavItem, &Interaction, Option<&NavItemDisabled>),
        (With<Button>, Changed<Interaction>),
    >,
    mut next_state: ResMut<NextState<UiState>>,
) {
    for (nav_item, interaction, disabled) in &nav_items {
        if disabled.is_none() && *interaction == Interaction::Pressed {
            next_state.set(nav_item.target);
        }
    }
}

// ── Collapse / expand ─────────────────────────────────────────────────────────

/// Toggles `SidebarState` when the chevron button is pressed.
#[tracing::instrument(skip_all)]
pub fn handle_chevron_click(
    chevrons: Query<&Interaction, (With<CollapseChevron>, Changed<Interaction>)>,
    mut sidebar_state: ResMut<SidebarState>,
) {
    for interaction in &chevrons {
        if *interaction == Interaction::Pressed {
            sidebar_state.expanded = !sidebar_state.expanded;
            sidebar_state.user_expanded = sidebar_state.expanded;
            sidebar_state.width = if sidebar_state.expanded { 200.0 } else { 56.0 };
        }
    }
}

/// Applies the current `SidebarState::width` to the sidebar node and
/// shows/hides `NavItemLabel` nodes and flips the chevron based on expanded state.
#[tracing::instrument(skip_all)]
pub fn apply_sidebar_width(
    sidebar_state: Res<SidebarState>,
    mut sidebars: Query<&mut Node, With<SidebarNode>>,
    mut labels: Query<&mut Node, (With<NavItemLabel>, Without<SidebarNode>)>,
    mut chevrons: Query<&mut Text, With<ChevronText>>,
) {
    if !sidebar_state.is_changed() {
        return;
    }
    for mut node in &mut sidebars {
        node.width = Val::Px(sidebar_state.width);
    }
    let display = if sidebar_state.expanded {
        Display::Flex
    } else {
        Display::None
    };
    for mut node in &mut labels {
        node.display = display;
    }
    let chevron_char = if sidebar_state.expanded { "<" } else { ">" };
    for mut text in &mut chevrons {
        text.0 = chevron_char.to_string();
    }
}

/// Auto-collapse the sidebar when the viewport is narrower than
/// `COLLAPSE_THRESHOLD_PX`, and restore when it widens again.
#[tracing::instrument(skip_all)]
pub fn auto_collapse_on_narrow_viewport(
    windows: Query<&Window>,
    mut sidebar_state: ResMut<SidebarState>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let width = window.resolution.width();
    let should_be_collapsed = width < COLLAPSE_THRESHOLD_PX;

    if should_be_collapsed && sidebar_state.expanded {
        sidebar_state.expanded = false;
        sidebar_state.width = 56.0;
        // user_expanded is NOT changed — it retains the user's preference.
    } else if !should_be_collapsed && !sidebar_state.expanded && sidebar_state.user_expanded {
        sidebar_state.expanded = true;
        sidebar_state.width = 200.0;
    }
}
