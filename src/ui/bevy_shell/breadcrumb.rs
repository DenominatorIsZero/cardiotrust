//! Breadcrumb context bar — updates when `UiState` transitions.

use bevy::prelude::*;

use crate::{
    ui::{colors, UiState},
    ScenarioList, SelectedSenario,
};

/// Marker component on the text node inside the breadcrumb bar.
#[derive(Component, Debug)]
pub struct BreadcrumbText;

/// Returns the base breadcrumb path for a given [`UiState`], without any
/// scenario-specific suffix.
#[tracing::instrument(level = "trace")]
fn base_path(state: UiState) -> &'static str {
    match state {
        UiState::Home => "Home",
        UiState::Explorer => "Home > Explorer",
        UiState::Scenario => "Home > Explorer > Scenario",
        UiState::Results => "Home > Explorer > Scenario > Results",
        UiState::Volumetric => "Home > Explorer > Scenario > Volumetric",
        UiState::Scheduler => "Home > Scheduler",
    }
}

/// Returns `true` for views that live "under" a specific scenario so that the
/// scenario ID should appear in the breadcrumb path.
#[tracing::instrument(level = "trace")]
fn shows_scenario_id(state: UiState) -> bool {
    matches!(
        state,
        UiState::Scenario | UiState::Results | UiState::Volumetric
    )
}

/// Builds the full breadcrumb string for the given state, appending the
/// selected scenario ID when appropriate.
#[tracing::instrument(skip_all)]
fn build_breadcrumb(
    state: UiState,
    selected_scenario: &SelectedSenario,
    scenario_list: &ScenarioList,
) -> String {
    let base = base_path(state);
    if shows_scenario_id(state) {
        if let Some(id) = selected_scenario.index.and_then(|i| {
            scenario_list
                .entries
                .get(i)
                .map(|e| e.scenario.get_id().clone())
        }) {
            // Insert the scenario ID after "Explorer" in the path.
            // e.g. "Home > Explorer > Scenario" → "Home > Explorer > <id> > Scenario"
            return base.replacen("Explorer > ", &format!("Explorer > {id} > "), 1);
        }
    }
    base.to_string()
}

/// Updates the breadcrumb text whenever `UiState` transitions.
#[tracing::instrument(skip_all)]
pub fn update_breadcrumb(
    mut transitions: EventReader<StateTransitionEvent<UiState>>,
    mut texts: Query<(&mut Text, &mut TextColor), With<BreadcrumbText>>,
    selected_scenario: Res<SelectedSenario>,
    scenario_list: Res<ScenarioList>,
) {
    for event in transitions.read() {
        if let Some(entered) = event.entered {
            let crumb = build_breadcrumb(entered, &selected_scenario, &scenario_list);
            for (mut text, mut color) in &mut texts {
                text.0.clone_from(&crumb);
                color.0 = colors::GREY1;
            }
        }
    }
}
