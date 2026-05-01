## Why

The Bevy navigation shell now has a dedicated Volumetric route, but the volumetric experience is still defined by the older egui layout rather than a Bevy-first view that gives the 3-D scene priority. This change brings the volumetric screen in line with the new UI direction so the heart scene, overlay controls, and signal plot behave like a cohesive Bevy view on desktop and WASM.

## What Changes

- Add a Bevy-native volumetric view layout with a dominant 3-D viewport, a collapsible right-edge control overlay, and a bottom signal plot panel.
- Organize volumetric controls into four overlay sections: voxel coloring, visibility, cutting plane, and sensor bracket.
- Define the expected panel behaviors for expand/collapse, fullscreen mode, responsive breakpoints, and plot resizing/collapsing.
- Preserve the existing visualization data semantics and camera interaction while changing how users access those controls in the Bevy UI.
- Keep the implementation compatible with desktop and WASM targets.

## Capabilities

### New Capabilities
- `bevy-volumetric-view`: The Bevy-native Volumetric screen layout, overlay controls, bottom signal plot panel, toolbar actions, and responsive behavior for the 3-D visualization workspace.

### Modified Capabilities

## Impact

- `src/ui/` Bevy view modules for the Volumetric route and its layout/state.
- Volumetric control bindings that bridge the Bevy UI to the existing visualization resources and systems.
- The signal plot presentation layer, including resizing/collapse state and manual sample selection behavior.
- Existing visualization rendering, camera controls, and scenario-loaded data flow, which remain the backing engine for the new UI.
