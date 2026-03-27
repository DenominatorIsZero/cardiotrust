## Why

The current egui Explorer is a dense, hard-to-scan table that gives no visual identity to scenarios. The Bevy UI redesign (phase 3 of the implementation strategy) introduces a card-grid Explorer view that surfaces status, thumbnails, key metrics, and timestamps at a glance — replacing the table with a more approachable, portfolio-quality layout that works equally well on native and WASM.

## What Changes

- New Bevy UI `ExplorerView` plugin replaces the egui scenario table in the `UiType::Bevy` path
- Responsive card grid: each scenario rendered as a `ScenarioCard` with status badge, thumbnail/progress area, metrics row, title, comment, and timestamp
- Filter/sort toolbar: filter by status, sort by date/loss/dice/name, text search
- "New Scenario" action card (dashed border) at end of grid
- Empty-state fallback when project has no scenarios
- On-demand thumbnail generation for `Done` scenarios (280×200, cached in resource)
- Card hover and selected (orange border) states
- Right-click / long-press context menu per card: copy, delete, schedule/unschedule, open in Results, open in Volumetric
- Click on card sets active scenario and navigates to Scenario view

## Capabilities

### New Capabilities

- `ui-explorer-view`: Bevy UI Explorer view — card grid layout, scenario cards, toolbar, empty state, thumbnail caching, context menu, and responsive column layout

### Modified Capabilities

- `ui-navigation`: Explorer nav item must transition to the Explorer view and respect the "greyed-out when no project" rule already in the spec; active card selection feeds the existing `ActiveScenario` resource that navigation already depends on

## Impact

- `src/ui/bevy_ui/explorer/` — new module (view plugin, card widget, toolbar, thumbnail cache)
- `src/ui/bevy_ui/mod.rs` — register ExplorerView plugin
- `src/ui/bevy_ui/navigation.rs` — wire Explorer nav rail item to the new view
- Depends on existing `bevy-sidebar-rail`, `bevy-view-routing`, and `ui-project-state` specs
- No changes to simulation, algorithm, or data-model code
- No egui code touched (egui path remains functional)
