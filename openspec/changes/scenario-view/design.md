## Context

The scenario editor currently lives in `src/ui/scenario.rs` (top bar) and `src/ui/scenario/{data,algorithm,common}.rs` (parameter panels). It uses `egui_extras::TableBuilder` with three fixed columns (Parameter | Value | Description) and a two-column `CentralPanel` layout — simulation parameters on the left, algorithm parameters on the right.

The project is actively migrating from egui-based UI (`UiType::EGui`) to Bevy-native UI (`UiType::Bevy`). The Explorer view is already fully implemented in Bevy-native; the scenario editor is still egui-only. The breadcrumb bar, sidebar, and content slot are Bevy-native infrastructure that the new view must slot into.

The Gruvbox Material palette is already defined in `src/ui/colors.rs` as `bevy::prelude::Color` constants and used throughout the Bevy-native shell.

## Goals / Non-Goals

**Goals:**
- Implement the scenario view as a Bevy-native UI component that mounts into the existing content slot infrastructure (`bevy_content_area`)
- Organize parameters into three tabs: Simulation, Algorithm, Model
- Add a Scenario Header Bar (ID, status badge, model-type selector, action buttons, comment field)
- Implement collapsible sections within each tab with icon, title, and chevron
- Replace the three-column table layout with single-column parameter rows (label | control | value+unit)
- Enforce read-only / disabled state when scenario is not in Planning status
- Add hover tooltips on parameter labels (replacing the third-column descriptions)
- Implement Copy Scenario and Delete Scenario (with confirmation) workflows
- Support conditional parameter visibility (e.g., sensors-per-axis for Cube geometry only)

**Non-Goals:**
- Rewriting or refactoring the simulation/algorithm backend
- Changing any `Configuration` data structures (this is a pure UI change)
- Implementing the Results view or Visualization view
- Changing the egui legacy path (`UiType::EGui`) — it remains as-is during transition
- Responsive/mobile layout (the application is a desktop tool; < 800 px is a stretch goal)

## Decisions

### Decision 1: Implement as Bevy-native UI, not egui

The project's direction is toward Bevy-native UI (the Explorer is already native). The new scenario view will follow the same architecture: Bevy UI nodes, `BackgroundColor`/`BorderColor`/`TextColor` components, click/hover observers, and the existing color palette.

**Alternative considered:** Extend the existing egui panels. Rejected because: (a) it would entrench the egui path the project is moving away from; (b) egui does not provide smooth animation primitives for section collapse/expand; (c) the Bevy-native shell already handles routing and breadcrumb state.

### Decision 2: Tab state and collapse state held in a Bevy resource

A `ScenarioViewState` resource stores: `active_tab: ScenarioTab` (enum) and `section_collapsed: HashMap<SectionId, bool>`. This is created when the view is spawned and cleared when the view is despawned.

**Alternative considered:** Store state as ECS components on the tab/section nodes. Rejected because multiple systems need to read/write the same state (header bar actions, tab clicks, section header clicks), and a single resource is simpler to coordinate.

### Decision 3: Parameter rows as custom Bevy UI widgets

Each parameter row is a function `spawn_param_row(parent, label, unit, section_id, param_id)` that spawns a horizontal Bevy UI node containing:
- A `Text` label node (fixed 200px width)
- A control slot node (flexible, fills remaining space)
- A value+unit `Text` node (fixed 80px width, right-aligned)

Sliders, combo boxes, checkboxes, and number inputs are implemented as reusable widget functions that populate the control slot. This mirrors the existing `draw_*_settings` pattern but in Bevy-native form.

**Alternative considered:** Use egui for parameter controls inside a Bevy UI frame (hybrid). Rejected because mixing two rendering paths in the same panel is fragile and prevents applying consistent theming.

### Decision 4: Tooltip via overlay node with visibility toggling

Tooltips are implemented as a single shared overlay `Text` node (absolute positioning, z-index above content) that is repositioned and made visible on hover events. A `TooltipTarget` component on each label node stores the tooltip string.

**Alternative considered:** Per-label tooltip nodes (hidden/shown individually). Rejected because it spawns many hidden nodes and is harder to position correctly relative to the cursor.

### Decision 5: Keep egui scenario path until new view is feature-complete

The `UiType::EGui` scenario path remains unchanged. The new view is wired into `UiType::Bevy` only. During development, pressing F2 continues to switch between the two modes. The egui path is removed in a separate cleanup task after the Bevy-native view is verified.

**Alternative considered:** Delete egui path immediately. Rejected because it removes the ability to compare behavior during development and risks regressions without a reference.

### Decision 6: Copy and Delete invoke existing scenario backend functions

The Copy and Delete buttons emit Bevy events (`CopyScenarioEvent`, `DeleteScenarioEvent { scenario_id }`) that are handled by existing systems in `src/core/` (or new thin event-handler systems there). The UI layer does not touch storage directly.

A delete confirmation is a modal overlay node, not a separate egui window.

## Risks / Trade-offs

- **[Risk] Bevy-native sliders are not standard** — egui has a well-tested `Slider` widget; Bevy UI has no built-in slider. We must implement a custom slider widget.  
  → Mitigation: Build a minimal `SliderWidget` (track + filled portion + draggable thumb) as a reusable node bundle. Keep it simple: no animation, drag-only interaction. Number inputs (`DragValue` equivalent) can use a `TextEdit`-style `TextInput` node for the initial implementation.

- **[Risk] Conditional visibility causes layout reflows** — showing/hiding nodes in a vertical stack forces Bevy to re-layout the section body.  
  → Mitigation: Use `Display::None` (sets `Val::ZERO` size) rather than `Visibility::Hidden`, so hidden rows take no space. Bevy 0.15+ handles this correctly with `Node::display`.

- **[Risk] Many parameter rows increase entity count** — ~40 parameters × ~5 nodes each = ~200 entities per view.  
  → Mitigation: This is well within Bevy's comfortable range for a desktop application. No LOD strategy needed.

- **[Risk] Scroll position loss on tab switch** — if the view is rebuilt on each tab switch, scroll position resets.  
  → Mitigation: Keep all three tab bodies in the ECS tree at all times; toggle `Display::None / Flex` to switch tabs rather than despawning and respawning.

## Migration Plan

1. Add the new Bevy-native scenario view under `src/ui/bevy_shell/scenario/` following the same module pattern as `bevy_shell/explorer/`
2. Register the view plugin in `src/ui/bevy_shell/mod.rs`
3. Wire routing: when `UiState::Scenario` is entered with `UiType::Bevy`, spawn the scenario view root into the content slot
4. The egui path continues to function unchanged under `UiType::EGui`
5. Once the Bevy view is verified against all scenarios, remove the egui scenario files in a follow-up

**Rollback:** Switch `UiType` back to `EGui` via F2 at any time.

## Open Questions

- Should the custom slider widget support keyboard input (arrow keys to increment)? Start without, add later if needed.
- The `ComboBox` equivalent in Bevy-native: use a dropdown overlay node, or continue using `egui::ComboBox` for enum selections? Recommend native dropdown for consistency, but this adds implementation scope.
- Exact icon assets for section headers (gear, antenna, etc.) — are Bevy icon fonts or image assets available in the project? If not, omit icons in the first iteration and use colored rectangles as visual anchors.
