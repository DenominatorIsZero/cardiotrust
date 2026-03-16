## MODIFIED Requirements

### Requirement: Exactly one content view is active at all times

The application SHALL have exactly one active content view at any time, selected from: Home, Explorer, Scenario, Results, Volumetric, and Scheduler. When the Bevy backend is active, the Home view is the initial view. When the EGui backend is active, the initial view remains Explorer. Switching views is instantaneous. These behaviors apply only when the respective backend is active.

#### Scenario: Only one content panel renders per frame

- **WHEN** the application is in any active view state and the EGUI backend is selected
- **THEN** exactly one content panel occupies the central area of the screen

#### Scenario: Navigation bar is always visible

- **WHEN** the application is in any view state and the EGUI backend is selected
- **THEN** the top navigation bar is rendered

#### Scenario: No EGUI panels render when Bevy backend is selected

- **WHEN** the UI backend selector is set to `Bevy`
- **THEN** no EGUI draw systems execute and no EGUI panels appear on screen

#### Scenario: Bevy backend starts on Home view

- **WHEN** the application is launched with the Bevy backend active
- **THEN** the initial view is Home

### Requirement: Navigation to project-dependent views is disabled when no project is loaded

When the Bevy backend is active and no project has been opened, the Explorer, Scenario, Results, Volumetric, and Scheduler navigation items SHALL be disabled and non-interactive. The Home navigation item SHALL always be enabled.

#### Scenario: Project-dependent sidebar items are disabled with no project

- **WHEN** the Bevy backend is active and no project folder is open
- **THEN** Explorer, Scenario, Results, Volumetric, and Scheduler nav items are disabled and cannot be activated

#### Scenario: Home nav item is always enabled

- **WHEN** the Bevy backend is active, regardless of whether a project is loaded
- **THEN** the Home nav item is enabled and activatable

#### Scenario: Project-dependent items become enabled after a project is opened

- **WHEN** a project folder is opened from the Home view
- **THEN** the Explorer nav item becomes enabled and activatable

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
