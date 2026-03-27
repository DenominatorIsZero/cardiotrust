## 1. Module scaffolding

- [x] 1.1 Create `src/ui/bevy_ui/explorer/mod.rs` with `ExplorerViewPlugin` struct and `Plugin` impl (stub — registers no systems yet)
- [x] 1.2 Create empty module files: `card.rs`, `toolbar.rs`, `thumbnail.rs`, `context_menu.rs`, `empty_state.rs` under `src/ui/bevy_ui/explorer/`
- [x] 1.3 Register `ExplorerViewPlugin` in `src/ui/bevy_ui/mod.rs` behind the `UiType::Bevy` guard
- [x] 1.4 Add `mod explorer;` declaration and re-export to `src/ui/bevy_ui/mod.rs`
- [x] 1.5 Run `just check` — verify zero new errors before adding logic

## 2. Thumbnail cache resource

- [x] 2.1 Define `ThumbnailCache` resource in `thumbnail.rs` as `HashMap<ScenarioId, ThumbnailState>` where `ThumbnailState` is `Pending | Generating | Ready(Handle<Image>)`
- [x] 2.2 Register `ThumbnailCache` as a Bevy resource in `ExplorerViewPlugin`
- [x] 2.3 Add system `queue_thumbnail_generation` that iterates Done scenarios without a `Ready` entry and spawns an async task (one per frame max); task invokes the existing image-generation pipeline at 280×200 resolution
- [x] 2.4 Add system `poll_thumbnail_tasks` that checks completed tasks and transitions state to `Ready(handle)`
- [x] 2.5 Write unit test for `ThumbnailState` transitions (no asset server required — test state machine only)

## 3. Scenario card widget

- [x] 3.1 Define `ScenarioCardBundle` marker component and spawning function `spawn_card(commands, scenario, thumbnail_cache)` in `card.rs`
- [x] 3.2 Implement status badge node: pill-shaped `Node` with correct palette color per lifecycle status
- [x] 3.3 Implement thumbnail area node: renders `Ready` thumbnail image, spinner for `Generating`/`Pending`, placeholder icon for Planning/Queued, error icon for Failed, and progress ring + ETC text for Running
- [x] 3.4 Implement metrics row node: Dice score + final loss labels, visible only for Done scenarios
- [x] 3.5 Implement title node: truncated scenario ID, `fg0` color
- [x] 3.6 Implement comment node: italic, `grey1`, max 2 lines with ellipsis, hidden when no comment
- [x] 3.7 Implement timestamp node: creation date, `grey1`, small text
- [x] 3.8 Add `Interaction`-based system for hover state: change card background to `bg2` on hover
- [x] 3.9 Add system to apply orange 2 px border on the card matching `ActiveScenario`; remove border on all others

## 4. Card grid layout

- [x] 4.1 Create `ExplorerGridNode` marker; spawn grid root node in `ExplorerViewPlugin::build` on `OnEnter(AppView::Explorer)` and despawn on `OnExit`
- [x] 4.2 Add system `update_grid_columns` that reads computed node width each frame and sets `GridTrack` column count per breakpoint (1/2/3/4)
- [x] 4.3 Add system `sync_cards_to_scenarios` that diffs the scenario list against spawned cards and inserts/removes cards accordingly
- [x] 4.4 Add "New Scenario" action card at end of grid: dashed border, "+" icon, "New Scenario" label

## 5. Toolbar

- [x] 5.1 Spawn toolbar `Node` above the grid; add `StatusFilter`, `SortOrder`, `SearchQuery` resources
- [x] 5.2 Implement status filter dropdown widget in `toolbar.rs`; writes to `StatusFilter` resource on selection
- [x] 5.3 Implement sort order dropdown widget; writes to `SortOrder` resource
- [x] 5.4 Implement text search field; writes to `SearchQuery` resource on text change
- [x] 5.5 Add system `apply_filter_and_sort` that reads all three resources and sets `Visibility` on cards accordingly; re-orders card nodes by sort criteria
- [x] 5.6 Add "New Scenario" button in toolbar (right-aligned) with `orange` accent — same action as action card

## 6. Empty state

- [x] 6.1 Implement `EmptyStateNode` in `empty_state.rs`: centered "No scenarios yet" text + subtitle + "New Scenario" button
- [x] 6.2 Add system `toggle_empty_state` that shows `EmptyStateNode` and hides the grid when zero cards pass the active filter, and vice versa

## 7. Card interactions

- [x] 7.1 Add click handler system for scenario cards: sets `ActiveScenario` resource and triggers transition to `AppView::Scenario`
- [x] 7.2 Add click handler for "New Scenario" action card and toolbar button: creates new Planning scenario, sets it active, replaces action card with a new scenario card in inline-edit state (name + comment fields editable); stay on Explorer view
- [x] 7.2a Add inline-edit confirm handler: on Enter or focus-out, commit name/comment and transition card to normal display state
- [x] 7.3 Implement `context_menu.rs`: spawn `ContextMenuNode` on right-click at cursor position with `ZIndex::Global(100)`
- [x] 7.4 Populate context menu items based on scenario state (Copy, Delete, Schedule, Unschedule, Open in Results, Open in Volumetric)
- [x] 7.5 Add system `dismiss_context_menu` that despawns the menu on any click outside its bounds

## 8. Navigation wiring

- [x] 8.1 Wire the Explorer nav rail item (in `src/ui/bevy_ui/navigation.rs`) to emit `AppView::Explorer` transition events
- [x] 8.2 Verify Explorer nav item is disabled when no project is loaded (existing `ui-navigation` spec requirement)
- [x] 8.3 Verify Explorer nav item is enabled after a project is opened from Home view

## 9. Integration and verification

- [x] 9.1 Run `just check` — zero warnings or errors
- [x] 9.2 Run `just test` — all existing tests pass
- [x] 9.3 Run `just lint` — all `#[tracing::instrument]` annotations present on public functions in new module
- [x] 9.4 Manual smoke test native: open project, verify card grid renders, filter/sort works, thumbnail generates, card click navigates
- [x] 9.5 Manual smoke test WASM: build with `just release` targeting wasm, verify card grid in browser, thumbnails cached in memory
- [x] 9.6 Verify egui path (`UiType::EGui`) is unaffected — toggle with F2 and confirm existing Explorer table still works
