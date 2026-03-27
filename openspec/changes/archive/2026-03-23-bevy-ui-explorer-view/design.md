## Context

The Bevy UI rewrite is proceeding in phases. Phase 1 (sidebar rail) and Phase 2 (Home view) are complete. This design covers Phase 3: the Explorer view. The Explorer must replace the egui scenario table in the `UiType::Bevy` path with a responsive card grid. The egui path (`UiType::EGui`) must remain untouched and fully functional.

Relevant existing infrastructure:
- `bevy-sidebar-rail` spec and implementation — provides the nav rail that will host the Explorer nav item
- `bevy-view-routing` spec and implementation — the view-routing system the Explorer plugin registers with
- `ui-project-state` spec — `ActiveProject` and `ActiveScenario` resources that the Explorer reads and writes
- `src/ui/colors.rs` — Gruvbox Material palette constants already defined

## Goals / Non-Goals

**Goals:**
- Implement `ExplorerView` Bevy plugin: card grid, toolbar, empty state, thumbnail cache
- Responsive grid: 1–4 columns based on available width
- Per-card: status badge, thumbnail/progress/placeholder area, metrics row, title, comment, timestamp
- Toolbar: status filter, sort order, text search, New Scenario button
- "New Scenario" action card at end of grid
- On-demand thumbnail generation for Done scenarios (280×200, cached in a Bevy `Resource`)
- Card selection writes to `ActiveScenario` resource and triggers navigation to Scenario view on click
- Context menu per card (copy, delete, schedule/unschedule, open in Results/Volumetric)
- Card hover and selected (orange border) visual states
- Empty-state UI when project has no scenarios

**Non-Goals:**
- Modifying the egui Explorer or any egui code
- Thumbnail persistence to disk (WASM-compatible in-memory cache only for this phase)
- Pagination (all scenarios rendered; scrollable)
- List view toggle (grid only for MVP)

## Decisions

### D1: Bevy UI node tree per card rather than egui widgets

All cards are built from Bevy `Node`/`ImageNode`/`Text` entities rather than spawning egui panels. This is consistent with the existing Bevy UI phases and keeps the rendering path unified.

*Alternatives considered:* Embedding an egui sub-panel for the Explorer inside the Bevy window — rejected because it mixes rendering backends and undermines the migration goal.

### D2: Thumbnail cache as a `Resource<HashMap<ScenarioId, Handle<Image>>>`

A single Bevy resource holds a map from `ScenarioId` → `Handle<Image>`. Generation is dispatched as a one-shot Bevy task; the task writes back into the map when complete.

*Alternatives considered:* Storing thumbnails in the scenario data struct — rejected because thumbnails are view-layer concerns; they should not pollute the domain model.

### D3: Responsive columns via measured available width

The grid system reads the `Node`'s computed width each frame and derives column count using the breakpoints from the design spec (≥1400px → 4, 1000–1399px → 3, 700–999px → 2, <700px → 1). Cards are sized to fill their column with a 16px gap.

*Alternatives considered:* Fixed 3-column layout — rejected because the design spec requires responsiveness and the WASM build runs at variable browser widths.

### D4: Context menu as an overlay `Node` spawned on right-click

Right-click (or long-press on touch) spawns a small `Node` overlay positioned near the cursor. The overlay despawns when focus moves away or an action is selected. This avoids a dependency on egui for the menu.

*Alternatives considered:* Using egui context menu — rejected to maintain the clean Bevy-only rendering boundary.

### D5: Module layout under `src/ui/bevy_ui/explorer/`

```
src/ui/bevy_ui/explorer/
  mod.rs          — ExplorerViewPlugin (registers all systems)
  card.rs         — ScenarioCard spawn/update systems
  toolbar.rs      — FilterBar, SortBar, SearchField systems
  thumbnail.rs    — ThumbnailCache resource + generation task
  context_menu.rs — ContextMenu overlay systems
  empty_state.rs  — EmptyState node
```

The plugin is registered in `src/ui/bevy_ui/mod.rs` behind the existing `UiType::Bevy` guard.

## Risks / Trade-offs

- **Thumbnail generation cost** → Mitigation: generate at 280×200 (small), throttle to one task per frame, skip if a task is already in flight for that scenario.
- **Bevy UI layout performance with many cards** → Mitigation: mark card nodes `visibility: Hidden` for scenarios outside the viewport (manual culling); revisit if profiling shows issues.
- **Context menu z-ordering** → Mitigation: spawn context menu at a fixed high z-index (`ZIndex::Global(100)`); despawn on any click outside.
- **WASM thumbnail memory** → Thumbnails are `Handle<Image>` backed by GPU memory; on WASM this is WebGL2/WebGPU texture memory. Keep thumbnail resolution small (280×200) to stay within budgets.

## Migration Plan

1. Add `explorer` submodule, implement behind a `#[cfg]` or `UiType` check.
2. Register `ExplorerViewPlugin` in `bevy_ui/mod.rs`.
3. Wire Explorer nav rail item to the new view in `navigation.rs`.
4. Verify existing egui path is unaffected (`just test`, `just check`).
5. Manual smoke test in native and WASM builds.
6. No rollback needed — egui path remains fully functional.

## Open Questions

~~- Should "New Scenario" navigate to Scenario view immediately, or stay on Explorer with the card in an inline-edit state for the name/comment?~~
**Resolved:** Stay on Explorer. The new card enters an inline-edit state for name/comment; navigation to Scenario view only happens on an explicit card click.

~~- Thumbnail generation uses CPU rendering of simulation output data. Should we reuse the existing image-generation pipeline or write a lighter-weight path?~~
**Resolved:** Reuse the existing pipeline. The thumbnail task invokes the same image-generation code at reduced resolution. Improvements will be deferred to the Results view rework phase.
