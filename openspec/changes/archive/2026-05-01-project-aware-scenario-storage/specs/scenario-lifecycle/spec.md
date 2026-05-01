## REMOVED Requirements

### Requirement: Execution produces a persisted, self-contained record
**Reason**: Persisted storage is no longer defined by the lifecycle object itself. Scenario persistence and payload access move to the dedicated scenario-storage capability.
**Migration**: Use the scenario-storage capability for persisted metadata, payload loading, and output discovery while keeping lifecycle state on the scenario metadata object.

## ADDED Requirements

### Requirement: Payload access is explicit and lifecycle-neutral

Accessing persisted payload for a scenario SHALL be an explicit operation separate from loading scenario metadata. If payload loading fails for a scenario, that failure SHALL be reported independently and SHALL NOT change the scenario's lifecycle state recorded in metadata.

#### Scenario: Payload load success does not change lifecycle state
- **WHEN** payload is explicitly loaded for a scenario whose metadata is already present
- **THEN** the scenario's lifecycle state SHALL remain the same as it was before payload loading began

#### Scenario: Done scenario remains Done when payload loading fails
- **WHEN** a scenario's metadata indicates Done and an explicit payload load later fails because persisted payload is missing or corrupted
- **THEN** the scenario SHALL remain in Done state and the payload failure SHALL be treated as a separate error
