//! Empty-state node — shown when zero scenarios pass the active filter.

use bevy::prelude::*;

use super::{
    card::create_new_scenario,
    toolbar::{fuzzy_match, SearchFocused, SearchQuery, StatusFilter},
    ExplorerGridNode,
};
use crate::{ui::colors, ProjectState, ScenarioList, SelectedSenario};

// ── Marker components ─────────────────────────────────────────────────────────

/// Marker for the empty-state root node.
#[derive(Component, Debug)]
pub struct EmptyStateNode;

/// Marker for the generic "no scenarios" sub-group.
#[derive(Component, Debug)]
pub struct EmptyStateGenericGroup;

/// Marker for the search-specific "no matches" sub-group.
#[derive(Component, Debug)]
pub struct EmptyStateSearchGroup;

/// Marker for the "New Scenario" button inside the empty state.
#[derive(Component, Debug)]
pub struct EmptyStateNewScenarioButton;

/// Marker for the search-empty message text node.
#[derive(Component, Debug)]
pub struct EmptyStateSearchMessage;

/// Marker for the "Clear Search" button in the search-empty sub-group.
#[derive(Component, Debug)]
pub struct EmptyStateClearSearchButton;

// ── Spawn ─────────────────────────────────────────────────────────────────────

/// Spawns the empty-state node as a sibling of the grid inside `ExplorerViewRoot`.
#[tracing::instrument(skip_all)]
pub fn spawn_empty_state(
    mut commands: Commands,
    roots: Query<Entity, With<super::ExplorerViewRoot>>,
) {
    let Ok(root) = roots.single() else {
        return;
    };

    let node = commands
        .spawn((
            EmptyStateNode,
            Node {
                width: Val::Percent(100.0),
                flex_grow: 1.0,
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(16.0),
                display: Display::None, // hidden by default; toggled by `toggle_empty_state`
                ..default()
            },
            BackgroundColor(colors::BG0),
        ))
        .with_children(|col| {
            // ── Generic "no scenarios" sub-group ──────────────────────────────
            col.spawn((
                EmptyStateGenericGroup,
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(12.0),
                    display: Display::Flex,
                    ..default()
                },
            ))
            .with_children(|grp| {
                grp.spawn((
                    Text::new("No scenarios yet"),
                    TextFont {
                        font_size: 24.0,
                        ..default()
                    },
                    TextColor(colors::FG0),
                ));
                grp.spawn((
                    Text::new("Create your first scenario to get started."),
                    TextFont {
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(colors::GREY1),
                ));
                // New Scenario button
                grp.spawn((
                    EmptyStateNewScenarioButton,
                    Button,
                    Node {
                        padding: UiRect::axes(Val::Px(24.0), Val::Px(12.0)),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    BorderRadius::all(Val::Px(4.0)),
                    BackgroundColor(colors::ORANGE),
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new("+ New Scenario"),
                        TextFont {
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(colors::BG0),
                    ));
                });
            });

            // ── Search-specific "no matches" sub-group ────────────────────────
            col.spawn((
                EmptyStateSearchGroup,
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(12.0),
                    display: Display::None, // hidden until search returns no results
                    ..default()
                },
            ))
            .with_children(|grp| {
                grp.spawn((
                    EmptyStateSearchMessage,
                    Text::new(String::new()),
                    TextFont {
                        font_size: 20.0,
                        ..default()
                    },
                    TextColor(colors::FG0),
                ));
                grp.spawn((
                    Text::new("Clear search to see all scenarios"),
                    TextFont {
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(colors::GREY1),
                ));
                grp.spawn((
                    EmptyStateClearSearchButton,
                    Button,
                    Node {
                        padding: UiRect::axes(Val::Px(20.0), Val::Px(10.0)),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    BorderRadius::all(Val::Px(4.0)),
                    BackgroundColor(colors::BG3),
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new("Clear Search"),
                        TextFont {
                            font_size: 13.0,
                            ..default()
                        },
                        TextColor(colors::FG0),
                    ));
                });
            });
        })
        .id();

    commands.entity(root).add_child(node);
}

// ── Systems ────────────────────────────────────────────────────────────────────

