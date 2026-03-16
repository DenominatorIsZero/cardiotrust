## Purpose

Governs view routing for the Bevy UI backend — which views exist, how navigation between them is controlled, precondition guards that restrict access, and keyboard shortcuts for navigation.

## Requirements

### Requirement: Six named views are available when the Bevy UI backend is active

The application SHALL support six views under the Bevy backend: Home, Explorer, Scenario, Results, Volumetric, and Scheduler. Exactly one of these views is active at any time. The default view on entering the Bevy backend SHALL be Home.

#### Scenario: Only one view is active at any time under the Bevy backend

- **WHEN** the Bevy UI backend is active
- **THEN** exactly one of the six views is designated as the current active view

#### Scenario: Home is the initial view when switching to the Bevy backend

- **WHEN** the Bevy UI backend is selected and no prior view navigation has occurred
- **THEN** the active view is Home

### Requirement: View transitions respect precondition guards

Navigation to Scenario SHALL be disabled when no scenario is selected. Navigation to Results or Volumetric SHALL be disabled when the selected scenario has not completed execution. All other views are always accessible when the Bevy backend is active.

#### Scenario: Scenario view is inaccessible without a selected scenario

- **WHEN** no scenario is selected and the user attempts to navigate to the Scenario view
- **THEN** the navigation is rejected and the active view does not change

#### Scenario: Results view is inaccessible when the scenario is not done

- **WHEN** the selected scenario has not completed execution and the user attempts to navigate to Results
- **THEN** the navigation is rejected and the active view does not change

#### Scenario: Volumetric view is inaccessible when the scenario is not done

- **WHEN** the selected scenario has not completed execution and the user attempts to navigate to Volumetric
- **THEN** the navigation is rejected and the active view does not change

### Requirement: Keyboard shortcuts 1–6 navigate to views

Pressing digit keys 1 through 6 SHALL attempt to navigate to Home, Explorer, Scenario, Results, Volumetric, and Scheduler respectively, subject to the same precondition guards that govern sidebar navigation.

#### Scenario: Digit key navigates to the corresponding view

- **WHEN** the user presses a digit key (1–6) and the target view's preconditions are satisfied
- **THEN** the application navigates to the corresponding view

#### Scenario: Digit key is a no-op when preconditions are not met

- **WHEN** the user presses a digit key for a view whose preconditions are not satisfied
- **THEN** the active view does not change

### Requirement: The Escape key navigates to the parent view

Pressing Escape SHALL navigate from the current view to its logical parent: Results → Scenario, Scenario → Explorer, Volumetric → Scenario. For views with no parent (Home, Explorer, Scheduler), Escape navigates to Home.

#### Scenario: Escape from Results goes to Scenario

- **WHEN** the active view is Results and the user presses Escape
- **THEN** the active view becomes Scenario

#### Scenario: Escape from Scenario goes to Explorer

- **WHEN** the active view is Scenario and the user presses Escape
- **THEN** the active view becomes Explorer

#### Scenario: Escape from Volumetric goes to Scenario

- **WHEN** the active view is Volumetric and the user presses Escape
- **THEN** the active view becomes Scenario

#### Scenario: Escape from a top-level view goes to Home

- **WHEN** the active view is Explorer, Scheduler, or Home and the user presses Escape
- **THEN** the active view becomes Home
