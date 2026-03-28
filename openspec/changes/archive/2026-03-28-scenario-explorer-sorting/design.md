## Context

The scenario explorer toolbar in `src/ui/bevy_shell/explorer/toolbar.rs` already defines a `SortOrder` resource (enum with `DateNewest`, `LossLowest`, `DiceHighest`, `Name` variants), spawns four pill buttons wired to `handle_sort_click`, and visually highlights the active button. The `apply_filter_and_sort` system reads `SortOrder` but never acts on it — the reordering block was deferred with a TODO at line 684.

Card entities are children of a shared grid/scroll container in the explorer scene. The system `sync_cards_to_scenarios` spawns or despawns cards as the scenario list changes; it reads `ScenarioList` (a Bevy `Resource`) where each `ScenarioBundle` holds a reference to a `Scenario`.

## Goals / Non-Goals

**Goals:**
- Clicking a sort button reorders card entities in the DOM so the grid visually reflects the chosen order.
- All four orderings work: DateNewest (by `scenario.started` desc), LossLowest (by `summary.loss` asc), DiceHighest (by `summary.dice` desc), Name (display name asc, case-insensitive).
- Scenarios missing the required field for the selected order sort to the end.

**Non-Goals:**
- Persisting the chosen sort order across app restarts.
- Adding new sort keys beyond the four already exposed in the UI.
- Changing how cards are spawned or the card layout.

## Decisions

### D1 — Sort by reordering Bevy children, not re-spawning cards

**Decision:** In `apply_filter_and_sort`, after computing the desired entity order, apply it by calling `world.entity_mut(parent).insert_children(index, &[entity])` (or equivalent Bevy 0.15 API) to reorder the existing child list, rather than despawning and re-spawning cards.

**Rationale:** Re-spawning is expensive (animations, textures re-upload) and causes visual flicker. Bevy's `Children` component is an ordered `SmallVec`; mutating child order is cheap and is the intended API for DOM-reorder effects.

**Alternatives considered:** Sorting `ScenarioList.entries` on each sort change and letting `sync_cards_to_scenarios` respawn — rejected because `sync_cards_to_scenarios` uses ID-based diffing and would despawn+spawn all cards on every sort change.

### D2 — Map card entity → scenario data via an ECS component

**Decision:** Attach a `CardScenarioId(String)` component to each card root entity at spawn time (in `spawn_card`). `apply_filter_and_sort` queries `Query<(Entity, &CardScenarioId)>` to build the entity→scenario-id mapping, then looks up each scenario in `ScenarioList` for sort-key extraction.

**Rationale:** Avoids positional index fragility; the component is a stable, cheap label. The entity→ID mapping survives partial despawns from filter changes.

**Alternatives considered:** Using the entity's position in `Children` to index into `ScenarioList.entries` — fragile once filtering removes some cards. Storing a direct `&Scenario` pointer — not possible across frames in Bevy.

### D3 — Run sort in `apply_filter_and_sort` on `SortOrder` change only

**Decision:** Gate the reorder pass with a `Res<SortOrder>` change detection check (`order.is_changed()`). Also re-sort whenever `ScenarioList` changes (new scenario arrives or a scenario finishes and gains a summary).

**Rationale:** Avoids unnecessary child-list mutations every frame; child reordering still triggers Bevy's change propagation.

### D4 — Stable secondary sort on scenario ID

**Decision:** After applying the primary sort key, use `scenario.get_id()` as a stable tiebreaker ascending. This makes the order deterministic for equal-metric scenarios.

## Risks / Trade-offs

- [Risk] Bevy's `insert_children` / `reorder_children` API surface has shifted between minor versions; the exact call must match the workspace's `bevy` version. → **Mitigation**: Check `Cargo.toml` for the pinned Bevy version before implementation; use `commands.entity(parent).replace_children(&ordered_entities)` which is stable across 0.14–0.15.
- [Risk] `apply_filter_and_sort` currently also applies the status filter. Reordering must happen after filtering so hidden cards are not included in the child order computation. → **Mitigation**: Collect visible entities first, sort them, then call `replace_children` with only the visible set (already the intended behavior for the filter).
- [Risk] Scenarios with `summary = None` need a defined position for `LossLowest`/`DiceHighest`. → **Mitigation**: Use `f32::MAX` / `f32::MIN` sentinel so they sort to the end; document this in code comments.
