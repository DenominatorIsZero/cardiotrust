## Why

The Bevy shell already exposes a Scheduler route, but the content area is empty and the scheduler controls still live in the legacy egui top bar. This makes scheduler control inconsistent with the rest of the Bevy-native UI and hides the fleet-level execution dashboard behind controls that do not show queue state or aggregate completion time.

## What Changes

- Add a Bevy-native Scheduler view that serves as the primary scheduler control dashboard.
- Move the scheduler Start, Stop, and concurrency-limit controls out of the legacy egui top bar and into the Scheduler view.
- Add a scheduler summary header that shows the current scheduler state, the number of running scenarios, the number of scheduled scenarios, and an aggregate remaining-time estimate for the active queue.
- Add separate Running and Scheduled sections in the Scheduler view, with running scenarios listed first.
- Show each running scenario's progress, epoch progress, and per-scenario remaining-time estimate.
- Show each scheduled scenario's predicted remaining time using observed time per epoch from currently running scenarios.
- Preserve existing navigation behavior: the Scheduler route remains project-dependent and stays accessible from the Bevy sidebar and keyboard shortcuts.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `ui-navigation`: The Scheduler view gains defined dashboard content, becomes the primary location for scheduler controls, and displays queue-wide monitoring information.

## Impact

- `src/ui/bevy_shell/` - add a new Scheduler view plugin and node tree.
- `src/ui/bevy_shell/mod.rs` - register the Scheduler view.
- `src/ui/topbar.rs` - remove legacy scheduler controls from the egui top bar.
- `src/core/scenario.rs` and scheduler-facing UI code - reuse existing per-scenario ETA data to derive queue-wide estimates.
- `openspec/specs/ui-navigation/spec.md` - update user-visible scheduler control and Scheduler view behavior.
