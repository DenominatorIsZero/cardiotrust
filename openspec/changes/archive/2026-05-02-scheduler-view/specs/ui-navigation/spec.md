## MODIFIED Requirements

### Requirement: The scheduler can be started and stopped from the navigation bar

The Scheduler view SHALL provide explicit controls to start and stop the background simulation scheduler and to adjust the maximum concurrent job count at runtime. "Start" transitions the scheduler to its active state; "Stop" transitions it to its paused state. Each control is only enabled when the corresponding transition is valid. Changing the concurrent job count affects future launch decisions and does not interrupt already-running scenarios.

#### Scenario: Start is disabled when scheduler is already running

- **WHEN** the scheduler is in its active (non-paused) state
- **THEN** the Start control is disabled

#### Scenario: Stop is disabled when scheduler is already paused

- **WHEN** the scheduler is in its paused state
- **THEN** the Stop control is disabled

#### Scenario: Concurrency limit is adjustable from the Scheduler view

- **WHEN** the user changes the concurrent job count in the Scheduler view
- **THEN** future scheduler launch decisions use the new limit without interrupting already-running scenarios

### Requirement: The Scheduler view presents fleet-level scheduler status and queue progress

The Scheduler view SHALL present a dashboard summary for the current scheduler queue. The summary SHALL show the scheduler state, the number of currently running scenarios, the number of scheduled scenarios waiting to run, and an aggregate remaining-time estimate for work still in the queue when enough observed progress data exists to compute one.

#### Scenario: Scheduler summary reflects the current queue

- **WHEN** the Scheduler view is open and scenarios are running or scheduled
- **THEN** the dashboard summary shows the current scheduler state, running count, and scheduled count

#### Scenario: Aggregate remaining time is shown when observed progress exists

- **WHEN** at least one running scenario has enough observed progress to estimate time per epoch
- **THEN** the Scheduler view shows an aggregate remaining-time estimate for the running and scheduled queue

#### Scenario: Aggregate remaining time is omitted when insufficient data exists

- **WHEN** no running scenario has enough observed progress to estimate time per epoch
- **THEN** the Scheduler view shows that the queue remaining time is unknown rather than displaying a fabricated estimate

### Requirement: The Scheduler view separates running scenarios from scheduled scenarios

The Scheduler view SHALL display currently running scenarios and scheduled scenarios in separate sections, with running scenarios listed before scheduled scenarios.

#### Scenario: Running scenarios appear in the running section

- **WHEN** one or more scenarios are actively executing
- **THEN** those scenarios are displayed in the running section of the Scheduler view

#### Scenario: Scheduled scenarios appear in the scheduled section

- **WHEN** one or more scenarios are waiting for capacity
- **THEN** those scenarios are displayed in the scheduled section of the Scheduler view

### Requirement: Scheduler rows expose progress and remaining-time information

Each running scenario row in the Scheduler view SHALL show the scenario identifier, lifecycle status, epoch progress, and per-scenario remaining-time estimate. Each scheduled scenario row SHALL show the scenario identifier, lifecycle status, total queued epoch count, and a predicted remaining-time estimate derived from the observed time per epoch of currently running scenarios when such data exists.

#### Scenario: Running scenario row shows live progress

- **WHEN** a scenario is in the Running state
- **THEN** its Scheduler row shows live epoch progress and its per-scenario remaining-time estimate

#### Scenario: Scheduled scenario row shows predicted time from observed running pace

- **WHEN** a scenario is in the Scheduled state and the system has observed time-per-epoch data from running scenarios
- **THEN** its Scheduler row shows a predicted remaining-time estimate derived from that observed running pace

#### Scenario: Scheduled scenario row omits prediction when no observed running pace exists

- **WHEN** a scenario is in the Scheduled state and the system has not observed enough running progress to estimate time per epoch
- **THEN** its Scheduler row shows that the predicted remaining time is unknown
