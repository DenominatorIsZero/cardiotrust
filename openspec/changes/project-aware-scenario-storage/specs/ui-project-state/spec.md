## MODIFIED Requirements

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

## MODIFIED Requirements

### Requirement: Scenario list is loaded from the active project path

When a project folder is opened, the application SHALL replace the entire loaded scenario collection with a new collection rooted at that folder. The loaded collection SHALL contain exactly the valid scenario metadata found in that project. Project loading SHALL be metadata-only and SHALL NOT fail solely because a scenario's persisted payload files are missing.

#### Scenario: Scenario list reflects the opened project folder

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
