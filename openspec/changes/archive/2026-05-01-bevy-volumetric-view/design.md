## Context

The application already has three relevant pieces in place: the Bevy navigation shell and content-area host, the existing volumetric rendering pipeline, and an egui-based volumetric control surface. This change bridges them by defining a Bevy-native Volumetric view that preserves current visualization semantics while replacing the screen layout and interaction model with the overlay-heavy design in `docs/ui-design/volumetric-view.md`.

The main constraint is that the 3-D scene remains the primary surface. The new UI must not reintroduce a permanent sidebar that reduces the viewport, must continue to work with the existing camera and visualization state, and must stay viable on both native and WASM builds. Plotting is still best served by an egui overlay because Bevy UI does not provide an equivalent plotting widget.

## Goals / Non-Goals

**Goals:**
- Render the Volumetric route as a dedicated Bevy view with a large viewport, right-edge collapsible control overlay, and bottom plot panel.
- Reuse existing visualization resources and behaviors for color mode, animation state, beat selection, cutting plane, sensor placement, and voxel initialization.
- Support responsive behavior for narrow windows, plus fullscreen and plot-collapse states that maximize viewport space.
- Keep the design compatible with WASM without introducing native-only dependencies.

**Non-Goals:**
- Changing the numerical visualization behavior, color semantics, or scene-construction rules already covered by the visualization spec.
- Replacing the plotting implementation with a pure Bevy widget.
- Implementing screenshot export if the platform plumbing is not already available.
- Redesigning the global Bevy shell, breadcrumb model, or non-Volumetric views.

## Decisions

### 1. Build the Volumetric screen as a Bevy content-area view with one egui plot overlay

The main layout should live inside the Bevy content slot so it follows the shell's routing and sizing rules. The signal plot remains an egui overlay anchored to the bottom of the content area because `egui_plot` already matches the interaction requirements and works on WASM.

**Alternative considered:** Implement the entire screen in egui within the Bevy route.
**Rejected:** That would bypass the Bevy shell layout direction and keep the view visually inconsistent with the rest of the Bevy UI migration.

### 2. Keep overlay state in a dedicated Volumetric UI resource

The view needs state that is purely presentational: active overlay section, whether the controls are expanded, whether fullscreen is active, plot height, and whether the plot is collapsed. This state should be stored separately from visualization-domain resources so layout concerns do not leak into rendering logic.

**Alternative considered:** Fold the overlay state into existing visualization resources.
**Rejected:** These values do not affect the scientific state of the scene; mixing them together would make the visualization systems harder to reason about and test.

### 3. Drive control widgets by adapting existing visualization state instead of duplicating it

Every control in the overlay should read and write the same underlying state already used by the existing volumetric behavior. The Bevy UI layer becomes a new interaction surface over the same visualization model.

**Alternative considered:** Introduce a second view-model layer and synchronize it back to visualization resources.
**Rejected:** That adds synchronization risk for controls like manual sample selection, beat changes, and cutting-plane parameters without adding user-visible value.

### 4. Represent the right-side controls as icon tabs plus a slide-out panel

The collapsed state is a narrow vertical strip of section buttons. Activating a section expands a fixed-width panel over the viewport; activating the same section again, clicking outside, or entering fullscreen collapses it.

**Alternative considered:** Keep a fixed-width control column.
**Rejected:** The design brief explicitly prioritizes viewport area and calls for controls that stay accessible without permanently shrinking the 3-D scene.

### 5. Use size thresholds rather than separate layouts for responsiveness

The same component structure should handle desktop and narrow widths by changing widths, default visibility, and overlay behavior based on viewport size. Below the narrow breakpoint, the control panel becomes a fuller overlay and the plot starts collapsed.

**Alternative considered:** Build a second mobile-specific Volumetric tree.
**Rejected:** The behavior changes are modest; a second layout would increase maintenance cost and divergence risk.

## Risks / Trade-offs

- [Mixed Bevy + egui input regions] -> Scope egui interaction to the plot region and explicitly ignore pointer gestures there when routing viewport camera input.
- [Overlay state drifting from scene state] -> Bind controls directly to existing visualization resources and keep the new resource limited to layout-only state.
- [Fullscreen and responsive rules causing layout edge cases] -> Centralize the derived layout calculations so panel width, plot height, and collapsed states are resolved in one place.
- [Optional toolbar actions exceeding current platform support] -> Treat screenshot capture as an optional action that can remain disabled until backing support exists.
- [WASM performance under dense voxel scenes] -> Keep the UI layer lightweight and avoid adding per-frame UI work beyond the existing plot and control updates.

## Migration Plan

1. Add the new Bevy Volumetric view state and layout resource behind the existing Bevy routing path.
2. Mount the viewport host, overlay controls, and bottom plot container without changing visualization-domain behavior.
3. Bind each control section to the existing visualization state and verify parity with the current volumetric interactions.
4. Enable responsive and fullscreen behaviors, then retire the older egui-only volumetric screen from the Bevy route.

Rollback is low risk because the rendering pipeline remains in place; the route can temporarily fall back to the previous volumetric presentation if the new view is not stable.

## Open Questions

- Whether the screenshot toolbar action should ship as a disabled placeholder in the first implementation if capture support is incomplete.
- Whether click-outside-to-collapse should include clicks inside the viewport only, or any click outside the panel including the plot region.
