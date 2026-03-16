### Requirement: The Home view is the initial view of the application

On application start the active view SHALL be the Home view. No project SHALL be loaded and no scenario SHALL be selected until the user explicitly opens a project from the Home view.

#### Scenario: App starts on Home view

- **WHEN** the application is launched for the first time with no prior session
- **THEN** the Home view is the first view rendered and no project is loaded

#### Scenario: App starts on Home view even when recent projects exist

- **WHEN** the application is launched and one or more recent projects are recorded
- **THEN** the Home view is still displayed; the user must explicitly select a project to proceed

### Requirement: Home view displays a title area

The Home view SHALL display the application name and a subtitle at the top of the content area.

#### Scenario: Title and subtitle are visible on Home view

- **WHEN** the Home view is active
- **THEN** the application name "CardioTrust" and the subtitle "Cardiac Electrophysiological Simulation" are visible

### Requirement: Home view provides an Open Project action on native builds

On native (non-WASM) builds, the Home view SHALL provide a control that opens the platform's folder-selection dialog. The user-selected folder becomes the active project.

#### Scenario: Clicking Open Project opens a folder dialog

- **WHEN** the user activates the Open Project control on a native build
- **THEN** the platform folder-selection dialog is presented

#### Scenario: Selecting a folder loads it as the active project

- **WHEN** the user selects a folder from the dialog
- **THEN** scenarios are loaded from that folder, the folder is recorded as the active project, and the application transitions to the Explorer view

#### Scenario: Cancelling the dialog has no effect

- **WHEN** the user dismisses the folder-selection dialog without choosing a folder
- **THEN** the current project state is unchanged and the Home view remains active

### Requirement: Home view displays recent projects

The Home view SHALL display a list of previously opened project folders (up to 8 entries). Each entry can be activated to open that project immediately.

#### Scenario: Recent projects list shows previously opened folders

- **WHEN** the Home view is displayed and one or more folders have been opened in prior sessions
- **THEN** each folder appears as a clickable entry in the Recent Projects list

#### Scenario: Activating a recent project loads it

- **WHEN** the user activates a recent project entry
- **THEN** scenarios are loaded from that folder and the application transitions to Explorer

#### Scenario: Recent projects list is empty on first launch

- **WHEN** the Home view is displayed for the first time with no prior session
- **THEN** the Recent Projects list is empty

### Requirement: Home view shows placeholder demo project cards on WASM builds

On WASM builds, the Home view SHALL display placeholder cards representing bundled demo projects. These cards SHALL be visible but non-functional in this phase.

#### Scenario: Demo project cards are visible on WASM

- **WHEN** the application is running as a WASM build and the Home view is active
- **THEN** at least one demo project card is displayed

#### Scenario: Demo project cards are not shown on native builds

- **WHEN** the application is running as a native build
- **THEN** no demo project cards are displayed

### Requirement: Opening a project replaces the current project

If a project is already loaded, opening a new project from the Home view SHALL fully replace the existing project: all loaded scenarios and any selected scenario are discarded before the new project is loaded.

#### Scenario: Opening a second project clears the first

- **WHEN** a project is already loaded and the user opens a different folder from the Home view
- **THEN** all previously loaded scenarios are removed, no scenario is selected, and scenarios from the new folder are loaded

#### Scenario: Opening the same folder reloads it

- **WHEN** a project is already loaded and the user opens the same folder again
- **THEN** the scenario list is reloaded from disk, reflecting any changes made outside the app
