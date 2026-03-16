//! Keyboard shortcuts for view navigation.
//!
//! Digit keys 1–6 navigate to views (subject to precondition guards).
//! Escape navigates to the logical parent view.

use bevy::prelude::*;

use crate::{core::scenario::Status, ui::UiState, ProjectState, ScenarioList, SelectedSenario};

/// Handles keyboard shortcuts for navigation.
///
/// * `1` → Home
/// * `2` → Explorer
/// * `3` → Scenario  (guard: scenario selected)
/// * `4` → Results   (guard: scenario Done)
/// * `5` → Volumetric (guard: scenario Done)
/// * `6` → Scheduler
/// * Escape → parent view
#[tracing::instrument(skip_all)]
pub fn handle_keyboard_shortcuts(
    keys: Res<ButtonInput<KeyCode>>,
    ui_state: Res<State<UiState>>,
    selected_scenario: Res<SelectedSenario>,
    scenario_list: Res<ScenarioList>,
    project_state: Res<ProjectState>,
    mut next_state: ResMut<NextState<UiState>>,
) {
    let has_project = project_state.current_path.is_some();
    let has_selection = selected_scenario.index.is_some();
    let scenario_done = selected_scenario.index.is_some_and(|i| {
        scenario_list
            .entries
            .get(i)
            .is_some_and(|e| e.scenario.get_status() == &Status::Done)
    });

    // Digit shortcuts
    let digit_map: [(KeyCode, UiState); 6] = [
        (KeyCode::Digit1, UiState::Home),
        (KeyCode::Digit2, UiState::Explorer),
        (KeyCode::Digit3, UiState::Scenario),
        (KeyCode::Digit4, UiState::Results),
        (KeyCode::Digit5, UiState::Volumetric),
        (KeyCode::Digit6, UiState::Scheduler),
    ];
    for (key, target) in digit_map {
        if keys.just_pressed(key) {
            // Shortcuts 2–6 require a project to be loaded.
            let project_ok = matches!(target, UiState::Home) || has_project;
            if project_ok && can_navigate_to(target, has_selection, scenario_done) {
                next_state.set(target);
            }
            return;
        }
    }

    // Escape → parent
    if keys.just_pressed(KeyCode::Escape) {
        let parent = parent_of(*ui_state.get());
        let project_ok = matches!(parent, UiState::Home) || has_project;
        if project_ok && can_navigate_to(parent, has_selection, scenario_done) {
            next_state.set(parent);
        }
    }
}

/// Returns the logical parent view (used for Escape key navigation).
#[tracing::instrument(level = "trace")]
fn parent_of(state: UiState) -> UiState {
    match state {
        UiState::Results | UiState::Volumetric => UiState::Scenario,
        UiState::Scenario => UiState::Explorer,
        _ => UiState::Home,
    }
}

/// Returns `true` if navigation to `target` is allowed.
#[tracing::instrument(level = "trace")]
fn can_navigate_to(target: UiState, has_selection: bool, scenario_done: bool) -> bool {
    match target {
        UiState::Scenario => has_selection,
        UiState::Results | UiState::Volumetric => scenario_done,
        _ => true,
    }
}
