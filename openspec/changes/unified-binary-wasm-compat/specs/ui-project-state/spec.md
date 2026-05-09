## MODIFIED Requirements

### Requirement: The application tracks the currently active project path

The application SHALL track whether a project is loaded and which project it came from as part of the loaded scenario collection rather than as a separate mutable record. On native builds this project identity is a folder path. On builds that expose bundled projects instead of user-selected folders, the loaded scenario collection SHALL still identify which bundled project is active even when no filesystem path exists.

#### Scenario: No project is active at startup
- **WHEN** the application starts
- **THEN** the loaded scenario collection SHALL indicate that no project is active and the application is in a "no project loaded" state

#### Scenario: Opening a native project sets the loaded project root
- **WHEN** the user selects a project folder on a native build
- **THEN** the loaded scenario collection SHALL record that folder as its project identity

#### Scenario: Activating a bundled project sets the active project identity
- **WHEN** the user activates a bundled project on a web build
- **THEN** the loaded scenario collection SHALL record that bundled project as the active project identity

#### Scenario: Navigating to Home does not clear the loaded project identity
- **WHEN** the user returns to the Home view and a new project selection has not yet been made
- **THEN** the loaded scenario collection SHALL still refer to the previously loaded project

### Requirement: Recent project history is persisted across sessions

The application SHALL persist a list of recently opened project folder paths (up to 8 entries, most-recent first) on native builds. This list SHALL survive application restarts and be restored when the app launches. Web builds with bundled projects SHALL NOT promise durable recent-project history across page reloads.

#### Scenario: Opening a project adds it to recent history
- **WHEN** the user opens a project folder on a native build
- **THEN** that folder path is added to the top of the recent history list

#### Scenario: Duplicate entries are not added
- **WHEN** the user opens a folder on a native build that is already in the recent history
- **THEN** the folder is moved to the top of the list rather than appearing twice

#### Scenario: History is capped at eight entries
- **WHEN** more than eight distinct project folders have been opened on a native build
- **THEN** only the eight most recently opened are retained in the list

#### Scenario: History survives an app restart on native
- **WHEN** the application is closed and reopened after opening one or more projects on a native build
- **THEN** the recent history list contains the same entries as before the restart

#### Scenario: Web build starts without durable recent-project history
- **WHEN** the application is reloaded as a web build
- **THEN** the system SHALL NOT require any prior bundled project selection history to be restored
