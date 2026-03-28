## Why

The scenario explorer toolbar already exposes four sort buttons (Date, Loss, Dice, Name) and the `SortOrder` resource is wired through the ECS, but clicking any button has no visible effect on the card grid — the reordering logic was explicitly deferred with a TODO comment (`toolbar.rs:684`). Users cannot meaningfully navigate large scenario lists without working sort.

## What Changes

- Implement actual card reordering in `apply_filter_and_sort` so the card grid reflects the selected `SortOrder` resource.
- `DateNewest`: sort by `scenario.started` descending (most recent first); scenarios with no timestamp go last.
- `LossLowest`: sort by `scenario.summary.loss` ascending; scenarios without a summary go last.
- `DiceHighest`: sort by `scenario.summary.dice` descending; scenarios without a summary go last.
- `Name`: sort by display name (comment if non-empty, else id) ascending, case-insensitive.
- Remove the `let _ = order;` placeholder and the associated TODO comment.
- Active sort button is already highlighted orange; no visual changes needed.

## Capabilities

### New Capabilities

- `scenario-sort`: Sorting the scenario card grid by date, loss, dice, or name using the existing toolbar buttons.

### Modified Capabilities

(none — no existing spec requirements are changing, only an unimplemented TODO is being fulfilled)

## Impact

- `src/ui/bevy_shell/explorer/toolbar.rs` — `apply_filter_and_sort` system: remove TODO placeholder, implement sort logic that reorders card entities in the parent node's children list.
- `src/ui/bevy_shell/explorer/card.rs` — may need `CardScenarioId` or similar component to map card entities back to scenario data for ordering.
- No API or data-model changes; `SortOrder`, `SortOrderButton`, `Scenario`, and `ScenarioList` structs are unchanged.
- No breaking changes.
