## Purpose

Governs the concurrent simulation scheduler — the component responsible for dequeuing scheduled scenarios, enforcing a bounded concurrency limit, relaying execution progress to the application without blocking the render loop, and guaranteeing that every launched scenario eventually reaches a terminal state.

This spec is distinct from `scenario-lifecycle` (which governs the internal execution and persistence of a single scenario) and from `ui-navigation` (which governs the user controls that start and stop the scheduler). The scheduler operates at the fleet level: it coordinates when and how many scenarios run simultaneously, but it does not define what happens inside a scenario's execution.

## ADDED Requirements

### Requirement: The scheduler operates in three distinct states

The scheduler SHALL have a Paused state (no scenarios are launched, no execution occurs), an Available state (scheduled scenarios are dequeued and launched up to the concurrency limit), and an Unavailable state (the concurrency limit is reached; no new scenarios are launched, but already-running scenarios continue). Transitions between states are explicit: the user transitions between Paused and Available; the system transitions between Available and Unavailable automatically based on load.

#### Scenario: Paused scheduler launches no new scenarios

- **WHEN** the scheduler is in the Paused state and Scheduled scenarios are present
- **THEN** no new simulation threads are started

#### Scenario: Available scheduler launches Scheduled scenarios

- **WHEN** the scheduler is in the Available state and at least one Scheduled scenario exists below the concurrency limit
- **THEN** that scenario transitions to a running state and begins execution

#### Scenario: Scheduler becomes Unavailable when concurrency limit is reached

- **WHEN** the number of concurrently running scenarios equals the configured maximum
- **THEN** the scheduler transitions to Unavailable and stops launching new scenarios

#### Scenario: Scheduler returns to Available when a running scenario completes

- **WHEN** a running scenario completes and the number of running scenarios drops below the configured maximum
- **THEN** the scheduler transitions back to Available

### Requirement: Every launched scenario eventually reaches a terminal state

The scheduler SHALL guarantee that every scenario it transitions out of Scheduled eventually reaches the Done state, regardless of errors during execution. A scenario that encounters an execution error is still marked as Done rather than left in a perpetually running state.

#### Scenario: Normal completion marks scenario as Done

- **WHEN** a running scenario's simulation completes without errors
- **THEN** the scenario status becomes Done

#### Scenario: Execution error does not leave scenario in a non-terminal state

- **WHEN** an error occurs during a running scenario's simulation
- **THEN** the scenario status is set to Done (or Aborted) and does not remain in a running state

### Requirement: Execution progress is observable without blocking the application loop

The scheduler SHALL relay per-epoch progress and final summary information from background simulation threads to the main application state each cycle. This relay SHALL be non-blocking: if no new progress information is available, the relay completes immediately with no delay.

#### Scenario: Progress updates are visible each application cycle

- **WHEN** a running scenario completes a new epoch
- **THEN** the updated epoch count and summary are reflected in the shared application state before the next render frame

#### Scenario: No progress update causes no delay

- **WHEN** a running scenario has not completed a new epoch since the last cycle
- **THEN** the scheduler's progress-relay step completes immediately without waiting

### Requirement: Concurrency limit is configurable at runtime

The maximum number of simultaneously running scenarios SHALL be a value that can be adjusted at runtime by the user. Changes to the limit take effect for future launch decisions; already-running scenarios are not affected.

#### Scenario: Increasing the limit allows more concurrent scenarios

- **WHEN** the concurrency limit is increased while the scheduler is Available
- **THEN** additional Scheduled scenarios may be launched up to the new limit

#### Scenario: Decreasing the limit does not abort running scenarios

- **WHEN** the concurrency limit is decreased to a value below the current number of running scenarios
- **THEN** already-running scenarios continue to completion and are not interrupted

### Requirement: Completed scenario data is persisted by the scheduler

When the scheduler detects that a simulation thread has finished, it SHALL mark the scenario as Done and then write the scenario's final state (configuration, metrics, and simulation artifacts) to stable storage before releasing its execution resources.

#### Scenario: Scenario is persisted after its status is updated to Done

- **WHEN** a simulation thread completes
- **THEN** the scenario's status is set to Done in application state, and then the scenario data is written to storage
