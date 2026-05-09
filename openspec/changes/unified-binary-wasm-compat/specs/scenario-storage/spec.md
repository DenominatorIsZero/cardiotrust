## MODIFIED Requirements

### Requirement: Scenario persistence is scoped to the selected project

The system SHALL scope each scenario to the currently selected project. On builds with durable local project storage, scenario metadata, payload, generated result assets, animation frames, and exports SHALL remain associated with that scenario within that project. On builds without durable local project storage, the selected bundled project SHALL still define the scenario collection and isolate it from other bundled projects for the duration of the session.

#### Scenario: Two projects with the same scenario identifier remain isolated
- **WHEN** two different projects each contain a scenario with the same identifier
- **THEN** opening one project SHALL expose only that project's metadata and generated assets for that scenario, never assets from the other project

#### Scenario: Generated outputs belong to the selected scenario in the selected project
- **WHEN** the user generates result images, animation frames, or exports for a scenario
- **THEN** those outputs SHALL remain associated with that scenario within the currently selected project

#### Scenario: Bundled web projects remain isolated within the session
- **WHEN** the user activates one bundled web project and later activates a different bundled web project
- **THEN** scenarios and generated outputs from the first bundled project SHALL NOT appear in the second bundled project's scenario collection

### Requirement: Stored scenario operations persist against the active project

Creating, saving, copying, and deleting a scenario SHALL operate against the active project's scenario collection. A copied scenario SHALL become a new independent scenario in the same project with a new identifier and Planning status. On durable-storage builds, these operations SHALL persist across future project reloads. On session-scoped builds, these operations SHALL remain available for the active session even if they are not guaranteed to survive a page reload.

#### Scenario: New scenario is persisted into the active project
- **WHEN** the user creates a scenario while a project is open
- **THEN** the new scenario SHALL be added to that active project's scenario collection

#### Scenario: Copy creates an independent stored scenario in the same project
- **WHEN** the user copies an existing scenario
- **THEN** the system SHALL create a second scenario in the same project with a new identifier, Planning status, and metadata derived from the source scenario

#### Scenario: Delete removes the stored scenario from the active project
- **WHEN** the user deletes a deletable scenario
- **THEN** that scenario SHALL be removed from the active project's scenario collection

#### Scenario: Session-scoped project changes remain visible during the web session
- **WHEN** the user creates, copies, edits, runs, or deletes scenarios inside a bundled web project
- **THEN** those changes SHALL remain visible everywhere that project is used until the session ends or another bundled project is activated

## ADDED Requirements

### Requirement: Durable exports are only guaranteed where stable project storage exists

The system SHALL guarantee durable result exports only on builds that provide stable project storage. On builds without stable project storage, the system SHALL keep scenario execution and result viewing available without promising durable exported files.

#### Scenario: Native project retains durable exports
- **WHEN** the user creates exports while working in a project with durable storage
- **THEN** those exported outputs SHALL remain associated with that project for later access

#### Scenario: Web project does not promise durable exports
- **WHEN** the user works inside a bundled web project
- **THEN** the system SHALL NOT claim that exported files will remain available after the current session ends
