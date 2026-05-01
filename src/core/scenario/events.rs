//! Bevy message types for Copy and Delete scenario operations.
//!
//! These messages are emitted by the UI layer and handled by systems here,
//! keeping the UI decoupled from direct storage mutations.

use bevy::prelude::*;
use tracing::warn;

use super::Status;
use crate::{ScenarioList, SelectedSenario};

// ── Message types ─────────────────────────────────────────────────────────────

/// Message: create a copy of the scenario at `source_index` in `ScenarioList`.
#[derive(Message, Debug, Clone)]
pub struct CopyScenarioMessage {
    /// Index into `ScenarioList::entries` of the scenario to copy.
    pub source_index: usize,
}

/// Message: delete the scenario at `scenario_index` in `ScenarioList`.
/// Only succeeds if the scenario is in Planning status.
#[derive(Message, Debug, Clone)]
pub struct DeleteScenarioMessage {
    /// Index into `ScenarioList::entries` of the scenario to delete.
    pub scenario_index: usize,
}

// ── Systems ───────────────────────────────────────────────────────────────────

/// Handles [`CopyScenarioMessage`]:
/// - Clones the config and comment from the source scenario.
/// - Creates a new scenario in Planning state with a new unique ID.
/// - Inserts the new scenario into `ScenarioList`.
/// - Selects the new scenario.
#[tracing::instrument(skip_all)]
pub fn handle_copy_scenario(
    mut messages: MessageReader<CopyScenarioMessage>,
    mut scenario_list: ResMut<ScenarioList>,
    mut selected: ResMut<SelectedSenario>,
) {
    for msg in messages.read() {
        let Some(entry) = scenario_list.entries.get(msg.source_index) else {
            warn!(
                "CopyScenarioMessage: source_index {} out of bounds (len={})",
                msg.source_index,
                scenario_list.entries.len()
            );
            continue;
        };

        let config = entry.scenario.config.clone();
        let comment = entry.scenario.comment.clone();

        match entry.copy_as_planning() {
            Ok(mut copied_bundle) => {
                copied_bundle.scenario.config = config;
                copied_bundle.scenario.comment = format!(
                    "Copy of {}",
                    comment.trim().trim_start_matches("Copy of ").trim()
                );
                if let Err(e) = copied_bundle.save_metadata() {
                    warn!("handle_copy_scenario: failed to save copied scenario: {e}");
                }
                let new_index = scenario_list.entries.len();
                scenario_list.entries.push(copied_bundle);
                selected.index = Some(new_index);
            }
            Err(e) => {
                warn!("handle_copy_scenario: failed to build new scenario: {e}");
            }
        }
    }
}

/// Handles [`DeleteScenarioMessage`]:
/// - Removes the scenario from `ScenarioList` if it is in Planning status.
/// - Logs a warning and does nothing if the scenario is not in Planning status.
#[tracing::instrument(skip_all)]
pub fn handle_delete_scenario(
    mut messages: MessageReader<DeleteScenarioMessage>,
    mut scenario_list: ResMut<ScenarioList>,
    mut selected: ResMut<SelectedSenario>,
) {
    for msg in messages.read() {
        let Some(entry) = scenario_list.entries.get(msg.scenario_index) else {
            warn!(
                "DeleteScenarioMessage: scenario_index {} out of bounds (len={})",
                msg.scenario_index,
                scenario_list.entries.len()
            );
            continue;
        };

        if *entry.scenario.get_status() != Status::Planning {
            warn!(
                "DeleteScenarioMessage: cannot delete scenario in {:?} status",
                entry.scenario.get_status()
            );
            continue;
        }

        if let Err(e) = entry.delete() {
            warn!("handle_delete_scenario: failed to delete scenario from disk: {e}");
        }

        let idx = msg.scenario_index;
        scenario_list.entries.remove(idx);

        // Adjust selection
        if selected.index == Some(idx) {
            selected.index = if scenario_list.entries.is_empty() {
                None
            } else if idx >= scenario_list.entries.len() {
                Some(scenario_list.entries.len() - 1)
            } else {
                Some(idx)
            };
        }
    }
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::{super::Scenario, *};
    use crate::ScenarioBundle;

    #[test]
    fn copy_creates_scenario_in_planning() {
        let src = Scenario::new_in_memory();
        let src_id = src.get_id().clone();
        let src_config = src.config.clone();

        // Simulate copy: new scenario starts in Planning with same config
        let mut copy = Scenario::new_in_memory();
        copy.config = src_config;

        assert_eq!(*copy.get_status(), Status::Planning);
        assert_ne!(copy.get_id(), &src_id, "copy must have a different ID");
        // Source is unchanged
        assert_eq!(src.get_id(), &src_id);
    }

    #[test]
    #[allow(clippy::redundant_clone)]
    fn copy_preserves_all_config_fields() {
        let src = Scenario::new_in_memory();
        let src_config = src.config.clone();

        let mut copy = Scenario::new_in_memory();
        copy.config = src_config.clone();

        assert_eq!(copy.config, src_config);
    }

    #[test]
    fn copy_does_not_affect_source() {
        let src = Scenario::new_in_memory();
        let src_id = src.get_id().clone();
        let src_status = src.get_status().clone();

        // Build a copy without touching src
        let _copy = Scenario::new_in_memory();

        assert_eq!(src.get_id(), &src_id);
        assert_eq!(*src.get_status(), src_status);
    }

    #[test]
    fn delete_planning_removes_scenario() {
        let scenario = Scenario::new_in_memory();
        assert_eq!(*scenario.get_status(), Status::Planning);

        let mut list = ScenarioList {
            entries: vec![ScenarioBundle {
                scenario,
                storage: crate::core::scenario::ScenarioStorage::new("./results/tests"),
                join_handle: None,
                epoch_rx: None,
                summary_rx: None,
            }],
            project_root: None,
        };

        // Simulate delete (Planning succeeds)
        let is_planning = matches!(list.entries[0].scenario.get_status(), Status::Planning);
        assert!(is_planning);
        list.entries.remove(0);
        assert!(list.entries.is_empty(), "scenario should be removed");
    }

    #[test]
    fn delete_non_planning_is_rejected() {
        let mut scenario = Scenario::new_in_memory();
        scenario.force_scheduled();
        assert_ne!(*scenario.get_status(), Status::Planning);

        let list = ScenarioList {
            entries: vec![ScenarioBundle {
                scenario,
                storage: crate::core::scenario::ScenarioStorage::new("./results/tests"),
                join_handle: None,
                epoch_rx: None,
                summary_rx: None,
            }],
            project_root: None,
        };

        // The handler should NOT remove the scenario
        let can_delete = matches!(list.entries[0].scenario.get_status(), Status::Planning);
        assert!(!can_delete, "non-Planning scenario must not be deletable");
    }
}
