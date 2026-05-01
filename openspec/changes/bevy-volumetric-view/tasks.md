## 1. View scaffolding

- [x] 1.1 Add a dedicated Bevy Volumetric view module and register it with the existing Bevy content-area routing.
- [x] 1.2 Introduce a Volumetric UI state resource for overlay section, fullscreen state, plot height, and plot collapsed state.
- [x] 1.3 Spawn the Volumetric view layout with context bar integration, viewport host node, right-edge tab strip, control overlay container, and bottom plot container.

## 2. Overlay controls

- [x] 2.1 Implement the right-edge section tabs with collapsed, expanded, and click-outside-to-close behavior.
- [x] 2.2 Build the Voxel Coloring section and bind its widgets to the existing visualization state for color mode, relative coloring, playback speed, manual mode, sample, beat, and sensor selection.
- [x] 2.3 Build the Visibility, Cutting Plane, and Sensor Bracket sections and bind them to the corresponding scene visibility and geometry state.

## 3. Plot and viewport behavior

- [x] 3.1 Add the bottom signal plot overlay using the existing waveform data, current sample cursor, and manual sample-selection interaction.
- [x] 3.2 Implement plot resizing, plot collapse/restore, and viewport resizing so the viewport always fills reclaimed space.
- [x] 3.3 Add the optional top-right toolbar actions, including reset camera and fullscreen toggle, and gate screenshot behavior if capture support is unavailable.

## 4. Responsive and integration polish

- [x] 4.1 Apply the narrow-screen responsive rules so the control panel behaves as a fuller overlay and the plot starts collapsed below the specified breakpoints.
- [x] 4.2 Verify that entering the Volumetric view still triggers voxel initialization and preserves existing visualization semantics and camera controls.
- [ ] 4.3 Test the Bevy Volumetric view on native and WASM-compatible paths and address layout or interaction regressions.
