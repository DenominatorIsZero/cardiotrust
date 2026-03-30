## Why

The current scenario configuration UI presents all parameters as a two-column wall of sliders and fields, making it cognitively expensive to locate and edit specific settings. Researchers need a structured, navigable editor that organizes the ~40 configuration parameters by concern (simulation, algorithm, model), matches the existing Gemini mockup design direction, and enforces the configuration-locking rules already specified in `scenario-lifecycle`.

## What Changes

- Replace the current flat parameter list with a tabbed layout: **Simulation**, **Algorithm**, **Model**
- Add a **Scenario Header Bar** with ID display, status badge, model-type selector, action buttons (Save, Copy, Delete, Schedule/Unschedule), and a comment field
- Implement **collapsible sections** within each tab (accordion-style) with section icons and chevron indicators
- Render parameters as structured **parameter rows**: label + control + value/unit, replacing the current two-column layout
- Add **tooltip descriptions** on hover over parameter labels (replacing the current third-column description text)
- Support **conditional parameter visibility** (e.g., "Sensors per axis" only for Cube geometry)
- Enforce **disabled state** for all editor controls when scenario is not in Planning status
- Implement **Copy Scenario** workflow (duplicate config, navigate to new scenario, preserve expand/collapse state)
- Add **Delete Scenario** confirmation dialog

## Capabilities

### New Capabilities

- `ui-scenario-view`: The full scenario configuration editor view — tabbed layout, collapsible sections, parameter rows, header bar, action buttons, and conditional visibility logic

### Modified Capabilities

- `scenario-lifecycle`: The view must enforce configuration locking (read-only when not Planning) and trigger schedule/unschedule transitions from the header bar; the Copy and Delete actions also interact with lifecycle state

## Impact

- `src/ui/` — new scenario view module(s); routing from the existing content area to this view
- `src/scenario/` or equivalent — Copy and Delete actions need backend support
- `bevy-content-area`, `bevy-view-routing` specs govern how the view is mounted and navigated to
- `ui-explorer-view` spec: the Explorer navigates to this view on scenario selection
- No changes to the simulation pipeline, algorithm, or model code
