//! Project loading system — watches `ProjectState` for path changes and
//! reloads `ScenarioList` accordingly.

use bevy::prelude::*;
use tracing::warn;

use crate::{ui::UiState, ProjectState, ScenarioList, SelectedSenario};

/// Watches for changes to [`ProjectState::current_path`]. When a new path is
/// set, loads the corresponding [`ScenarioList`], resets [`SelectedSenario`],
/// and transitions to [`UiState::Explorer`].
///
/// On load errors a warning is logged and the app stays on the Home view.
#[tracing::instrument(skip_all)]
pub fn load_project_on_path_change(
    mut commands: Commands,
    project_state: Res<ProjectState>,
    mut next_state: ResMut<NextState<UiState>>,
    mut selected_scenario: ResMut<SelectedSenario>,
) {
    if !project_state.is_changed() {
        return;
    }
    if let Some(path) = &project_state.current_path {
        match ScenarioList::load_from(path) {
            Ok(scenario_list) => {
                selected_scenario.index = None;
                commands.insert_resource(scenario_list);
                next_state.set(UiState::Explorer);
            }
            Err(e) => {
                warn!("Failed to load project from {}: {}", path.display(), e);
            }
        }
    }
}