/// Shows/hides `EmptyStateNode` and its sub-groups based on scenario count and
/// active search query. Also hides/shows the grid accordingly.
#[allow(clippy::type_complexity)]
#[tracing::instrument(skip_all)]
pub fn toggle_empty_state(
    scenario_list: Res<ScenarioList>,
    filter: Res<StatusFilter>,
    search: Res<SearchQuery>,
    mut empty_nodes: Query<&mut Node, With<EmptyStateNode>>,
    mut grid_nodes: Query<&mut Node, (With<ExplorerGridNode>, Without<EmptyStateNode>)>,
    mut generic_groups: Query<
        &mut Node,
        (
            With<EmptyStateGenericGroup>,
            Without<EmptyStateNode>,
            Without<ExplorerGridNode>,
            Without<EmptyStateSearchGroup>,
        ),
    >,
    mut search_groups: Query<
        &mut Node,
        (
            With<EmptyStateSearchGroup>,
            Without<EmptyStateNode>,
            Without<ExplorerGridNode>,
            Without<EmptyStateGenericGroup>,
        ),
    >,
    mut search_messages: Query<&mut Text, With<EmptyStateSearchMessage>>,
) {
    if !scenario_list.is_changed() && !filter.is_changed() && !search.is_changed() {
        return;
    }

    let query_lower = search.0.to_lowercase();

    // Count scenarios that pass both status filter and fuzzy search
    let visible_count = scenario_list
        .entries
        .iter()
        .filter(|e| {
            let s = &e.scenario;
            let status_ok = match *filter {
                StatusFilter::All => true,
                StatusFilter::Planning => {
                    s.get_status() == &crate::core::scenario::Status::Planning
                }
                StatusFilter::Queued => s.get_status() == &crate::core::scenario::Status::Scheduled,
                StatusFilter::Running => {
                    matches!(s.get_status(), crate::core::scenario::Status::Running(_))
                }
                StatusFilter::Done => s.get_status() == &crate::core::scenario::Status::Done,
                StatusFilter::Failed => s.get_status() == &crate::core::scenario::Status::Aborted,
            };
            if !status_ok {
                return false;
            }
            if query_lower.is_empty() {
                return true;
            }
            let comment = &s.comment;
            let id = s.get_id();
            let display_name = if comment.is_empty() {
                id.as_str()
            } else {
                comment.as_str()
            };
            fuzzy_match(&query_lower, &display_name.to_lowercase()).is_some()
                || fuzzy_match(&query_lower, &id.to_lowercase()).is_some()
        })
        .count();

    let is_search_active = !search.0.is_empty();
    let show_empty = visible_count == 0;
    let show_search_variant = show_empty && is_search_active;

    // Toggle empty-state root
    for mut node in &mut empty_nodes {
        node.display = if show_empty {
            Display::Flex
        } else {
            Display::None
        };
    }

    // Toggle grid
    for mut node in &mut grid_nodes {
        node.display = if show_empty {
            Display::None
        } else {
            Display::Grid
        };
    }

    // Toggle sub-groups
    for mut node in &mut generic_groups {
        node.display = if show_empty && !is_search_active {
            Display::Flex
        } else {
            Display::None
        };
    }

    for mut node in &mut search_groups {
        node.display = if show_search_variant {
            Display::Flex
        } else {
            Display::None
        };
    }

    // Update search message text
    if show_search_variant {
        for mut text in &mut search_messages {
            text.0 = format!("No scenarios match \"{}\"", search.0);
        }
    }
}

#[allow(clippy::type_complexity)]
/// "New Scenario" button inside the empty state.
#[tracing::instrument(skip_all)]
pub fn handle_empty_new_scenario_click(
    buttons: Query<
        &Interaction,
        (
            With<EmptyStateNewScenarioButton>,
            With<Button>,
            Changed<Interaction>,
        ),
    >,
    mut scenario_list: ResMut<ScenarioList>,
    mut selected: ResMut<SelectedSenario>,
    project_state: Res<ProjectState>,
) {
    for interaction in &buttons {
        if *interaction == Interaction::Pressed {
            create_new_scenario(&mut scenario_list, &mut selected, &project_state);
        }
    }
}

#[allow(clippy::type_complexity)]
/// "Clear Search" button inside the search-empty state.
#[tracing::instrument(skip_all)]
pub fn handle_empty_clear_search_click(
    buttons: Query<
        &Interaction,
        (
            With<EmptyStateClearSearchButton>,
            With<Button>,
            Changed<Interaction>,
        ),
    >,
    mut search: ResMut<SearchQuery>,
    mut focused: ResMut<SearchFocused>,
) {
    for interaction in &buttons {
        if *interaction == Interaction::Pressed {
            search.0.clear();
            focused.0 = false;
        }
    }
}
