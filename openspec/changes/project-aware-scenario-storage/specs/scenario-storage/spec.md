## ADDED Requirements

### Requirement: Scenario persistence is scoped to the selected project

The system SHALL persist each scenario inside the currently selected project as its own dedicated storage area. Scenario metadata, payload, generated result assets, animation frames, and exports SHALL remain associated with that scenario within that project. Scenarios with the same identifier in different projects SHALL remain isolated from each other.

#### Scenario: Two projects with the same scenario identifier remain isolated
- **WHEN** two different projects each contain a scenario with the same identifier
- **THEN** opening one project SHALL expose only that project's metadata and generated assets for that scenario, never assets from the other project

#### Scenario: Generated outputs belong to the selected scenario in the selected project
- **WHEN** the user generates result images, animation frames, or exports for a scenario
- **THEN** those outputs SHALL be written into that scenario's persisted storage area within the currently selected project

### Requirement: Project loading is metadata-only and tolerant of missing payload files

When a project is opened, the system SHALL load scenario metadata without eagerly loading persisted payload. Missing or unreadable payload files SHALL NOT prevent a scenario with valid metadata from appearing in the loaded scenario list.

#### Scenario: Done scenario with missing payload still appears in the project list
- **WHEN** a stored scenario's metadata is readable and indicates a Done state, but one or both payload files are missing
- **THEN** the scenario SHALL still appear in the loaded project list with Done status

#### Scenario: Opening a project does not require eager payload loading
- **WHEN** a project containing one or more valid scenarios is opened
- **THEN** the scenario list SHALL load without requiring payload data to be loaded for every entry

### Requirement: Payload loading is explicit and all-or-nothing

The system SHALL load persisted payload only when explicitly requested for a scenario. A payload load SHALL require both persisted data and persisted results; if either is missing or corrupted, the load SHALL fail and SHALL NOT partially succeed.

#### Scenario: Explicit payload load succeeds only when both payload parts are valid
- **WHEN** payload is requested for a scenario and both persisted data and persisted results are present and readable
- **THEN** the request SHALL succeed and make both payload parts available together

#### Scenario: Missing one payload part fails the whole load
- **WHEN** payload is requested for a scenario and either persisted data or persisted results is missing
- **THEN** the request SHALL fail and SHALL NOT expose a partially loaded payload

#### Scenario: Corrupted one payload part fails the whole load
- **WHEN** payload is requested for a scenario and either persisted data or persisted results is corrupted
- **THEN** the request SHALL fail and SHALL NOT expose a partially loaded payload

### Requirement: Stored scenario operations persist against the active project

Creating, saving, copying, and deleting a scenario SHALL operate against the active project's persisted scenario collection. A copied scenario SHALL become a new independent stored scenario in the same project with a new identifier and Planning status.

#### Scenario: New scenario is persisted into the active project
- **WHEN** the user creates a scenario while a project is open
- **THEN** the new scenario SHALL be stored in that active project and SHALL be present when that project is loaded again later

#### Scenario: Copy creates an independent stored scenario in the same project
- **WHEN** the user copies an existing scenario
- **THEN** the system SHALL persist a second scenario in the same project with a new identifier, Planning status, and metadata derived from the source scenario

#### Scenario: Delete removes the stored scenario from the active project
- **WHEN** the user deletes a deletable scenario
- **THEN** that scenario's persisted storage area SHALL be removed from the active project and the scenario SHALL no longer appear when the project is loaded again

### Requirement: Result and export work uses storage-resolved scenario outputs

Before generation or export work begins, the system SHALL resolve the target output file or directory for the selected scenario. Later visits to that same scenario in that same project SHALL be able to discover previously generated assets from those persisted outputs.

#### Scenario: Animation frames are rediscovered for the same stored scenario
- **WHEN** a scenario already has generated animation frames and the user later reopens the same project and scenario
- **THEN** the system SHALL be able to discover and reuse those persisted frames without requiring regeneration

#### Scenario: Exported outputs stay associated with the scenario that created them
- **WHEN** the user completes an export for a scenario
- **THEN** the resulting exported files SHALL remain associated with that scenario's persisted output area for later access
