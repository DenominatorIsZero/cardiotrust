//! Context menu overlay — spawned on right-click over a scenario card.
//!
//! The menu is positioned near the cursor and rendered at `ZIndex::Global(100)`.
//! It is dismissed on any click outside its bounds.

use bevy::prelude::*;

use crate::{
    core::scenario::Status,
    ui::{colors, UiState},
    ScenarioList, SelectedSenario,
};

// ── Components ────────────────────────────────────────────────────────────────

/// Marker for the context menu root node.
#[derive(Component, Debug)]
pub struct ContextMenuNode {
    /// The card entity this menu was opened for.
    pub for_entity: Entity,
    /// The scenario index this menu was opened for.
    pub scenario_index: usize,
}

/// Marks each menu item with its action.
#[derive(Component, Debug, Clone)]
pub enum ContextMenuAction {
    EditScenario,
    Copy,
    Delete,
    Schedule,
    Unschedule,
    OpenInResults,
    OpenInVolumetric,
}

// ── Systems ────────────────────────────────────────────────────────────────────

/// Spawns a context menu on right-click over a scenario card.
#[allow(clippy::too_many_arguments)]
#[tracing::instrument(skip_all)]
pub fn spawn_context_menu(
    mut commands: Commands,
    cards: Query<(Entity, &super::card::ScenarioCard, &Interaction)>,
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    existing_menus: Query<Entity, With<ContextMenuNode>>,
    scenario_list: Res<ScenarioList>,
) {
    if !mouse.just_pressed(MouseButton::Right) {
        return;
    }

    // Despawn any existing menu first.
    for entity in &existing_menus {
        commands.entity(entity).despawn();
    }

    // Find which card (if any) the cursor is over.
    let hovered_card = cards
        .iter()
        .find(|(_, _, interaction)| **interaction == Interaction::Hovered);

    let Some((card_entity, card, _)) = hovered_card else {
        return;
    };

    let Some(entry) = scenario_list.entries.get(card.index) else {
        return;
    };
    let status = entry.scenario.get_status();

    // Get cursor position.
    let cursor_pos = windows
        .single()
        .ok()
        .and_then(Window::cursor_position)
        .unwrap_or(Vec2::ZERO);

    let actions = context_actions_for(status);

    commands
        .spawn((
            ContextMenuNode {
                for_entity: card_entity,
                scenario_index: card.index,
            },
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(cursor_pos.x),
                top: Val::Px(cursor_pos.y),
                flex_direction: FlexDirection::Column,
                border: UiRect::all(Val::Px(1.0)),
                overflow: Overflow::clip(),
                min_width: Val::Px(160.0),
                ..default()
            },
            BorderRadius::all(Val::Px(4.0)),
            BackgroundColor(colors::BG1),
            BorderColor(colors::BG3),
            ZIndex(100),
        ))
        .with_children(|menu| {
            for action in actions {
                let label = action_label(&action);
                menu.spawn((
                    action,
                    Button,
                    Node {
                        width: Val::Percent(100.0),
                        padding: UiRect::axes(Val::Px(12.0), Val::Px(8.0)),
                        ..default()
                    },
                    BackgroundColor(Color::NONE),
                ))
                .with_children(|item| {
                    item.spawn((
                        Text::new(label),
                        TextFont {
                            font_size: 13.0,
                            ..default()
                        },
                        TextColor(colors::FG1),
                    ));
                });
            }
        });
}

/// Returns the list of applicable context actions for the given status.
#[tracing::instrument(level = "trace", skip_all)]
fn context_actions_for(status: &Status) -> Vec<ContextMenuAction> {
    let mut actions = vec![
        ContextMenuAction::EditScenario,
        ContextMenuAction::Copy,
        ContextMenuAction::Delete,
    ];
    match status {
        Status::Planning | Status::Aborted => actions.push(ContextMenuAction::Schedule),
        Status::Scheduled => actions.push(ContextMenuAction::Unschedule),
        Status::Done => {
            actions.push(ContextMenuAction::OpenInResults);
            actions.push(ContextMenuAction::OpenInVolumetric);
        }
        Status::Running(_) | Status::Simulating => {}
    }
    actions
}

#[tracing::instrument(level = "trace", skip_all)]
fn action_label(action: &ContextMenuAction) -> String {
    match action {
        ContextMenuAction::EditScenario => "Edit Scenario".to_string(),
        ContextMenuAction::Copy => "Copy Scenario".to_string(),
        ContextMenuAction::Delete => "Delete Scenario".to_string(),
        ContextMenuAction::Schedule => "Schedule".to_string(),
        ContextMenuAction::Unschedule => "Unschedule".to_string(),
        ContextMenuAction::OpenInResults => "Open in Results".to_string(),
        ContextMenuAction::OpenInVolumetric => "Open in Volumetric".to_string(),
    }
}

