## Purpose

Defines the layout, interaction model, and behavioural constraints of the Scenario editor view — the primary surface for inspecting and editing a single scenario's configuration, monitoring its lifecycle status, and invoking lifecycle actions such as scheduling, copying, and deletion.

## ADDED Requirements

### Requirement: Scenario editor organizes parameters into three tabs

The scenario editor SHALL present configuration parameters grouped into three top-level tabs: Simulation, Algorithm, and Model. Only the parameters belonging to the active tab SHALL be visible. Switching tabs SHALL NOT discard any unsaved changes in the inactive tabs.

#### Scenario: Switching tabs preserves unsaved parameter edits

- **WHEN** a user edits a parameter on the Simulation tab and then switches to the Algorithm tab
- **THEN** returning to the Simulation tab shows the edited value unchanged

#### Scenario: All parameters are reachable via tabs

- **WHEN** a user navigates through all three tabs
- **THEN** every configurable parameter of a scenario is accessible without scrolling outside of tab bodies

### Requirement: Scenario editor has a persistent header bar

The scenario editor SHALL display a header bar that is always visible regardless of which tab is active or how far the user has scrolled within a tab. The header bar SHALL show the scenario identifier, the current lifecycle status, a model-type selector, action buttons, and a comment field.

#### Scenario: Header bar remains visible during scroll

- **WHEN** a user scrolls deep into a long tab body
- **THEN** the scenario identifier, status, and action buttons remain visible at the top of the view

#### Scenario: Comment field auto-saves on focus loss

- **WHEN** a user edits the comment field and clicks or tabs away
- **THEN** the comment is persisted without requiring an explicit save action

### Requirement: Parameters are organized into collapsible sections within each tab

Within each tab, parameters SHALL be grouped into named sections. Each section SHALL be independently expandable and collapsible. The collapsed state of each section SHALL be preserved for the duration of the session.

#### Scenario: First section in each tab starts expanded

- **WHEN** a user opens the scenario editor for the first time in a session
- **THEN** the first section of the active tab is expanded and all other sections are collapsed

#### Scenario: Section collapse state is preserved across tab switches

- **WHEN** a user collapses a section on the Simulation tab, switches to the Algorithm tab, and then returns to the Simulation tab
- **THEN** the previously collapsed section is still collapsed

#### Scenario: Clicking the section header toggles expand/collapse

- **WHEN** a user clicks anywhere on a collapsed section header
- **THEN** the section body expands and the chevron indicator rotates to the expanded position

### Requirement: Parameter rows display label, control, and current value

Each configurable parameter SHALL be presented as a horizontal row containing a human-readable label, an interactive control widget, and the current value with its unit. The label, control, and value SHALL be visually distinct and consistently aligned across all rows.

#### Scenario: Slider rows show current value next to the slider

- **WHEN** a user views a parameter row that uses a slider control
- **THEN** the current numeric value and unit are displayed to the right of the slider track

#### Scenario: Changing a slider updates the displayed value immediately

- **WHEN** a user drags a slider thumb to a new position
- **THEN** the value display updates in real time as the thumb moves

### Requirement: Parameter descriptions are accessible via tooltip

Each parameter label SHALL support a hover tooltip that displays the parameter description. The tooltip SHALL appear after a short hover delay and disappear when the cursor leaves the label area.

#### Scenario: Hovering a parameter label shows its description

- **WHEN** a user hovers the cursor over a parameter label for longer than the hover delay
- **THEN** a tooltip containing the parameter description appears near the label

#### Scenario: Moving cursor away from label hides the tooltip

- **WHEN** a tooltip is visible and the user moves the cursor away from the parameter label
- **THEN** the tooltip disappears

### Requirement: Editor controls are disabled when scenario is not in Planning status

When a scenario is in any status other than Planning, all parameter controls in the editor SHALL be visually muted and unresponsive to interaction. The header bar SHALL remain interactive, with appropriate action buttons shown or hidden based on status (e.g., Save is hidden for non-Planning scenarios, Schedule becomes Unschedule).

#### Scenario: Attempting to edit a parameter in a non-Planning scenario has no effect

- **WHEN** a scenario is in the Running status and a user attempts to move a slider
- **THEN** the slider does not move and the configuration value is not changed

#### Scenario: Header actions adapt to scenario status

- **WHEN** a scenario is in the Scheduled status
- **THEN** the header bar shows an Unschedule button instead of a Schedule button, and the Save button is not visible

### Requirement: Conditionally visible parameters are hidden when irrelevant

Parameters that only apply under specific conditions (e.g., a geometry-specific sub-option) SHALL be hidden when their condition is not met. Hidden parameters SHALL take no vertical space in the layout.

#### Scenario: Sensors-per-axis parameter is hidden for non-grid geometries

- **WHEN** the sensor geometry is set to a geometry that does not use a grid arrangement
- **THEN** the sensors-per-axis parameter row is not visible

#### Scenario: Sensors-per-axis parameter appears when grid geometry is selected

- **WHEN** the sensor geometry is changed to a grid arrangement type
- **THEN** the sensors-per-axis parameter row becomes visible without requiring a page reload or navigation

### Requirement: Copy Scenario creates a duplicate and navigates to it

Activating the Copy action from the header bar SHALL create a new scenario with all configuration values copied from the current scenario, assign it a new unique identifier, and navigate the user directly to the new scenario's editor.

#### Scenario: Copied scenario has identical configuration values

- **WHEN** a user activates the Copy action on a scenario
- **THEN** the newly created scenario has configuration values equal to the source scenario in all fields

#### Scenario: Navigation moves to the new scenario after copy

- **WHEN** a copy operation completes
- **THEN** the editor view shows the newly created scenario, not the original

### Requirement: Delete Scenario requires confirmation before removal

Activating the Delete action from the header bar SHALL present a confirmation prompt. The scenario SHALL be deleted and the user navigated away only after explicit confirmation. Dismissing the prompt SHALL leave the scenario unchanged.

#### Scenario: Dismissing the confirmation leaves the scenario intact

- **WHEN** a user activates Delete and then dismisses the confirmation prompt
- **THEN** the scenario is not deleted and the editor remains on the same scenario

#### Scenario: Confirming deletion removes the scenario and navigates away

- **WHEN** a user activates Delete and confirms the prompt
- **THEN** the scenario is removed and the user is navigated to the Explorer view
