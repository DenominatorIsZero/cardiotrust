//! Click and keyboard interaction handlers for scenario cards.

use bevy::{input::keyboard::KeyboardInput, prelude::*};

use super::components::{
    CardEditMode, CardQuickAction, CardQuickActionKind, LastCardClick, NewScenarioActionCard,
    ScenarioCard, DOUBLE_CLICK_SECS,
};
use crate::{core::scenario::Status, ScenarioList, SelectedSenario};

#[allow(clippy::type_complexity)]
/// Handles card clicks:
/// - First click → select the card.
/// - Click on already-selected card → enter inline name/comment edit mode.
/// - Double-click → navigate: Results view if Done, Scenario view otherwise.
///
/// Skips processing when a quick-action child button was pressed (those are
/// handled by `handle_card_quick_actions`).
#[tracing::instrument(skip_all)]
pub fn handle_card_click(
    cards: Query<(&ScenarioCard, &Interaction), (With<Button>, Changed<Interaction>)>,
    quick_actions: Query<&Interaction, (With<CardQuickAction>, With<Button>)>,
    mut selected: ResMut<SelectedSenario>,
    mut edit_mode: ResMut<CardEditMode>,
    mut last_click: ResMut<LastCardClick>,
    mut next_state: ResMut<NextState<crate::ui::UiState>>,
    scenario_list: Res<ScenarioList>,
    time: Res<Time>,
) {
    // If any quick-action button is currently pressed, don't also fire card logic.
    let quick_action_pressed = quick_actions.iter().any(|i| *i == Interaction::Pressed);

    for (card, interaction) in &cards {
        if *interaction != Interaction::Pressed {
            continue;
        }
        if quick_action_pressed {
            continue;
        }

        let now = time.elapsed_secs_f64();
        let is_double_click =
            last_click.index == Some(card.index) && (now - last_click.time) < DOUBLE_CLICK_SECS;

        // Update last-click tracking.
        last_click.time = now;
        last_click.index = Some(card.index);

        if is_double_click {
            // Double-click: navigate to the appropriate view.
            selected.index = Some(card.index);
            edit_mode.editing_index = None;
            edit_mode.draft = String::new();
            let is_done = scenario_list
                .entries
                .get(card.index)
                .is_some_and(|e| e.scenario.get_status() == &Status::Done);
            if is_done {
                next_state.set(crate::ui::UiState::Results);
            } else {
                next_state.set(crate::ui::UiState::Scenario);
            }
        } else if selected.index == Some(card.index) {
            // Second single-click on already-selected card → inline edit.
            let draft = scenario_list
                .entries
                .get(card.index)
                .map(|e| e.scenario.comment.clone())
                .unwrap_or_default();
            edit_mode.editing_index = Some(card.index);
            edit_mode.draft = draft;
        } else {
            // First click → select; cancel any in-progress edit.
            selected.index = Some(card.index);
            if edit_mode.editing_index.is_some() {
                edit_mode.editing_index = None;
                edit_mode.draft = String::new();
            }
        }
    }
}

/// Handles quick-action button presses ([D] delete, [C] copy, [S] schedule).
#[allow(clippy::type_complexity)]
#[tracing::instrument(skip_all)]
pub fn handle_card_quick_actions(
    quick_actions: Query<(&CardQuickAction, &Interaction), (With<Button>, Changed<Interaction>)>,
    mut scenario_list: ResMut<ScenarioList>,
    mut selected: ResMut<SelectedSenario>,
) {
    for (action, interaction) in &quick_actions {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let idx = action.index;
        match action.kind {
            CardQuickActionKind::Delete => {
                if idx < scenario_list.entries.len() {
                    let _ = scenario_list.entries[idx].delete();
                    scenario_list.entries.remove(idx);
                    if selected.index == Some(idx) {
                        selected.index = None;
                    }
                }
            }
            CardQuickActionKind::Copy => {
                if let Some(entry) = scenario_list.entries.get(idx) {
                    match entry.copy_as_planning() {
                        Ok(mut copied_bundle) => {
                            copied_bundle.scenario.config = entry.scenario.config.clone();
                            copied_bundle.scenario.comment = format!(
                                "Copy of {}",
                                entry
                                    .scenario
                                    .comment
                                    .as_str()
                                    .trim()
                                    .to_string()
                                    .trim_start_matches("Copy of ")
                                    .trim()
                            );
                            if let Err(e) = copied_bundle.save_metadata() {
                                tracing::warn!("Failed to save copied scenario: {e}");
                            }
                            scenario_list.entries.push(copied_bundle);
                        }
                        Err(e) => tracing::warn!("Failed to create copy of scenario: {e}"),
                    }
                }
            }
            CardQuickActionKind::Schedule => {
                if let Some(entry) = scenario_list.entries.get_mut(idx) {
                    let _ = entry.scenario.schedule();
                }
            }
        }
    }
}

