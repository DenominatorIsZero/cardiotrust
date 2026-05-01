### Requirement: The application tracks the currently active project path

The application SHALL track whether a project is loaded and which project folder it came from as part of the loaded scenario collection rather than as a separate mutable record. This loaded-project record SHALL be accessible to UI systems that need to know whether a project is open. When no project has been opened, the loaded scenario collection SHALL indicate that no project root is present.

#### Scenario: No project is active at startup

- **WHEN** the application starts
- **THEN** the loaded scenario collection SHALL indicate that no project root is present and the application is in a "no project loaded" state

#### Scenario: Opening a project sets the loaded project root

- **WHEN** the user selects a project folder
- **THEN** the loaded scenario collection SHALL record that folder as its project root

#### Scenario: Navigating to Home does not clear the loaded project root

- **WHEN** the user returns to the Home view and a new project selection has not yet been made
- **THEN** the loaded scenario collection SHALL still refer to the previously opened project

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

When a project folder is opened, the application SHALL replace the entire loaded scenario collection with a new collection rooted at that folder. The loaded collection SHALL contain exactly the valid scenario metadata found in that project. Project loading SHALL be metadata-only and SHALL NOT fail solely because a scenario's persisted payload files are missing.

#### Scenario: Scenario list reflects the active project folder

- **WHEN** a project folder is opened
- **THEN** the loaded scenario collection SHALL contain exactly the scenarios found in that folder

#### Scenario: Previously loaded scenarios are not retained after a project change

- **WHEN** a new project folder is opened while another project was already loaded
- **THEN** scenarios from the previous project SHALL no longer appear in the loaded scenario collection

#### Scenario: Folders with no valid scenarios result in an empty list

- **WHEN** a project folder containing no valid scenario metadata is opened
- **THEN** the loaded scenario collection SHALL be empty and no error SHALL prevent navigation to Explorer

#### Scenario: Missing payload files do not block metadata loading

- **WHEN** a project folder contains valid scenario metadata for a scenario whose payload files are missing
- **THEN** that scenario SHALL still appear in the loaded scenario collection
