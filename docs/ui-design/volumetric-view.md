# Volumetric View -- 3D Visualization

## Purpose

Interactive 3D visualization of the heart model, sensors, and simulation results. The 3D viewport is the star -- controls should be accessible but not dominant.

## Layout

```
+------------------------------------------------------------------+
| Context bar: My Project > Scenario v8_Dr > Volumetric            |
+------------------------------------------------------------------+
|                                                        +--------+|
|                                                        | Voxel  ||
|                                                        | Color  ||
|                 3D Viewport                            |--------||
|              (Bevy render target)                      | Visib. ||
|                                                        |--------||
|                                                        | Cut    ||
|                                                        | Plane  ||
|                                                        |--------||
|                                                        | Sensor ||
|                                                        | Bracket||
|                                                        +--------+|
+------------------------------------------------------------------+
|                    Signal Plot (resizable)                        |
|  [measurement waveform with vertical cursor]                     |
+------------------------------------------------------------------+
```

## 3D Viewport

- Takes up the majority of the screen (everything not occupied by the control panel or plot)
- Bevy's camera (pan/orbit/zoom via `bevy_editor_cam`) controls the view
- Background: Bevy scene background (dark, matching theme)
- The "Init Voxels" action is triggered automatically when entering the view (if not already initialized)

## Control Panel -- Collapsible Overlay

Instead of a fixed-width left sidebar that permanently reduces viewport space, controls live in a **collapsible right-side panel** that overlays the 3D viewport.

### Panel behavior
- **Default**: Collapsed -- shows only section tab icons along the right edge
- **Expand**: Click any section icon to expand the panel (~280px wide)
- **Collapse**: Click the active section icon again, or click outside the panel
- **Semi-transparent background**: `bg1` at 90% opacity, so the 3D scene is partially visible behind it
- Panel slides in/out with a quick animation

### Section tabs (vertical, right edge)

Each section is an icon button stacked vertically:

1. **Voxel Coloring** (palette icon)
2. **Visibility** (eye icon)
3. **Cutting Plane** (scissors/plane icon)
4. **Sensor Bracket** (bracket icon)

Clicking a tab opens that section's controls in the expanded panel.

### Section contents

#### Voxel Coloring
- **Color mode**: Combo box with 10 ColorMode variants
- **Relative coloring**: Checkbox
- **Playback speed**: Slider (0.01 to 1.0, logarithmic)
- **Manual mode**: Checkbox
- **Sample**: Slider (0 to max, disabled when not manual)
- **Motion Step**: Slider (0 to num_beats)
- **Sensor**: Slider (0 to num_sensors)

#### Visibility
A simple list of toggles:
- Heart
- Cutting plane
- Sensors
- Sensor bracket
- Torso
- Room

Each as a labeled checkbox, compact vertical list.

#### Cutting Plane
- **Enabled**: Checkbox
- **Origin (X, Y, Z)**: 3 number inputs
- **Normal (X, Y, Z)**: 3 number inputs (step 0.01)
- **Opacity**: Slider (0 to 1)

#### Sensor Bracket
- **Position (X, Y, Z)**: 3 number inputs (mm)
- **Radius**: Number input (mm)

## Signal Plot -- Bottom Panel

The time-series measurement plot at the bottom of the view.

### Layout
- **Height**: 250px default (was 400px -- slightly reduced to give more viewport space)
- **Resizable**: Drag handle at the top edge of the plot panel to resize vertically
- **Collapsible**: Double-click the drag handle to minimize to a thin bar (30px), double-click again to restore
- Background: `bg_dim`

### Plot contents
- X-axis: Time (seconds)
- Y-axis: Measurement amplitude
- **Signal line**: `red` (current sensor, current beat)
- **Cursor**: Vertical line at the current sample time (`orange`)
- Axis labels and tick marks in `grey1`

### Plot interaction
- Click on the plot to set the current sample time (when in manual mode)
- Scroll to zoom the time axis
- The plot uses `egui_plot` (which is fine even in the Bevy UI -- it can render in an egui overlay specifically for this plot, since Bevy's native UI doesn't have a plotting widget)

## Toolbar (Optional)

A small floating toolbar in the top-right of the viewport:

- **Reset camera**: Button to reset to default view angle
- **Screenshot**: Capture current 3D view as PNG
- **Fullscreen**: Hide all panels and plot, maximize viewport (toggle)

These are small icon buttons, semi-transparent background.

## Responsive Behavior

- Below 1000px: control panel is full-overlay (covers viewport when open) instead of side panel
- Below 800px: plot panel starts collapsed
- The 3D viewport always fills available space

## WASM Notes

- Bevy's WebGPU/WebGL2 renderer handles the 3D viewport
- `egui_plot` works in WASM
- Camera controls work with mouse/touch
- Performance: may need to reduce voxel count or use LOD for complex models in WASM
