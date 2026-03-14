# Scheduler View -- Job Monitor

## Purpose

Dedicated view for monitoring and controlling scenario execution. Extracted from the old topbar (Start/Stop/Jobs slider) into a proper dashboard with at-a-glance status.

## Layout

```
+------------------------------------------------------------------+
| Context bar: My Project > Scheduler                              |
+------------------------------------------------------------------+
|                                                                  |
|  +--------------------+  +-------------------+  +--------------+ |
|  |                    |  |                   |  |              | |
|  |        3           |  |       47%         |  |    ~12m      | |
|  |   Running Jobs     |  |   Overall Progress|  |  Remaining   | |
|  |                    |  |                   |  |              | |
|  +--------------------+  +-------------------+  +--------------+ |
|                                                                  |
|  Overall: [================================>-----------] 47%     |
|                                                                  |
|  +------+  +------+  Parallel jobs: [==|====] 4                  |
|  |  Start |  | Stop |                                            |
|  +------+  +------+                                              |
|                                                                  |
+------------------------------------------------------------------+
|  ACTIVE JOBS                                                     |
+------------------------------------------------------------------+
|  v8_Dr_2026-03-14-09-30  Running  [========>----] 72%  ETC: 45s |
|  v8_Dr_2026-03-14-10-15  Running  [===>---------] 23%  ETC: 3m  |
|  v8_Dr_2026-03-14-10-20  Running  [=>-----------]  8%  ETC: 5m  |
+------------------------------------------------------------------+
|  QUEUE (2 waiting)                                               |
+------------------------------------------------------------------+
|  v8_Dr_2026-03-14-10-30  Queued                                  |
|  v8_Dr_2026-03-14-10-45  Queued                                  |
+------------------------------------------------------------------+
|  COMPLETED (5)                                                   |
+------------------------------------------------------------------+
|  v8_Dr_2026-03-14-08-00  Done   Loss: 1.2e-4  Dice: 0.87  1m2s |
|  v8_Dr_2026-03-14-08-15  Done   Loss: 2.3e-3  Dice: 0.72  58s  |
|  ...                                                             |
+------------------------------------------------------------------+
```

## Summary Cards (Top)

Three large stat cards across the top, displaying key numbers prominently:

### Card 1: Running Jobs
- **Large number**: Count of currently executing scenarios (e.g., "3")
- **Label**: "Running Jobs"
- **Color accent**: `green` number when jobs are active, `grey1` when idle
- Background: `bg1`

### Card 2: Overall Progress
- **Large number**: Percentage of all scheduled scenarios completed (e.g., "47%")
- **Label**: "Overall Progress"
- **Supplementary**: "X of Y scenarios"
- **Color**: `orange` accent
- Background: `bg1`

### Card 3: Estimated Time Remaining
- **Large number**: Estimated time for all queued+running scenarios to complete (e.g., "~12m")
- **Label**: "Remaining"
- **Color**: `aqua` accent
- **Calculation**: Sum of individual ETCs for running + queued scenarios
- Shows "--" when no jobs are scheduled
- Background: `bg1`

## Overall Progress Bar

Below the summary cards, a full-width progress bar:
- Track: `bg3`
- Fill: gradient from `orange` to `green` as it approaches 100%
- Text overlay: percentage
- Height: 24px

## Controls Row

### Start Button
- `green` background, white text
- Starts/resumes the scheduler
- Disabled when already running or nothing to schedule

### Stop Button
- `red` background, white text
- Pauses the scheduler (finishes current epoch, stops picking new scenarios)
- Disabled when already stopped

### Parallel Jobs Slider
- Range: 1 to 32
- Current value displayed beside the slider
- Label: "Parallel jobs"
- Adjustable while running (takes effect for next job pickup)

## Job Lists

Three sections below the controls, each showing a list of scenarios:

### Active Jobs
- Only visible when jobs are running
- Each row shows:
  - Scenario ID (truncated, clickable -- navigates to Scenario view)
  - Status badge: "Running" in `green`
  - Individual progress bar (inline, ~200px wide)
  - Percentage
  - ETC (estimated time to completion)
  - Current epoch / total epochs (small text)
- Rows update in real-time (progress bars animate)

### Queue
- Scenarios scheduled but not yet started
- Each row shows:
  - Scenario ID (clickable)
  - Status: "Queued" in `blue`
  - Drag handle (optional): reorder queue priority
- "Clear Queue" button to unschedule all queued scenarios

### Completed
- Recently completed scenarios (current session)
- Each row shows:
  - Scenario ID (clickable)
  - Status: "Done" in `green` or "Failed" in `red`
  - Key metrics: Loss, Dice (for Done)
  - Total execution time
  - Quick action: "View Results" button (navigates to Results view)
- Collapsible section (default: collapsed if many, expanded if few)

## Empty State

When no scenarios are scheduled and none have run:
- Centered message: "No scenarios scheduled"
- Subtitle: "Go to the Explorer to schedule scenarios for execution"
- Link/button to Explorer

## Real-Time Updates

The scheduler view should update frequently:
- Progress bars update every ~500ms
- ETC recalculates based on recent epoch durations
- When a job completes, it animates from Active to Completed
- When a new job starts, it animates from Queue to Active
- Summary cards update with each change

## Responsive Behavior

- Summary cards: 3 across on wide screens, stack to 2+1 or 3 vertical on narrow
- Job lists: always full-width, single column
- On narrow screens: progress bars shrink, ETC text may wrap below

## WASM Considerations

- WASM runs scenarios on the main thread or via Web Workers
- Only 1 parallel job in WASM (no true multi-threading without SharedArrayBuffer)
- The parallel jobs slider should cap at 1 for WASM builds, with a note explaining why
- ETC calculation works the same way
