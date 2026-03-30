use bevy::prelude::*;

use super::{
    super::{card::NewScenarioActionCard, ExplorerGridNode},
    SearchQuery, SortOrder, SortOrderButton, StatusFilter, StatusFilterButton,
};
use crate::ScenarioList;

#[allow(clippy::type_complexity)]
/// Writes the selected filter to `StatusFilter` resource.
#[tracing::instrument(skip_all)]
pub fn handle_status_filter_click(
    buttons: Query<(&StatusFilterButton, &Interaction), (With<Button>, Changed<Interaction>)>,
    mut filter: ResMut<StatusFilter>,
) {
    for (btn, interaction) in &buttons {
        if *interaction == Interaction::Pressed {
            *filter = btn.0;
        }
    }
}

#[allow(clippy::type_complexity)]
/// Writes the selected sort order to `SortOrder` resource.
#[tracing::instrument(skip_all)]
pub fn handle_sort_click(
    buttons: Query<(&SortOrderButton, &Interaction), (With<Button>, Changed<Interaction>)>,
    mut order: ResMut<SortOrder>,
) {
    for (btn, interaction) in &buttons {
        if *interaction == Interaction::Pressed {
            *order = btn.0;
        }
    }
}

/// Updates filter and sort pill button backgrounds to show which one is active.
#[allow(clippy::type_complexity)]
#[tracing::instrument(skip_all)]
pub fn update_toolbar_button_visuals(
    filter: Res<StatusFilter>,
    order: Res<SortOrder>,
    mut filter_buttons: Query<
        (
            Entity,
            &StatusFilterButton,
            &Interaction,
            &mut BackgroundColor,
        ),
        (With<Button>, Without<SortOrderButton>),
    >,
    mut sort_buttons: Query<
        (Entity, &SortOrderButton, &Interaction, &mut BackgroundColor),
        (With<Button>, Without<StatusFilterButton>),
    >,
    mut text_colors: Query<(&ChildOf, &mut TextColor)>,
) {
    // Collect active-state per entity so we can update child text colours.
    let mut active_entities: std::collections::HashSet<Entity> = std::collections::HashSet::new();

    for (entity, btn, interaction, mut bg) in &mut filter_buttons {
        let is_active = btn.0 == *filter;
        if is_active {
            active_entities.insert(entity);
        }
        bg.0 = if is_active {
            crate::ui::colors::ORANGE
        } else {
            match interaction {
                Interaction::Hovered => crate::ui::colors::BG2,
                _ => crate::ui::colors::BG3,
            }
        };
    }

    for (entity, btn, interaction, mut bg) in &mut sort_buttons {
        let is_active = btn.0 == *order;
        if is_active {
            active_entities.insert(entity);
        }
        bg.0 = if is_active {
            crate::ui::colors::ORANGE
        } else {
            match interaction {
                Interaction::Hovered => crate::ui::colors::BG2,
                _ => crate::ui::colors::BG3,
            }
        };
    }

    // Update text colour for children of all filter/sort buttons.
    let all_button_entities: std::collections::HashSet<Entity> = filter_buttons
        .iter()
        .map(|(e, _, _, _)| e)
        .chain(sort_buttons.iter().map(|(e, _, _, _)| e))
        .collect();

    for (child_of, mut text_color) in &mut text_colors {
        let parent = child_of.parent();
        if !all_button_entities.contains(&parent) {
            continue;
        }
        text_color.0 = if active_entities.contains(&parent) {
            crate::ui::colors::BG0
        } else {
            crate::ui::colors::FG1
        };
    }
}

