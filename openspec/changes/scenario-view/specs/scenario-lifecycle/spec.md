## ADDED Requirements

### Requirement: Scenarios can be copied to produce a new independent scenario

The system SHALL support creating a copy of an existing scenario. The copy SHALL receive a new unique identifier and have configuration values identical to the source. The source scenario SHALL remain unmodified. The copy SHALL start in the Planning state regardless of the source scenario's status.

#### Scenario: Copied scenario starts in Planning status

- **WHEN** a scenario in any lifecycle state is copied
- **THEN** the resulting scenario has Planning status

#### Scenario: Copying does not affect the source scenario

- **WHEN** a scenario is copied
- **THEN** the source scenario's status, configuration, and identifier are unchanged

#### Scenario: Copy preserves all configuration fields

- **WHEN** a scenario is copied
- **THEN** the new scenario's configuration compares equal to the source configuration in all fields

### Requirement: Planning scenarios can be deleted

The system SHALL support permanently removing a scenario that is in the Planning status. Scenarios in any other status SHALL NOT be deletable. Deletion is irreversible.

#### Scenario: Deleting a Planning scenario removes it

- **WHEN** a scenario in Planning status is deleted
- **THEN** the scenario no longer exists and cannot be retrieved

#### Scenario: Deleting a non-Planning scenario is rejected

- **WHEN** an attempt is made to delete a scenario that is Scheduled, Running, or Done
- **THEN** the operation returns an error and the scenario is unchanged
