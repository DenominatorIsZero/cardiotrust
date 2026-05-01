## MODIFIED Requirements

### Requirement: Home view provides an Open Project action on native builds

On native (non-WASM) builds, the Home view SHALL provide a control that opens the platform's folder-selection dialog when project switching is allowed. The user-selected folder becomes the active project. If project switching is currently blocked by active computation, the control SHALL be disabled and SHALL NOT open the dialog.

#### Scenario: Clicking Open Project opens a folder dialog

- **WHEN** the user activates the Open Project control on a native build and project switching is allowed
- **THEN** the platform folder-selection dialog SHALL be presented

#### Scenario: Selecting a folder loads it as the active project

- **WHEN** the user selects a folder from the dialog while project switching is allowed
- **THEN** scenarios SHALL be loaded from that folder and the application SHALL transition to the Explorer view

#### Scenario: Cancelling the dialog has no effect

- **WHEN** the user dismisses the folder-selection dialog without choosing a folder
- **THEN** the current loaded project state SHALL be unchanged and the Home view SHALL remain active

#### Scenario: Open Project control is disabled while switching is blocked

- **WHEN** any loaded scenario is actively Simulating or Running
- **THEN** the Open Project control SHALL be disabled and activating it SHALL NOT open a folder-selection dialog

## MODIFIED Requirements

### Requirement: Home view displays recent projects

The Home view SHALL display a list of previously opened project folders (up to 8 entries). Each entry can be activated to open that project immediately when project switching is allowed. If project switching is blocked by active computation, recent-project entries SHALL be visibly disabled.

#### Scenario: Recent projects list shows previously opened folders

- **WHEN** the Home view is displayed and one or more folders have been opened in prior sessions
- **THEN** each folder SHALL appear as an entry in the Recent Projects list

#### Scenario: Activating a recent project loads it

- **WHEN** the user activates a recent project entry while project switching is allowed
- **THEN** scenarios SHALL be loaded from that folder and the application SHALL transition to Explorer

#### Scenario: Recent projects list is empty on first launch

- **WHEN** the Home view is displayed for the first time with no prior session
- **THEN** the Recent Projects list SHALL be empty

#### Scenario: Recent project entries are disabled while switching is blocked

- **WHEN** any loaded scenario is actively Simulating or Running
- **THEN** recent project entries SHALL be disabled and activating them SHALL NOT switch projects

## MODIFIED Requirements

### Requirement: Opening a project replaces the current project

If a project is already loaded, opening a new project from the Home view SHALL fully replace the existing project when project switching is allowed: all loaded scenarios and any selected scenario are discarded before the new project is loaded. If switching is blocked by active computation, the replacement SHALL NOT begin.

#### Scenario: Opening a second project clears the first

- **WHEN** a project is already loaded and the user opens a different folder from the Home view while project switching is allowed
- **THEN** all previously loaded scenarios SHALL be removed, no scenario SHALL remain selected, and scenarios from the new folder SHALL be loaded

#### Scenario: Opening the same folder reloads it

- **WHEN** a project is already loaded and the user opens the same folder again while project switching is allowed
- **THEN** the scenario list SHALL be reloaded from storage, reflecting any changes made outside the app

#### Scenario: Project replacement is blocked during active computation

- **WHEN** a project is already loaded and any scenario is actively Simulating or Running
- **THEN** attempting to open another project SHALL leave the current project and selection unchanged

## ADDED Requirements

### Requirement: Home view shows an inline reason when project switching is blocked

When project switching is blocked, the Home view SHALL display a clear inline notice explaining that the current project cannot be switched until active computation finishes. Scheduled scenarios alone SHALL NOT trigger this blocked state.

#### Scenario: Running scenario shows blocked-switching notice
- **WHEN** at least one loaded scenario is Running
- **THEN** the Home view SHALL display an inline notice explaining that project switching is unavailable until the active run finishes

#### Scenario: Simulating scenario shows blocked-switching notice
- **WHEN** at least one loaded scenario is Simulating
- **THEN** the Home view SHALL display an inline notice explaining that project switching is unavailable until the active computation finishes

#### Scenario: Scheduled scenarios do not show blocked-switching notice
- **WHEN** one or more scenarios are Scheduled but no scenario is Simulating or Running
- **THEN** the Home view SHALL NOT show the blocked-switching notice and project-open controls SHALL remain enabled
