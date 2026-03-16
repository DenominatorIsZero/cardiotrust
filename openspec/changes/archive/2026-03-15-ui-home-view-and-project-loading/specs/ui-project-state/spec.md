## ADDED Requirements

### Requirement: The application tracks the currently active project path

The application SHALL maintain a record of which project folder is currently open. This record SHALL be accessible to all UI systems that need to know whether a project is loaded and which folder it refers to. When no project has been opened, the record SHALL reflect that no project is active.

#### Scenario: No project is active at startup

- **WHEN** the application starts
- **THEN** the active project path is absent and the application is in a "no project loaded" state

#### Scenario: Opening a project sets the active path

- **WHEN** the user selects a project folder
- **THEN** the active project path is updated to that folder

#### Scenario: Active path is cleared when the project is unloaded

- **WHEN** the user returns to the Home view and a new project selection has not yet been made
- **THEN** the active project path remains set to the previously opened project (it is not cleared merely by navigating to Home)

### Requirement: Recent project history is persisted across sessions

The application SHALL persist a list of recently opened project folder paths (up to 8 entries, most-recent first). This list SHALL survive application restarts and be restored when the app launches.

#### Scenario: Opening a project adds it to recent history

- **WHEN** the user opens a project folder
- **THEN** that folder path is added to the top of the recent history list

#### Scenario: Duplicate entries are not added

- **WHEN** the user opens a folder that is already in the recent history
- **THEN** the folder is moved to the top of the list rather than appearing twice

#### Scenario: History is capped at eight entries

- **WHEN** more than eight distinct project folders have been opened
- **THEN** only the eight most recently opened are retained in the list

#### Scenario: Recent history survives an app restart

- **WHEN** the application is closed and reopened after opening one or more projects
- **THEN** the recent history list contains the same entries as before the restart

### Requirement: Scenario list is loaded from the active project path

When the active project path is set, the application SHALL load all valid scenarios from that path. The previously loaded scenario list SHALL be discarded entirely before loading begins.

#### Scenario: Scenario list reflects the active project folder

- **WHEN** a project folder is opened
- **THEN** the scenario list contains exactly the scenarios found in that folder

#### Scenario: Previously loaded scenarios are not retained after a project change

- **WHEN** a new project folder is opened while another project was already loaded
- **THEN** scenarios from the previous project no longer appear in the scenario list

#### Scenario: Folders with no valid scenarios result in an empty list

- **WHEN** a project folder containing no valid scenario data is opened
- **THEN** the scenario list is empty and no error prevents navigation to Explorer
