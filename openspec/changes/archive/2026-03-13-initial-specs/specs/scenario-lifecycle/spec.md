## Purpose

Governs the scenario as the top-level unit of experimental work — its lifecycle state machine, configuration locking, pre-run unification of simulation and estimation parameters, execution dispatch, result persistence, and progress observability. A scenario is the container that binds a configuration to a run and its outcomes.

This spec is distinct from `scheduler` (which governs how multiple scenarios are queued and executed concurrently across the application) and from `configuration` (which governs the parameter values a scenario holds). Scenario lifecycle concerns the progression of a single experiment from creation to completion; the scheduler concerns fleet-level coordination of many experiments.

## ADDED Requirements

### Requirement: Scenario lifecycle follows a strict state machine

A scenario SHALL transition through a defined sequence of lifecycle states. Attempting an invalid transition SHALL produce an error rather than silently mutating the scenario into an inconsistent state. The valid forward progression is: Planning → Scheduled → Simulating → Running → Done. Abortion is reachable from any running state.

#### Scenario: Scheduling a Planning scenario succeeds

- **WHEN** a scenario in the Planning state is scheduled
- **THEN** its status becomes Scheduled and the operation succeeds

#### Scenario: Scheduling a non-Planning scenario fails

- **WHEN** a scenario that is already Scheduled, Running, or Done is scheduled again
- **THEN** the operation returns an error and the scenario status is unchanged

#### Scenario: Unscheduling a Scheduled scenario returns it to Planning

- **WHEN** a Scheduled scenario is unscheduled
- **THEN** its status becomes Planning

#### Scenario: Unscheduling a non-Scheduled scenario fails

- **WHEN** an attempt is made to unschedule a scenario that is not in the Scheduled state
- **THEN** the operation returns an error and the scenario status is unchanged

### Requirement: Configuration is locked once a scenario leaves Planning

A scenario's configuration SHALL be modifiable only while the scenario is in the Planning state. Once the scenario is Scheduled, Running, or Done, all configuration fields are read-only. This ensures that a running or completed experiment has an immutable, auditable configuration.

#### Scenario: Configuration can be changed during Planning

- **WHEN** a scenario is in the Planning state
- **THEN** any configuration field can be modified

#### Scenario: Configuration cannot be changed after scheduling

- **WHEN** a scenario is in the Scheduled or Running state
- **THEN** no configuration changes are applied and the attempt is blocked

### Requirement: Simulation and estimation configurations are unified before execution

Before a scenario begins execution, the shared parameters (sensor geometry, sample rate, duration) that appear in both the simulation and estimation configurations SHALL be synchronized so both subsystems use identical values. This unification is automatic and happens exactly once, at the start of execution.

#### Scenario: Sensor geometry is consistent between simulation and estimation after unification

- **WHEN** a scenario begins execution
- **THEN** the sensor array geometry used for simulation and the sensor array geometry used for estimation are identical

#### Scenario: Pseudo-inverse mode is forced to a single epoch

- **WHEN** the pseudo-inverse algorithm type is selected and execution begins
- **THEN** the epoch count is set to one, regardless of the value in the original configuration

### Requirement: Execution produces a persisted, self-contained record

When a scenario completes successfully, its configuration, summary metrics, simulation data, and estimation results SHALL be written to stable storage. The scenario is fully recoverable from storage — reloading it at a later time produces a scenario value equivalent to the one at completion.

#### Scenario: Completed scenario can be reloaded from storage

- **WHEN** a Done scenario is saved and then reloaded from storage
- **THEN** the reloaded scenario has the same configuration, status, and summary metrics as the saved one

#### Scenario: Simulation data and estimation results are stored separately from configuration

- **WHEN** the scenario configuration is reloaded from storage
- **THEN** simulation data and estimation results are not eagerly loaded — they are fetched on demand

### Requirement: Progress is observable during execution

While a scenario is in the Running state, callers SHALL be able to observe progress as a fraction of completion. The system SHALL also provide an estimate of the remaining time based on elapsed time and progress so far.

#### Scenario: Progress fraction is between zero and one during execution

- **WHEN** a running scenario's progress is sampled at any point during execution
- **THEN** the returned fraction is in the range [0.0, 1.0]

#### Scenario: Estimated time remaining decreases as execution proceeds

- **WHEN** the estimated time remaining is sampled at two points during a normally progressing run
- **THEN** the second sample is less than or equal to the first

### Requirement: Scenarios have unique, auto-generated identifiers

Every scenario SHALL receive a unique identifier at construction time. When no explicit identifier is provided, the system SHALL generate one that is practically guaranteed to be unique across all scenarios created on the same machine.

#### Scenario: Auto-generated identifiers are unique across concurrent creations

- **WHEN** multiple scenarios are constructed in rapid succession without explicit identifiers
- **THEN** all assigned identifiers are distinct

#### Scenario: Explicit identifier is used when provided

- **WHEN** a scenario is constructed with an explicit identifier
- **THEN** the scenario's identifier equals the provided value
