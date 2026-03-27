//! Empty-state node — shown when zero scenarios pass the active filter.

use bevy::prelude::*;

use super::{
    card::create_new_scenario,
    toolbar::StatusFilter,
    ExplorerGridNode,
};
use crate::{ui::colors, ProjectState, ScenarioList, SelectedSenario};

// ── Marker components ─────────────────────────────────────────────────────────

/// Marker for the empty-state root node.
#[derive(Component, Debug)]
pub struct EmptyStateNode;

/// Marker for the "New Scenario" button inside the empty state.
#[derive(Component, Debug)]
pub struct EmptyStateNewScenarioButton;

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
            col.spawn((
                Text::new("No scenarios yet"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(colors::FG0),
            ));
            col.spawn((
                Text::new("Create your first scenario to get started."),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(colors::GREY1),
            ));
            // New Scenario button
            col.spawn((
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
        })
        .id();

    commands.entity(root).add_child(node);
}

// ── Systems ────────────────────────────────────────────────────────────────────

/// Shows `EmptyStateNode` and hides the grid when no cards pass the filter;
/// vice-versa otherwise.
#[tracing::instrument(skip_all)]
pub fn toggle_empty_state(
    scenario_list: Res<ScenarioList>,
    filter: Res<StatusFilter>,
    mut empty_nodes: Query<&mut Node, With<EmptyStateNode>>,
    mut grid_nodes: Query<&mut Node, (With<ExplorerGridNode>, Without<EmptyStateNode>)>,
) {
    if !scenario_list.is_changed() && !filter.is_changed() {
        return;
    }

    let visible_count = match *filter {
        StatusFilter::All => scenario_list.entries.len(),
        StatusFilter::Planning => scenario_list
            .entries
            .iter()
            .filter(|e| e.scenario.get_status() == &crate::core::scenario::Status::Planning)
            .count(),
        StatusFilter::Queued => scenario_list
            .entries
            .iter()
            .filter(|e| e.scenario.get_status() == &crate::core::scenario::Status::Scheduled)
            .count(),
        StatusFilter::Running => scenario_list
            .entries
            .iter()
            .filter(|e| matches!(e.scenario.get_status(), crate::core::scenario::Status::Running(_)))
            .count(),
        StatusFilter::Done => scenario_list
            .entries
            .iter()
            .filter(|e| e.scenario.get_status() == &crate::core::scenario::Status::Done)
            .count(),
        StatusFilter::Failed => scenario_list
            .entries
            .iter()
            .filter(|e| e.scenario.get_status() == &crate::core::scenario::Status::Aborted)
            .count(),
    };

    let is_empty = visible_count == 0;

    for mut node in &mut empty_nodes {
        node.display = if is_empty { Display::Flex } else { Display::None };
    }
    for mut node in &mut grid_nodes {
        node.display = if is_empty { Display::None } else { Display::Grid };
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
