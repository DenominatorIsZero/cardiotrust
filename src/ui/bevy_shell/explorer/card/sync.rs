//! System that rebuilds scenario cards when the scenario list changes.

use bevy::prelude::*;

use super::{
    components::{CardEditMode, NewScenarioActionCard, ScenarioCard},
    spawn::{spawn_card, spawn_new_scenario_action_card},
};
use crate::{
    core::scenario::Status, ui::bevy_shell::explorer::thumbnail::ThumbnailCache, ScenarioList,
};

/// Rebuilds all scenario cards whenever `ScenarioList` or `ThumbnailCache` changes.
///
/// A full rebuild (despawn all, respawn all) is used instead of incremental diffing
/// because:
/// - Copies share the same ID as the original, so ID-based diffing misses them.
/// - Deletions shift indices, invalidating `ScenarioCard::index` on surviving cards.
/// - The list is never large enough for the overhead to matter.
///
/// Rebuilds are suppressed while a card is in inline-edit mode to avoid
/// interrupting the user mid-type.
#[allow(clippy::too_many_arguments)]
#[tracing::instrument(skip_all)]
pub fn sync_cards_to_scenarios(
    mut commands: Commands,
    scenario_list: Res<ScenarioList>,
    thumbnail_cache: Res<ThumbnailCache>,
    edit_mode: Res<CardEditMode>,
    grids: Query<Entity, With<crate::ui::bevy_shell::explorer::ExplorerGridNode>>,
    existing_cards: Query<Entity, With<ScenarioCard>>,
    action_cards: Query<Entity, With<NewScenarioActionCard>>,
) {
    if !scenario_list.is_changed() && !thumbnail_cache.is_changed() {
        return;
    }

    // Don't tear down cards while the user is typing a new name.
    if edit_mode.editing_index.is_some() {
        return;
    }

    let Ok(grid) = grids.single() else {
        return;
    };

    // Despawn every existing scenario card and the action card.
    for entity in &existing_cards {
        commands.entity(entity).despawn();
    }
    for entity in &action_cards {
        commands.entity(entity).despawn();
    }

    // Respawn all cards in index order.
    for (index, entry) in scenario_list.entries.iter().enumerate() {
        let scenario = &entry.scenario;
        let id = scenario.get_id().clone();

        let thumbnail = thumbnail_cache.states.get(&id);
        let metrics = if scenario.get_status() == &Status::Done {
            scenario.summary.as_ref().map(|s| (s.dice, s.loss))
        } else {
            None
        };
        let progress = if matches!(scenario.get_status(), Status::Running(_)) {
            Some(scenario.get_progress())
        } else {
            None
        };
        let etc_str = scenario.get_etc();
        let etc = if etc_str.is_empty() {
            None
        } else {
            Some(etc_str.as_str())
        };

        let ts = scenario
            .started
            .map(|dt| dt.format("%Y-%m-%d").to_string())
            .unwrap_or_default();
        let ts_opt = if ts.is_empty() {
            None
        } else {
            Some(ts.as_str())
        };

        let card_entity = spawn_card(
            &mut commands,
            &id,
            index,
            scenario.get_status(),
            &id,
            &scenario.comment,
            ts_opt,
            metrics,
            progress,
            etc,
            thumbnail,
        );
        commands.entity(grid).add_child(card_entity);
    }

    // Spawn the "New Scenario" action card at the end.
    let action = spawn_new_scenario_action_card(&mut commands);
    commands.entity(grid).add_child(action);
}
