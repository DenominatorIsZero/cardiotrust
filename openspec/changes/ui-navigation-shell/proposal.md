## Why

The app is destined to be embedded in a public portfolio as a WASM build, but its current flat top-bar navigation and egui-only UI look unpolished and are hard to extend into a six-view layout. Phase 1 of the UI redesign establishes the persistent Bevy-native navigation shell — a sidebar rail with view routing — that every subsequent view (Home, Explorer, Scheduler, etc.) will mount into.

## What Changes

- Add a left sidebar rail built with Bevy UI nodes (not egui), containing: app logo area, per-view navigation buttons, a spacer, and a Scheduler button pinned to the bottom.
- Extend `UiState` with two new variants: `Home` and `Scheduler` (formerly only accessible via topbar controls).
- Add a `SidebarState` resource tracking expanded/collapsed state (200 px vs 56 px) and the viewport-width auto-collapse threshold.
- Add a thin breadcrumb/context bar Bevy UI node at the top of the content column.
- Wire keyboard shortcuts 1–6 and Escape to the new view transitions.
- The existing egui topbar and its navigation buttons remain untouched under `UiType::EGui`; the new sidebar only renders under `UiType::Bevy`.
- No egui views are removed in this phase — the content area under the Bevy sidebar initially shows an empty placeholder that future phases will fill.

## Capabilities

### New Capabilities

- `bevy-sidebar-rail`: The persistent left-sidebar navigation shell — logo area, icon+label nav items with active/hover/disabled visual states, auto-collapse at narrow viewports, and a collapsed icon-only mode toggled by a chevron button.
- `bevy-view-routing`: Bevy-state-based routing for the six views (Home, Explorer, Scenario, Results, Volumetric, Scheduler) including precondition guards, keyboard shortcuts 1–6 and Escape, and breadcrumb context bar updates.
- `bevy-content-area`: The outer layout node that sits to the right of the sidebar, composing the breadcrumb bar and the swappable content slot.

### Modified Capabilities

- `ui-navigation`: The existing spec defines a top-bar navigation model with four views. The Bevy backend path now introduces six views and a sidebar rail model. The precondition rules (Scenario requires selection, Results/Volumetric require Done status) carry over unchanged. **No change to egui path behavior.**

## Impact

- `src/ui.rs` — extend `UiState` enum; add `SidebarState` resource; register new Bevy-path systems.
- `src/ui/bevy_shell/` — new module: `mod.rs`, `sidebar.rs`, `breadcrumb.rs`, `content_area.rs`, `routing.rs`.
- `src/ui/colors.rs` — no changes needed; constants are already defined and will be referenced by new systems.
- `Cargo.toml` — no new dependencies; Bevy UI node API is already available.
- Existing egui systems (`topbar.rs`, `explorer.rs`, `scenario.rs`, `results.rs`, `vol.rs`) — untouched.
