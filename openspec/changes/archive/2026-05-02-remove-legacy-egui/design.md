## Context

CardioTrust originally carried two UI backends side by side: a legacy egui-driven surface and the newer Bevy-native shell. The redesign documents explicitly treated the egui path as transitional, and the major views have now been migrated into Bevy-native screens. The remaining legacy path is concentrated in the UI plugin bootstrap, the old top-bar and panel modules, the `UiType` selector, and the F2 development toggle.

This cleanup is cross-cutting because it touches startup behavior, routing expectations, legacy modules, and dependency usage. The main constraint is that the repository must not remove crates that are still used by the retained product surface: the Bevy volumetric view still uses scoped egui overlays for the signal plot and screenshot overlay flow, and the Home/native project-loading flow still relies on the file-dialog dependency.

## Goals / Non-Goals

**Goals:**
- Reduce the UI architecture to one supported backend and remove the dead switching contract.
- Delete legacy egui-only modules and bootstrap wiring without changing the intended Bevy-native user experience.
- Remove only the dependencies that truly become unused after the code cleanup.
- Update OpenSpec requirements so they no longer describe backend-specific startup, top-bar rendering, or F2 backend switching.

**Non-Goals:**
- Replacing the scoped egui overlay that remains inside the Bevy volumetric view.
- Redesigning the Home, Explorer, Scenario, Results, Scheduler, or Volumetric Bevy-native experiences.
- Changing scheduler, scenario-editing, or visualization behavior beyond removing legacy UI entry points.

## Decisions

### 1. Remove the backend selector entirely instead of keeping a degenerate single-value abstraction

The `UiType` state exists only to gate systems between the legacy egui path and the Bevy-native path. Once the legacy path is deleted, retaining a single-value backend selector would preserve branching and documentation overhead without giving the product a meaningful extension point. The cleanup should therefore remove the selector, its defaulting behavior, and the F2 toggling system, then register the retained UI systems directly.

**Alternative considered:** Keep `UiType` with only a Bevy variant for future optional backends.
Rejected because it preserves needless runtime state and continues to imply a supported multi-backend contract that this change is explicitly retiring.

### 2. Treat legacy egui view files as deletions, not compatibility shims

The old `topbar`, `explorer`, `scenario`, and `vol` modules should be removed rather than left disconnected or hidden behind feature flags. Leaving them in-tree would continue the maintenance burden this cleanup is meant to remove and would make it harder to tell which UI implementation is authoritative.

**Alternative considered:** Leave the old modules in place but stop registering them.
Rejected because dormant code still drifts, still influences dependency retention, and still creates confusion during future UI work.

### 3. Clean dependencies from actual remaining usage, not from historical ownership

Dependency cleanup should happen after the code-path deletion is mapped. `egui`, `egui_plot`, and `bevy_egui` cannot be removed simply because the old full-screen egui UI is gone; the retained Bevy volumetric overlay still uses them. By contrast, any crate used only by the deleted legacy tables or panels should be removed once no remaining code references it.

**Alternative considered:** Remove all egui-family crates in the same change because the request mentions old egui cleanup.
Rejected because it would conflict with the still-supported volumetric overlay behavior and create an avoidable regression.

### 4. Express the behavioral change through targeted spec deltas only

This change affects existing capabilities rather than introducing a new one. The proposal therefore modifies `ui-navigation` and `ui-type-resource`: the former becomes single-path Bevy-native navigation behavior, and the latter is removed because its contract no longer applies.

**Alternative considered:** Introduce a new capability for legacy UI removal.
Rejected because the user-visible behavior is already governed by existing UI capabilities; adding a new capability would duplicate ownership rather than clarify it.

## Risks / Trade-offs

- [Risk] A supposedly legacy dependency is still referenced by the retained Bevy-native volumetric or project-loading flow -> Mitigation: inventory code references before removing manifest entries and keep any crate still used by the supported UI.
- [Risk] Startup or routing behavior changes accidentally because legacy gating code is intertwined with current initialization -> Mitigation: keep the cleanup focused on deleting backend branching, then verify the single-path startup state and major route transitions.
- [Risk] Documentation drift between archived migration-era changes and the new steady-state requirements -> Mitigation: update the active capability specs to describe only the supported product behavior and let archived changes remain historical records.

## Migration Plan

1. Remove backend-selection state and F2 toggling from the UI bootstrap, then wire the retained Bevy-native systems directly.
2. Delete the legacy egui modules and any imports, resources, or registrations that only served those modules.
3. Remove crates and feature flags that no longer have live references.
4. Verify the application still starts on Home, retains Bevy-native navigation behavior, and keeps any scoped overlays that remain part of the supported UI.

Rollback is straightforward during development: restore the deleted UI bootstrap and legacy modules from version control if cleanup exposes retained behavior that still depends on them.

## Open Questions

- None at proposal time; the remaining uncertainty is implementation-level dependency inventory rather than product direction.
