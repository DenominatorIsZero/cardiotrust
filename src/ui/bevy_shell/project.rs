//! Project loading system — watches pending project load requests and
//! replaces `ScenarioList` accordingly.

use bevy::prelude::*;
use tracing::warn;

use crate::{
    core::scenario::Status, ui::UiState, PendingProjectLoad, ScenarioList, SelectedSenario,
};

pub const PROJECT_SWITCH_BLOCKED_MESSAGE: &str =
    "Cannot switch projects while a scenario is simulating or running.";

#[tracing::instrument(level = "trace", skip_all)]
pub(crate) fn is_project_switch_blocked(scenario_list: &ScenarioList) -> bool {
    scenario_list.entries.iter().any(|entry| {
        matches!(
            entry.scenario.get_status(),
            Status::Simulating | Status::Running(_)
        )
    })
}

/// Watches for changes to [`PendingProjectLoad`]. When a new path is
/// set, loads the corresponding [`ScenarioList`], resets [`SelectedSenario`],
/// and transitions to [`UiState::Explorer`].
///
/// On load errors a warning is logged and the app stays on the Home view.
#[tracing::instrument(skip_all)]
pub fn load_project_on_path_change(
    mut commands: Commands,
    mut pending_project_load: ResMut<PendingProjectLoad>,
    scenario_list: Res<ScenarioList>,
    mut next_state: ResMut<NextState<UiState>>,
    mut selected_scenario: ResMut<SelectedSenario>,
) {
    if let Some(path) = pending_project_load.0.take() {
        if is_project_switch_blocked(&scenario_list) {
            warn!(
                "Blocked project switch to {} because a scenario is still active",
                path.display()
            );
            return;
        }

        match ScenarioList::load_from(&path) {
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

#[cfg(test)]
mod tests {
    use std::{fs, path::Path};

    use super::*;
    use crate::{
        core::scenario::{Scenario, ScenarioStorage},
        ScenarioBundle,
    };

    fn test_project_dir(name: &str) -> std::path::PathBuf {
        Path::new("./results").join(name)
    }

    #[test]
    fn switching_is_blocked_only_for_running_or_simulating() {
        let mut scenario_list = ScenarioList::empty();

        let mut running = Scenario::build(Some("running".to_string()));
        running.set_running(3);
        scenario_list.entries.push(ScenarioBundle::new(
            running,
            ScenarioStorage::new("./results/tests"),
        ));
        assert!(is_project_switch_blocked(&scenario_list));

        scenario_list.entries.clear();
        let mut simulating = Scenario::build(Some("simulating".to_string()));
        simulating.set_simulating();
        scenario_list.entries.push(ScenarioBundle::new(
            simulating,
            ScenarioStorage::new("./results/tests"),
        ));
        assert!(is_project_switch_blocked(&scenario_list));

        scenario_list.entries.clear();
        let mut scheduled = Scenario::build(Some("scheduled".to_string()));
        scheduled.force_scheduled();
        scenario_list.entries.push(ScenarioBundle::new(
            scheduled,
            ScenarioStorage::new("./results/tests"),
        ));
        assert!(!is_project_switch_blocked(&scenario_list));
    }

    #[test]
    fn blocked_load_does_not_replace_current_project() -> anyhow::Result<()> {
        let current_dir = test_project_dir("switch-guard-current-project");
        let pending_dir = test_project_dir("switch-guard-pending-project");
        for dir in [&current_dir, &pending_dir] {
            if dir.is_dir() {
                fs::remove_dir_all(dir)?;
            }
        }

        let current_storage = ScenarioStorage::new(current_dir.clone());
        let pending_storage = ScenarioStorage::new(pending_dir.clone());

        let mut current = Scenario::build(Some("active".to_string()));
        current.set_running(9);
        ScenarioBundle::create(current_storage, current)?;
        ScenarioBundle::create(
            pending_storage,
            Scenario::build(Some("new-project-scenario".to_string())),
        )?;

        let mut app = App::new();
        app.add_plugins(bevy::state::app::StatesPlugin);
        app.init_state::<UiState>();
        app.insert_resource(ScenarioList::load_from(&current_dir)?);
        app.insert_resource(PendingProjectLoad(Some(pending_dir.clone())));
        app.insert_resource(SelectedSenario { index: Some(0) });
        app.add_systems(Update, load_project_on_path_change);

        app.update();
        app.world_mut().flush();

        let scenario_list = app.world().resource::<ScenarioList>();
        assert_eq!(
            scenario_list.project_root.as_deref(),
            Some(current_dir.as_path())
        );
        assert_eq!(scenario_list.entries.len(), 1);
        assert_eq!(
            scenario_list.entries[0].scenario.get_id(),
            &"active".to_string()
        );
        assert!(app.world().resource::<PendingProjectLoad>().0.is_none());
        assert_eq!(app.world().resource::<SelectedSenario>().index, Some(0));

        for dir in [&current_dir, &pending_dir] {
            if dir.is_dir() {
                fs::remove_dir_all(dir)?;
            }
        }
        Ok(())
    }
}
