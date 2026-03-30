## 1. Module Scaffold and Routing

- [x] 1.1 Create `src/ui/bevy_shell/scenario/` directory with `mod.rs` declaring a `ScenarioViewPlugin`
- [x] 1.2 Register `ScenarioViewPlugin` in `src/ui/bevy_shell/mod.rs`
- [x] 1.3 Add `ScenarioViewState` resource: `active_tab: ScenarioTab`, `section_collapsed: HashMap<SectionId, bool>`
- [x] 1.4 Wire routing in `src/ui/bevy_shell/routing.rs` or content area: when `UiState::Scenario` + `UiType::Bevy` is entered, spawn the scenario view root into the content slot; despawn on exit
- [x] 1.5 Verify the view mounts and unmounts cleanly (no leftover entities, no panics)

## 2. Backend Events for Copy and Delete

- [x] 2.1 Define `CopyScenarioEvent { source_id: ScenarioId }` and `DeleteScenarioEvent { scenario_id: ScenarioId }` in `src/core/`
- [x] 2.2 Implement `handle_copy_scenario` system: clones config, creates new scenario in Planning state with new unique ID, inserts into project state
- [x] 2.3 Implement `handle_delete_scenario` system: removes scenario from project state if it is in Planning status, returns error otherwise
- [x] 2.4 Add unit tests for copy (source unchanged, new ID, Planning status) and delete (Planning succeeds, non-Planning returns error)

## 3. Scenario Header Bar

- [x] 3.1 Spawn the header bar node: full-width `FlexRow`, `BG1` background, with left and right sections
- [x] 3.2 Left section: scenario ID `Text` node (`FG0`), status badge pill (reuse Explorer card style), model-type `ComboBox` (disabled when not Planning)
- [x] 3.3 Right section: Save button (`AQUA` accent, hidden when not Planning), Copy button, Delete button (`RED`), Schedule/Unschedule toggle button (`ORANGE` when scheduling), Comment `TextInput` field
- [x] 3.4 Implement comment auto-save on focus-loss observer
- [x] 3.5 Implement Schedule/Unschedule button observer: emits appropriate lifecycle event
- [x] 3.6 Implement Save button observer: persists current configuration
- [x] 3.7 Implement Copy button observer: emits `CopyScenarioEvent`, listens for completion event, navigates to new scenario
- [x] 3.8 Implement Delete button observer: shows confirmation modal overlay; on confirm emits `DeleteScenarioEvent` and navigates to Explorer; on dismiss closes modal
- [x] 3.9 Add system to update header bar node contents reactively when scenario status changes

## 4. Tab Bar

- [x] 4.1 Spawn tab bar node below the header: horizontal `FlexRow`, tabs for Simulation / Algorithm / Model
- [x] 4.2 Style active tab: `ORANGE` 3px bottom border, `FG0` text; inactive: no border, `GREY1` text; hover: `FG1` text
- [x] 4.3 Implement tab click observer: updates `ScenarioViewState.active_tab`, shows/hides tab body nodes via `Display::Flex` / `Display::None`

## 5. Collapsible Section Widget

- [x] 5.1 Create `spawn_section(parent, section_id, title, default_expanded)` function: spawns section header row + body container node
- [x] 5.2 Section header: `BG1` background, icon placeholder (colored rectangle), `FG0` title text, chevron `Text` (`v` / `>`)
- [x] 5.3 Section body: `BG0` background, 16px horizontal / 12px vertical padding, `Display::Flex` column
- [x] 5.4 Implement section header click observer: toggles `ScenarioViewState.section_collapsed[section_id]`, updates body `Display` and chevron character
- [x] 5.5 Initialize first section of each tab as expanded, all others collapsed

## 6. Parameter Row and Control Widgets

