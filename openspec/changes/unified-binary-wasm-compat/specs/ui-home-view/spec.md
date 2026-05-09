## MODIFIED Requirements

### Requirement: Home view displays recent projects

The Home view SHALL display a list of previously opened project folders on native builds only. Each entry can be activated to open that project immediately when project switching is allowed. If project switching is blocked by active computation, recent-project entries SHALL be visibly disabled.

#### Scenario: Recent projects list shows previously opened folders
- **WHEN** the Home view is displayed on a native build and one or more folders have been opened in prior sessions
- **THEN** each folder SHALL appear as an entry in the Recent Projects list

#### Scenario: Activating a recent project loads it
- **WHEN** the user activates a recent project entry on a native build while project switching is allowed
- **THEN** scenarios SHALL be loaded from that folder and the application SHALL transition to Explorer

#### Scenario: Recent projects list is empty on first launch
- **WHEN** the Home view is displayed on a native build for the first time with no prior session
- **THEN** the Recent Projects list is empty

#### Scenario: Recent project entries are disabled while switching is blocked
- **WHEN** any loaded scenario is actively Simulating or Running
- **THEN** recent project entries SHALL be disabled and activating them SHALL NOT switch projects

### Requirement: Home view shows bundled demo project cards on WASM builds

On WASM builds, the Home view SHALL display cards for bundled demo projects. These cards SHALL be functional project selectors: activating one SHALL load that bundled project's scenarios and transition the application to the Explorer view when project switching is allowed.

#### Scenario: Demo project cards are visible on WASM
- **WHEN** the application is running as a WASM build and the Home view is active
- **THEN** at least one bundled demo project card is displayed

#### Scenario: Activating a demo project loads it
- **WHEN** the user activates a bundled demo project card while project switching is allowed on a WASM build
- **THEN** that bundled project's scenarios become the active project and the application transitions to Explorer

#### Scenario: Demo project cards are disabled while switching is blocked
- **WHEN** any loaded scenario is actively Simulating or Running on a WASM build
- **THEN** activating a bundled demo project card SHALL NOT replace the current project

#### Scenario: Demo project cards are not shown on native builds
- **WHEN** the application is running as a native build
- **THEN** bundled demo project cards are not displayed
