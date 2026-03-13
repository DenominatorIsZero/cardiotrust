## Purpose

Governs the desktop application's user interface shell — page routing between the four content views (Explorer, Scenario editor, Results, Volumetric), precondition rules for navigation, the scenario list and its live progress display, configuration editing with state-based locking, on-demand result image generation and caching, and scheduler start/stop controls.

This spec is distinct from `visualization` (which governs the 3-D rendering that occupies the Volumetric view) and from `scheduler` (which governs the background execution logic that the UI controls expose). UI navigation covers observable user-facing behavior: what controls exist, when they are enabled, and what transitions they trigger.

## ADDED Requirements

### Requirement: Exactly one content view is active at all times

The application SHALL have exactly one active content view at any time, selected from: Explorer, Scenario, Results, and Volumetric. The top navigation bar is always visible regardless of the active view. Switching views is instantaneous — the outgoing view is fully replaced by the incoming view.

#### Scenario: Only one content panel renders per frame

- **WHEN** the application is in any active view state
- **THEN** exactly one content panel occupies the central area of the screen

#### Scenario: Navigation bar is always visible

- **WHEN** the application is in any view state
- **THEN** the top navigation bar is rendered

### Requirement: Navigation to a view is disabled when preconditions are not met

Navigation buttons SHALL be disabled when the corresponding preconditions are not satisfied. Specifically: "Scenario" requires a selected scenario; "Results" and "Volumetric" additionally require that the selected scenario has completed execution.

#### Scenario: Scenario navigation is disabled without a selection

- **WHEN** no scenario is selected in the Explorer
- **THEN** the Scenario navigation button is disabled and cannot be activated

#### Scenario: Results navigation is disabled for non-completed scenarios

- **WHEN** the selected scenario has not finished execution
- **THEN** the Results navigation button is disabled

#### Scenario: Volumetric navigation is disabled for non-completed scenarios

- **WHEN** the selected scenario has not finished execution
- **THEN** the Volumetric navigation button is disabled

#### Scenario: Navigating to the current view is disabled

- **WHEN** the application is already in a given view
- **THEN** the navigation button for that view is disabled

### Requirement: Simulation data and results are loaded before entering data views

When navigating to the Results or Volumetric views, the selected scenario's simulation data and estimation results SHALL be loaded into memory synchronously before the view is entered. The content views can assume data availability on their first render.

#### Scenario: Data is available when Results view first renders

- **WHEN** the user navigates to the Results view
- **THEN** the scenario's simulation data and estimation results are present in memory before any result images are requested

#### Scenario: Data is available when Volumetric view first renders

- **WHEN** the user navigates to the Volumetric view
- **THEN** the scenario's simulation data and estimation results are present in memory before any 3-D elements are populated

### Requirement: The Explorer lists all known scenarios with their current status

The Explorer view SHALL display all loaded scenarios in a scrollable list. Each entry SHALL show the scenario's identifier, lifecycle status, final performance metrics (when available), and a free-text comment. For running scenarios, the Explorer SHALL show live progress as a fraction of completion with an estimated time remaining.

#### Scenario: Running scenario shows animated progress

- **WHEN** a scenario is in the Running state
- **THEN** the Explorer displays that scenario's completion fraction and estimated time remaining, updating each render frame

#### Scenario: Completed scenario shows final summary metrics

- **WHEN** a scenario is in the Done state
- **THEN** the Explorer displays the scenario's final loss decomposition and segmentation scores

#### Scenario: New scenario is created from Explorer

- **WHEN** the user activates the "New" action in the Explorer
- **THEN** a new scenario in Planning state is added to the list, selected, and the application navigates to the Scenario view

### Requirement: Scenario editor allows configuration changes only during Planning

The Scenario view SHALL display all configuration parameters for the selected scenario. When the scenario is in Planning state, all parameters SHALL be editable. When the scenario is in any other state, all parameter controls SHALL be inert — displaying their current values but not accepting changes.

#### Scenario: All parameter controls are active during Planning

- **WHEN** a scenario in Planning state is viewed in the Scenario editor
- **THEN** all sliders, dropdowns, and text fields accept user input

#### Scenario: All parameter controls are inactive after scheduling

- **WHEN** a scenario in Scheduled, Running, or Done state is viewed in the Scenario editor
- **THEN** all parameter controls display values but do not accept changes

### Requirement: Simulation and algorithm model types are always synchronized in the editor

In the Scenario editor, the anatomy source selection (handcrafted vs. MRI) SHALL apply identically to both the simulation model and the estimation model. Selecting one anatomy source type in the editor updates both models in the same interaction.

#### Scenario: Changing anatomy source updates both simulation and estimation models

- **WHEN** the anatomy source type is switched in the Scenario editor
- **THEN** both the simulation model and the estimation model reflect the new anatomy source type

### Requirement: Result images are generated on demand and cached on disk

The Results view SHALL display generated images for the selected visualization type. Images SHALL be generated asynchronously on first request and cached to disk. Subsequent requests for the same image use the cached version without regenerating. A loading indicator is shown while generation is in progress.

#### Scenario: Image generation shows a loading indicator

- **WHEN** a result image type is selected for the first time
- **THEN** a loading indicator is displayed while the image is being generated

#### Scenario: Previously generated image is shown without regeneration

- **WHEN** a result image type that has already been generated is selected
- **THEN** the cached image is displayed immediately without triggering a new generation

#### Scenario: Changing scenarios resets result image state

- **WHEN** a different scenario is selected
- **THEN** all previously generated image references are cleared and images must be requested again for the new scenario

### Requirement: The scheduler can be started and stopped from the navigation bar

The top navigation bar SHALL provide explicit controls to start and stop the background simulation scheduler. "Start" transitions the scheduler to its active state; "Stop" transitions it to its paused state. Each control is only enabled when the corresponding transition is valid.

#### Scenario: Start is disabled when scheduler is already running

- **WHEN** the scheduler is in its active (non-paused) state
- **THEN** the Start control is disabled

#### Scenario: Stop is disabled when scheduler is already paused

- **WHEN** the scheduler is in its paused state
- **THEN** the Stop control is disabled