- [x] 6.1 Create `spawn_param_row(parent, label, unit, tooltip_text)` function: horizontal row with label node (200px), control slot node (flex grow), value+unit node (80px right-aligned)
- [x] 6.2 Implement `SliderWidget`: track node (`BG3`), filled portion (`ORANGE`), draggable thumb (circle, `FG0` fill); drag observer updates config value and value display text
- [x] 6.3 Implement `ComboBoxWidget`: button showing current selection + dropdown overlay node with options; click observer updates config value
- [x] 6.4 Implement `CheckboxWidget`: square node with checkmark toggle; click observer updates bool config value
- [x] 6.5 Implement `NumberInputWidget` (DragValue equivalent): text node that accepts click-drag or direct text entry; used for coordinate inputs
- [x] 6.6 Implement `XyzGroupWidget`: three `NumberInputWidget` nodes in a row with X/Y/Z labels
- [x] 6.7 Implement `TextInputWidget`: single-line text node for path inputs (MRI model path)
- [x] 6.8 Add `TooltipTarget` component to parameter label nodes storing the description string
- [x] 6.9 Implement shared tooltip overlay node (absolute, high z-index): hover-enter observer on `TooltipTarget` nodes starts a 500ms timer, shows tooltip at cursor position; hover-exit hides it

## 7. Simulation Tab Content

- [x] 7.1 Spawn "Core Setup" section: Sample Rate slider (1000–48000 Hz), Duration slider (0.1–60 s)
- [x] 7.2 Spawn "Sensor Configuration" section: Geometry combo, Motion combo, 3D-sensors checkbox, Array Origin XYZ group, Sensors per axis (conditional on grid geometry), Size / Radius / Count inputs, Motion Range / Steps inputs
- [x] 7.3 Implement conditional visibility for Sensors per axis: `Display::None` when geometry is not grid-type; `Display::Flex` when it is
- [x] 7.4 Spawn "Measurement Data" section: Covariance Mean slider (log scale), Covariance Std slider; collapsed by default

## 8. Algorithm Tab Content

- [x] 8.1 Spawn "Algorithm Settings" section (expanded by default): Algorithm type combo (ModelBased/GPU/PseudoInverse), Epochs slider, Batch Size slider, Freeze Gains checkbox, Freeze Delays checkbox
- [x] 8.2 Spawn "Optimizer Settings" section: Optimizer type combo, Learning Rate slider (log scale), LR Reduction Interval slider, LR Reduction Factor slider
- [x] 8.3 Spawn "Regularization Settings" section: Threshold slider, Strength slider
- [x] 8.4 Spawn "Metrics Settings" section: Snapshot Interval slider

## 9. Model Tab Content

- [x] 9.1 Spawn "Heart Geometry" section (expanded by default): Voxel Size slider, Heart Offset XYZ group, Heart Size XYZ group
- [x] 9.2 Spawn "Functional Settings" section: Control Function combo, Pathological checkbox, Current Factor slider
- [x] 9.3 Spawn "Propagation Velocity" section: per-tissue sliders (SA, Atrium, AV, HPS, Ventricle, Pathological)
- [x] 9.4 Spawn "Handcrafted Model" section (conditional: visible only when model type is Handcraft): SA center, AV/HPS toggles and position inputs, pathology region sliders
- [x] 9.5 Spawn "MRI Model" section (conditional: visible only when model type is MRI): path TextInput
- [x] 9.6 Implement conditional section visibility: show/hide Handcrafted or MRI section based on model-type selector in header

## 10. Disabled State Enforcement

- [x] 10.1 Add a system that runs when scenario status changes: sets all parameter row control nodes to a "disabled" marker component when not Planning
- [x] 10.2 In each widget's interaction observer, check for the disabled marker and early-return without modifying config values
- [x] 10.3 Apply visual muting (reduced opacity or `GREY1` text) to control widgets when disabled

## 11. Polish and Verification

- [x] 11.1 Verify all ~40 parameters are present and mapped to the correct tabs and sections per the design doc
- [x] 11.2 Verify that switching tabs does not lose unsaved parameter edits
- [x] 11.3 Verify collapse state persists across tab switches within a session
- [x] 11.4 Run `just test` and fix any failures introduced by the new backend events
- [x] 11.5 Run `just lint` and fix any missing `#[tracing::instrument]` attributes on new public functions
- [x] 11.6 Run `just fmt` to apply import formatting