/// Captures keyboard input when a card is in inline-edit mode and updates the
/// scenario comment. Enter/Escape commit or cancel.
#[tracing::instrument(skip_all)]
pub fn handle_card_inline_edit(
    mut edit_mode: ResMut<CardEditMode>,
    mut scenario_list: ResMut<ScenarioList>,
    mut keyboard: MessageReader<KeyboardInput>,
    search_focused: Res<super::super::toolbar::SearchFocused>,
) {
    // Don't consume keyboard events when search field is focused.
    if search_focused.0 {
        keyboard.clear();
        return;
    }

    let Some(editing_index) = edit_mode.editing_index else {
        keyboard.clear();
        return;
    };

    for event in keyboard.read() {
        if event.state != bevy::input::ButtonState::Pressed {
            continue;
        }
        match event.key_code {
            KeyCode::Enter | KeyCode::NumpadEnter => {
                // Commit the edit and persist to disk.
                if let Some(entry) = scenario_list.entries.get_mut(editing_index) {
                    entry.scenario.comment.clone_from(&edit_mode.draft);
                    if let Err(e) = entry.save_metadata() {
                        tracing::warn!("Failed to save scenario after comment edit: {e}");
                    }
                }
                edit_mode.editing_index = None;
                edit_mode.draft = String::new();
            }
            KeyCode::Escape => {
                // Cancel without saving.
                edit_mode.editing_index = None;
                edit_mode.draft = String::new();
            }
            KeyCode::Backspace => {
                edit_mode.draft.pop();
            }
            _ => {
                // Append printable characters.
                if let Some(text) = &event.text {
                    for ch in text.chars() {
                        if !ch.is_control() {
                            edit_mode.draft.push(ch);
                        }
                    }
                }
            }
        }
    }
}

#[allow(clippy::type_complexity)]
/// Handles "New Scenario" action card click: creates a new Planning scenario,
/// sets it as active, and syncs the card list (grid stays visible).
#[tracing::instrument(skip_all)]
pub fn handle_new_scenario_card_click(
    action_cards: Query<
        &Interaction,
        (
            With<NewScenarioActionCard>,
            With<Button>,
            Changed<Interaction>,
        ),
    >,
    mut scenario_list: ResMut<ScenarioList>,
    mut selected: ResMut<SelectedSenario>,
    project_state: Res<crate::ProjectState>,
) {
    for interaction in &action_cards {
        if *interaction == Interaction::Pressed {
            create_new_scenario(&mut scenario_list, &mut selected, &project_state);
        }
    }
}

/// Creates a new Planning scenario and appends it to the list.
#[tracing::instrument(skip_all)]
pub fn create_new_scenario(
    scenario_list: &mut ScenarioList,
    selected: &mut SelectedSenario,
    _project_state: &crate::ProjectState,
) {
    let Some(project_root) = scenario_list.project_root.clone() else {
        tracing::warn!("Cannot create a scenario without an active project");
        return;
    };
    let storage = crate::core::scenario::ScenarioStorage::new(project_root);
    match crate::ScenarioBundle::create(storage, crate::core::scenario::Scenario::build(None)) {
        Ok(bundle) => {
            let index = scenario_list.entries.len();
            scenario_list.entries.push(bundle);
            selected.index = Some(index);
        }
        Err(e) => {
            tracing::warn!("Failed to create new scenario: {}", e);
        }
    }
}
