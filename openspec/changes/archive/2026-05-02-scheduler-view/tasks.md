## 1. Scheduler View Shell

- [x] 1.1 Create `src/ui/bevy_shell/scheduler/` with a Scheduler view plugin, root marker, and spawn/despawn systems under `UiState::Scheduler`
- [x] 1.2 Register the Scheduler view plugin in `src/ui/bevy_shell/mod.rs`
- [x] 1.3 Spawn a fixed dashboard header and a scrollable body under the shared `ContentSlot`

## 2. Scheduler Dashboard Content

- [x] 2.1 Add summary UI that shows scheduler state, running count, scheduled count, and aggregate queue remaining time
- [x] 2.2 Add Start and Stop controls in the Scheduler view with enabled states that match the scheduler state machine
- [x] 2.3 Add a runtime concurrency control in the Scheduler view bound to the existing scheduler job-limit resource

## 3. Running And Scheduled Sections

- [x] 3.1 Render separate Running and Scheduled sections with running scenarios listed first
- [x] 3.2 Show per-running-scenario identifier, status, epoch progress, and existing per-scenario ETA
- [x] 3.3 Show per-scheduled-scenario identifier, status, total queued epochs, and predicted ETA when observed running pace exists
- [x] 3.4 Show explicit unknown-state copy when queue ETA or scheduled-scenario ETA cannot yet be derived

## 4. Queue ETA Projection

- [x] 4.1 Implement helper logic that derives observed time per epoch from currently running scenarios with non-zero progress and timing data
- [x] 4.2 Use the observed epoch rate to estimate aggregate remaining time for all running and scheduled work
- [x] 4.3 Ensure ETA projection remains observational and does not mutate scheduler or scenario execution state

## 5. Legacy Control Cleanup And Verification

- [x] 5.1 Remove duplicated scheduler Start, Stop, and job-limit controls from the legacy egui top bar
- [x] 5.2 Run `just fmt`
- [x] 5.3 Run `cargo check`
- [x] 5.4 Manually verify the Scheduler view in Bevy mode, including summary counts, control enablement, running rows, scheduled rows, and unknown ETA behavior