/// Despawns the context menu on any left-click, or on a right-click that lands
/// outside the menu.
///
/// Note: `ContextMenuNode` is not a `Button`, so it has no `Interaction` component.
/// We query menu item interactions separately to detect right-clicks over the menu.
#[tracing::instrument(skip_all)]
pub fn dismiss_context_menu(
    mut commands: Commands,
    mouse: Res<ButtonInput<MouseButton>>,
    menus: Query<Entity, With<ContextMenuNode>>,
    menu_items: Query<&Interaction, With<ContextMenuAction>>,
) {
    let left_clicked = mouse.just_pressed(MouseButton::Left);
    let right_clicked = mouse.just_pressed(MouseButton::Right);

    if !left_clicked && !right_clicked {
        return;
    }

    // On right-click only: keep open if cursor is over a menu item (a new menu
    // will be spawned for a different card by spawn_context_menu).
    // On left-click: always dismiss regardless.
    if right_clicked && !left_clicked {
        let over_item = menu_items
            .iter()
            .any(|i| *i == Interaction::Hovered || *i == Interaction::Pressed);
        if over_item {
            return;
        }
    }

    for entity in &menus {
        commands.entity(entity).despawn();
    }
}

/// Handles context menu item clicks — performs the action then closes the menu.
#[allow(clippy::type_complexity)]
#[allow(clippy::too_many_arguments)]
#[tracing::instrument(skip_all)]
pub fn handle_context_menu_actions(
    mut commands: Commands,
    items: Query<(&ContextMenuAction, &Interaction), (With<Button>, Changed<Interaction>)>,
    menus: Query<(Entity, &ContextMenuNode)>,
    mut scenario_list: ResMut<ScenarioList>,
    mut selected: ResMut<SelectedSenario>,
    mut next_state: ResMut<NextState<UiState>>,
) {
    for (action, interaction) in &items {
        if *interaction != Interaction::Pressed {
            continue;
        }

        // Get the scenario index from the first open menu.
        let scenario_index = menus.iter().next().map(|(_, m)| m.scenario_index);

        match action {
            ContextMenuAction::EditScenario => {
                if let Some(idx) = scenario_index {
                    selected.index = Some(idx);
                    next_state.set(UiState::Scenario);
                }
            }
            ContextMenuAction::Copy => {
                if let Some(idx) = scenario_index {
                    if let Some(entry) = scenario_list.entries.get(idx) {
                        match crate::core::scenario::Scenario::build(None) {
                            Ok(mut new_scenario) => {
                                new_scenario.config = entry.scenario.config.clone();
                                new_scenario.comment = format!(
                                    "Copy of {}",
                                    entry
                                        .scenario
                                        .comment
                                        .as_str()
                                        .trim()
                                        .trim_start_matches("Copy of ")
                                        .trim()
                                );
                                if let Err(e) = new_scenario.save() {
                                    tracing::warn!("Failed to save copied scenario: {e}");
                                }
                                scenario_list.entries.push(crate::ScenarioBundle {
                                    scenario: new_scenario,
                                    join_handle: None,
                                    epoch_rx: None,
                                    summary_rx: None,
                                });
                            }
                            Err(e) => tracing::warn!("Failed to create copy of scenario: {e}"),
                        }
                    }
                }
            }
            ContextMenuAction::Delete => {
                if let Some(idx) = scenario_index {
                    if idx < scenario_list.entries.len() {
                        let _ = scenario_list.entries[idx].scenario.delete();
                        scenario_list.entries.remove(idx);
                        if selected.index == Some(idx) {
                            selected.index = None;
                        }
                    }
                }
            }
            ContextMenuAction::Schedule => {
                if let Some(idx) = scenario_index {
                    if let Some(entry) = scenario_list.entries.get_mut(idx) {
                        let _ = entry.scenario.schedule();
                    }
                }
            }
            ContextMenuAction::Unschedule => {
                if let Some(idx) = scenario_index {
                    if let Some(entry) = scenario_list.entries.get_mut(idx) {
                        let _ = entry.scenario.unschedule();
                    }
                }
            }
            ContextMenuAction::OpenInResults => {
                if let Some(idx) = scenario_index {
                    selected.index = Some(idx);
                    next_state.set(UiState::Results);
                }
            }
            ContextMenuAction::OpenInVolumetric => {
                if let Some(idx) = scenario_index {
                    selected.index = Some(idx);
                    next_state.set(UiState::Volumetric);
                }
            }
        }

        // Dismiss the menu.
        for (entity, _) in &menus {
            commands.entity(entity).despawn();
        }
    }
}