/// Shows/hides cards based on active filter + search query, then reorders
/// visible cards according to `SortOrder`.
///
/// Toggles `Display` on the card's `Node` (not `Visibility`), so hidden cards
/// do not occupy grid space. Also updates `CardMatchHighlight` on label entities.
/// After filtering, sorts the visible entity list and calls
/// `replace_children` on the grid parent to apply the new order.
#[allow(clippy::too_many_arguments)]
#[tracing::instrument(skip_all)]
pub fn apply_filter_and_sort(
    mut commands: Commands,
    filter: Res<StatusFilter>,
    order: Res<SortOrder>,
    search: Res<SearchQuery>,
    scenario_list: Res<ScenarioList>,
    grids: Query<Entity, With<ExplorerGridNode>>,
    action_cards: Query<Entity, With<NewScenarioActionCard>>,
    mut cards: Query<(Entity, &super::super::card::ScenarioCard, &mut Node)>,
    card_ids: Query<(Entity, &super::super::card::CardScenarioId)>,
    mut name_labels: Query<
        (
            &super::super::card::CardNameLabel,
            &mut super::super::card::CardMatchHighlight,
        ),
        Without<super::super::card::CardIdLabel>,
    >,
    mut id_labels: Query<
        (
            &super::super::card::CardIdLabel,
            &mut super::super::card::CardMatchHighlight,
        ),
        Without<super::super::card::CardNameLabel>,
    >,
) {
    if !filter.is_changed()
        && !order.is_changed()
        && !search.is_changed()
        && !scenario_list.is_changed()
    {
        return;
    }

    let query_lower = search.0.to_lowercase();

    // Build entity → scenario-id map from the stable CardScenarioId component.
    let entity_to_sid: std::collections::HashMap<Entity, String> =
        card_ids.iter().map(|(e, cid)| (e, cid.0.clone())).collect();

    // Track which entities are visible after filtering.
    let mut visible_entities: Vec<Entity> = Vec::new();

    for (entity, card, mut node) in &mut cards {
        let Some(entry) = scenario_list.entries.get(card.index) else {
            continue;
        };
        let scenario = &entry.scenario;

        // Status filter
        let status_ok = match *filter {
            StatusFilter::All => true,
            StatusFilter::Planning => {
                scenario.get_status() == &crate::core::scenario::Status::Planning
            }
            StatusFilter::Queued => {
                scenario.get_status() == &crate::core::scenario::Status::Scheduled
            }
            StatusFilter::Running => matches!(
                scenario.get_status(),
                crate::core::scenario::Status::Running(_)
            ),
            StatusFilter::Done => scenario.get_status() == &crate::core::scenario::Status::Done,
            StatusFilter::Failed => {
                scenario.get_status() == &crate::core::scenario::Status::Aborted
            }
        };

        // Fuzzy search filter
        let (search_ok, name_range, id_range) = if query_lower.is_empty() {
            (true, None, None)
        } else {
            let comment = &scenario.comment;
            let id = scenario.get_id();
            let display_name = if comment.is_empty() {
                id.as_str()
            } else {
                comment.as_str()
            };
            let name_match = super::search::fuzzy_match(&query_lower, &display_name.to_lowercase());
            let id_match = super::search::fuzzy_match(&query_lower, &id.to_lowercase());
            let ok = name_match.is_some() || id_match.is_some();
            (ok, name_match, id_match)
        };

        node.display = if status_ok && search_ok {
            Display::Flex
        } else {
            Display::None
        };

        if status_ok && search_ok {
            visible_entities.push(entity);
        }

        // Update CardMatchHighlight on the label entities (matched by scenario index).
        for (label, mut highlight) in &mut name_labels {
            if label.index == card.index {
                highlight.name_range = name_range;
                highlight.id_range = id_range;
            }
        }
        for (label, mut highlight) in &mut id_labels {
            if label.index == card.index {
                highlight.name_range = name_range;
                highlight.id_range = id_range;
            }
        }
    }

    // ── Sort visible entities by the active SortOrder ─────────────────────────
    //
    // For each entity we look up its scenario via CardScenarioId → ScenarioList.
    // Scenarios missing the sort key (e.g. no summary) are sorted to the end.
    // A stable tiebreaker on scenario ID ensures a deterministic order.

    /// Look up the scenario for a card entity by ID, returning `None` if not found.
    #[tracing::instrument(skip_all)]
    fn find_scenario<'a>(
        entity: Entity,
        entity_to_sid: &std::collections::HashMap<Entity, String>,
        scenario_list: &'a ScenarioList,
    ) -> Option<&'a crate::core::scenario::Scenario> {
        let sid = entity_to_sid.get(&entity)?;
        scenario_list
            .entries
            .iter()
            .find(|e| e.scenario.get_id() == sid)
            .map(|e| &e.scenario)
    }

    visible_entities.sort_by(|&a, &b| {
        let sa = find_scenario(a, &entity_to_sid, &scenario_list);
        let sb = find_scenario(b, &entity_to_sid, &scenario_list);

        match *order {
            SortOrder::DateNewest => {
                // Most-recent first; None timestamps sort last.
                let ta = sa.and_then(|s| s.started);
                let tb = sb.and_then(|s| s.started);
                match (ta, tb) {
                    (Some(a_dt), Some(b_dt)) => b_dt.cmp(&a_dt).then_with(|| tiebreak_id(sa, sb)),
                    (Some(_), None) => std::cmp::Ordering::Less,
                    (None, Some(_)) => std::cmp::Ordering::Greater,
                    (None, None) => tiebreak_id(sa, sb),
                }
            }
            SortOrder::LossLowest => {
                // Lowest loss first; None summary → f32::MAX sorts last.
                let la = sa
                    .and_then(|s| s.summary.as_ref())
                    .map_or(f32::MAX, |s| s.loss);
                let lb = sb
                    .and_then(|s| s.summary.as_ref())
                    .map_or(f32::MAX, |s| s.loss);
                la.total_cmp(&lb).then_with(|| tiebreak_id(sa, sb))
            }
            SortOrder::DiceHighest => {
                // Highest dice first; None summary → f32::MIN sorts last.
                let da = sa
                    .and_then(|s| s.summary.as_ref())
                    .map_or(f32::MIN, |s| s.dice);
                let db = sb
                    .and_then(|s| s.summary.as_ref())
                    .map_or(f32::MIN, |s| s.dice);
                // Descending: compare b vs a.
                db.total_cmp(&da).then_with(|| tiebreak_id(sa, sb))
            }
            SortOrder::Name => {
                // Ascending by display name (comment if set, else id), case-insensitive.
                let na = sa.map_or_else(String::default, |s| {
                    if s.comment.is_empty() {
                        s.get_id().to_lowercase()
                    } else {
                        s.comment.to_lowercase()
                    }
                });
                let nb = sb.map_or_else(String::default, |s| {
                    if s.comment.is_empty() {
                        s.get_id().to_lowercase()
                    } else {
                        s.comment.to_lowercase()
                    }
                });
                na.cmp(&nb).then_with(|| tiebreak_id(sa, sb))
            }
        }
    });

    // Apply the sorted order to the grid parent's children list.
    // The "New Scenario" action card is always placed last; it was never part
    // of the sort input and must be explicitly re-appended so replace_children
    // doesn't detach it from the grid.
    let mut ordered: Vec<Entity> = visible_entities;
    if let Ok(action) = action_cards.single() {
        ordered.push(action);
    }
    if let Ok(grid) = grids.single() {
        commands.entity(grid).replace_children(&ordered);
    }
}

/// Stable tiebreaker: ascending by scenario ID (empty/missing IDs sort last).
#[tracing::instrument(skip_all)]
fn tiebreak_id(
    a: Option<&crate::core::scenario::Scenario>,
    b: Option<&crate::core::scenario::Scenario>,
) -> std::cmp::Ordering {
    let id_a = a.map_or("", |s| s.get_id().as_str());
    let id_b = b.map_or("", |s| s.get_id().as_str());
    id_a.cmp(id_b)
}
