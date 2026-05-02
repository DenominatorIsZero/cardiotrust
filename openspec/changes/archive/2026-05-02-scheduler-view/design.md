## Context

CardioTrust's Bevy shell already treats Scheduler as a first-class view in navigation, breadcrumbs, and keyboard shortcuts, but there is no Bevy-native content implementation for that route. At the same time, the legacy egui top bar still exposes Start, Stop, and concurrency controls, which conflicts with the direction of moving view-specific behavior into the Bevy content area.

The scheduler execution model already exists. It tracks a bounded number of running scenarios, a queue of scheduled scenarios, a runtime-adjustable concurrency limit, and per-scenario progress updates. Each running scenario also exposes a remaining-time estimate derived from elapsed time and epoch progress. The new work is therefore primarily a UI composition problem plus a small amount of queue-level projection logic.

## Goals / Non-Goals

**Goals:**
- Add a Bevy-native Scheduler view under the existing `UiState::Scheduler` route.
- Make the Scheduler view the primary place to start and stop the scheduler and adjust concurrency.
- Show fleet-level monitoring: scheduler state, running count, scheduled count, and queue-wide remaining time.
- Show running scenarios and scheduled scenarios in separate sections, with running first.
- Reuse existing per-scenario ETA behavior rather than inventing a second timing model.

**Non-Goals:**
- Changing scheduler execution semantics or state transitions.
- Reworking the Explorer's per-card progress display.
- Adding cancellation, reprioritization, or queue reordering.
- Replacing the egui UI path beyond removing the duplicated scheduler controls from the top bar.

## Decisions

### 1. Implement the Scheduler view as a dedicated Bevy-shell module

Create `src/ui/bevy_shell/scheduler/` following the same spawn/despawn pattern as the existing Bevy views. The plugin registers `OnEnter(UiState::Scheduler)` and `OnExit(UiState::Scheduler)` systems and parents its root node under the shared `ContentSlot`.

This keeps routing and ownership aligned with the rest of the shell. The alternative was to keep scheduler controls in the top bar and render only monitoring content in the view, but that would preserve split ownership and duplicate visible control surfaces.

### 2. Use a sticky dashboard header above a scrollable two-section body

The Scheduler view is structured as:

- dashboard header with queue summary and controls
- Running section
- Scheduled section

The header remains visible while the lists scroll. This preserves access to Start, Stop, and concurrency controls while browsing larger queues.

The alternative was a single unified list with inline controls at the top. That is simpler to spawn, but it makes the distinction between actively executing scenarios and queued scenarios less obvious, which matters for understanding scheduler capacity and ETA.

### 3. Derive queue-wide ETA from observed time per epoch of running scenarios

Per the agreed product behavior, the queue-wide prediction is based on observed time per epoch from currently running scenarios multiplied by the remaining epochs in both the running and scheduled portions of the queue.

The UI computes a normalized observed time-per-epoch for each running scenario that has non-zero progress and timing data. The aggregate estimate uses the average observed seconds per epoch across those scenarios. Running scenarios contribute their remaining epochs; scheduled scenarios contribute their full configured epoch counts.

If no running scenario has enough information to form an observed rate, the queue-wide ETA remains unknown instead of fabricating a value.

Alternative considered: use the last completed scenario duration. Rejected because it is less responsive to the current queue and may be misleading when the queue contains scenarios with different configurations.

### 4. Keep Scheduler view behavior observational, not mutational, beyond scheduler controls

Each scenario row shows identifying and progress information only. Rows may navigate to the Scenario editor when selected, but they do not expose queue mutation actions such as unschedule, cancel, or reorder.

This keeps the first iteration aligned with the existing scheduler domain model. The current specs define scheduling, execution, and progress observation, but not queue editing from the Scheduler view.

### 5. Remove duplicated scheduler controls from the egui top bar

The `ui-navigation` spec currently defines scheduler controls in the navigation bar. This change deliberately moves those controls into the Scheduler view so there is a single authoritative control surface in the Bevy path.

The alternative was to leave duplicate controls in both places for convenience. Rejected because duplication makes future changes harder and creates ambiguity about where scheduler dashboard behavior belongs.

## Risks / Trade-offs

- Queue ETA is only a heuristic -> Label it as a remaining-time estimate and leave it unknown when no observed rate exists.
- Scheduler rows may duplicate some Explorer information -> Keep the Scheduler view focused on fleet-level monitoring and omit Explorer-only content like thumbnails and result metrics.
- Sticky header behavior can be tricky in Bevy UI -> Use the existing shell layout pattern with a fixed top node and a separate scrollable body node.
- Mixed scenario configurations can skew queue ETA -> Base predictions on currently running observed epoch rates and avoid overclaiming precision in the copy.

## Migration Plan

1. Add the Scheduler view plugin and route content.
2. Move scheduler controls into the Scheduler view.
3. Remove duplicated scheduler controls from the egui top bar.
4. Update the `ui-navigation` spec to make the Scheduler view the authoritative control dashboard.

Rollback is straightforward: remove the new Scheduler view plugin and restore the top-bar controls.

## Open Questions

- Whether Scheduler rows should support direct navigation to the selected scenario's editor or stay fully read-only in the first iteration.
- Whether the queue-wide ETA should model concurrency more explicitly in a future refinement rather than using a simple aggregate observed epoch rate.
