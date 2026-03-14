# Scenario View -- Configuration Editor

## Purpose

Edit all parameters of a scenario. Replaces the current two-column wall of sliders with a tabbed, single-column layout using collapsible sections. Inspired by the Gemini mockup (see `references/new_ui/scenrario_mockup_downscaled.jpg`).

## Layout

```
+------------------------------------------------------------------+
| Context bar: My Project > Scenario v8_Dr_2026-03-14              |
+------------------------------------------------------------------+
| [Scenario Header Bar]                                            |
| ID: v8_Dr_2026-03-14  |  Status: Planning  |  Model: Handcraft  |
| [Save] [Copy] [Delete] [Schedule]          Comment: [________]  |
+------------------------------------------------------------------+
|  [ Simulation ]  [ Algorithm ]  [ Model ]                        |
+==================================================================+
|                                                                  |
| > Core Setup                                            [v]      |
| +--------------------------------------------------------------+ |
| | Sample Rate     [===|=========] 2000 Hz                      | |
| |                 1000             48000                        | |
| | Duration        [==|==========] 1.0 s                        | |
| |                 0.1              60                           | |
| +--------------------------------------------------------------+ |
|                                                                  |
| > Sensor Configuration                                  [v]      |
| +--------------------------------------------------------------+ |
| | Geometry        [Cylinder    v]                               | |
| | Motion          [Static      v]                               | |
| | 3D Sensors      [x]                                          | |
| | Array Origin    X [__] Y [__] Z [__] mm                      | |
| | ...                                                          | |
| +--------------------------------------------------------------+ |
|                                                                  |
| > Measurement Data                                      [^]      |
|   (collapsed -- click to expand)                                 |
|                                                                  |
| > Propagation Velocity                                  [^]      |
|   (collapsed)                                                    |
|                                                                  |
+------------------------------------------------------------------+
```

## Scenario Header Bar

A fixed header at the top of the content area (below the breadcrumb):

### Left section
- **Scenario ID**: Displayed as text, `fg0`
- **Status badge**: Same pill style as Explorer cards
- **Model type selector**: `ComboBox` (Handcrafted / MRI) -- only editable in Planning status

### Right section
- **Action buttons** (horizontal):
  - **Save** (`aqua` accent) -- saves current configuration
  - **Copy** -- duplicates scenario with new ID, navigates to the copy
  - **Delete** (`red`, requires confirmation) -- deletes and returns to Explorer
  - **Schedule** / **Unschedule** -- toggle, `orange` when scheduling
- **Comment field**: Single-line text input, auto-saves on blur

### Disabled state
When scenario is not in `Planning` status, all editor controls below are disabled (visually muted). The header still shows actions (Save is hidden, Schedule becomes Unschedule, etc.).

## Tabs

Three tabs below the header:

| Tab | Contents |
|-----|----------|
| **Simulation** | Core Setup, Sensor Configuration, Measurement Data, Propagation Velocity |
| **Algorithm** | Algorithm Settings, Optimizer, Regularization, Metrics |
| **Model** | Heart Settings (size/offset), Handcrafted Model params OR MRI Model path, Functional Settings |

### Tab styling
- Tabs are horizontal, centered or left-aligned below the header
- Active tab: `orange` underline (3px), text in `fg0`
- Inactive tab: no underline, text in `grey1`
- Hover: text in `fg1`
- Tabs span full width, with even spacing

### Tab assignment of current settings

**Simulation tab:**
- Core Setup: sample rate, duration
- Sensor Configuration: geometry, motion, 3D, origin, sensors per axis, size, radius, count, motion range/steps
- Measurement Data: covariance mean, covariance std

**Algorithm tab:**
- Algorithm Settings: type (ModelBased/GPU/PseudoInverse), epochs, batch size, freeze gains, freeze delays
- Optimizer Settings: optimizer type, learning rate, LR reduction interval, LR reduction factor
- Regularization Settings: threshold, strength
- Metrics Settings: snapshot interval

**Model tab:**
- Heart Geometry: voxel size, heart offset, heart size (handcrafted only)
- Functional Settings: control function, pathological toggle, current factor
- Propagation Velocity: sinoatrial, atrium, AV, HPS, ventricle, pathological
- Handcrafted Model: SA center, include atrium/AV/HPS, percentages, pathology region
- MRI Model: path input

## Collapsible Sections

Each section within a tab:

### Section header
- **Icon** (left): Small thematic icon (gear for Core Setup, antenna for Sensors, etc.)
- **Title**: Section name in `fg0`, slightly larger than body text
- **Collapse chevron** (right): `v` when expanded, `>` when collapsed
- Background: `bg1`, full width
- Click anywhere on the header to toggle
- Border-bottom: 1px `bg3`

### Section body
- Background: `bg0` (base)
- Padding: 16px horizontal, 12px vertical
- Contains parameter rows

### Default expand state
- First section in each tab starts expanded
- Others start collapsed
- State is remembered per session (not persisted)

## Parameter Rows

Each parameter is a single horizontal row:

```
Label                    [Control]              Value + Unit
```

### Row layout
- **Label**: Left-aligned, `fg1`, 200px min-width
- **Control**: Slider, combo box, checkbox, or input field -- fills available space
- **Value display**: Right-aligned, shows current numeric value with unit
- Row height: ~36px with 4px vertical padding
- Separator: subtle 1px `bg2` line between rows, or alternating row backgrounds (`bg0` / `bg_dim`)

### Control types

| Widget | Usage |
|--------|-------|
| Slider | Continuous numeric values (sample rate, duration, learning rate, etc.) |
| Combo box | Enum selections (geometry, optimizer, algorithm type) |
| Checkbox | Boolean toggles (3D sensors, pathological, freeze gains) |
| Number input | Precise values where slider is awkward (voxel coordinates) |
| XYZ group | Three number inputs in a row for 3D coordinates |
| Text input | Paths (MRI model) |

### Slider design
- Track: `bg3` (full width of control area)
- Filled portion: `orange` gradient (left of thumb)
- Thumb: Circle, `fg0` fill, `bg3` border
- Min/max labels below the track in `grey1`, small text
- Current value displayed in a box to the right of the slider

### Conditional visibility
- Parameters that depend on other settings (e.g., "Sensors per axis" only for Cube geometry) are shown/hidden with a slide animation
- When hidden, they take no space

## Description Tooltips

The current UI shows descriptions as a third column, which takes too much space. Instead:
- Hover over any parameter label to see a tooltip with the description
- Tooltip: `bg3` background, `fg1` text, 8px padding, max-width 300px
- Appears after 500ms hover delay
- Small "?" icon next to complex parameters (clicking also shows tooltip)

## Responsive Behavior

- Below 800px width: labels stack above controls instead of beside them
- The tab bar remains horizontal but may scroll
- Section headers remain full-width
- XYZ groups may stack to 3 rows instead of 1

## Copy Scenario Workflow

The collapsible section design works well with copy:
1. User clicks "Copy" in the header
2. New scenario is created with all values duplicated
3. User is navigated to the new scenario's editor
4. All sections are in the same expand/collapse state
5. User adjusts the specific parameters they want to change

This is better than a wizard because there's no forced linear flow -- users jump directly to the section they need.
