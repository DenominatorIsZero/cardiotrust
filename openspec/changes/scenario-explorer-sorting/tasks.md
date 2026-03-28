## 1. Attach Card Identity Component

- [x] 1.1 Define `CardScenarioId(pub String)` component in `src/ui/bevy_shell/explorer/card.rs` (derive `Component`, `Debug`, `Clone`)
- [x] 1.2 In `spawn_card`, insert `CardScenarioId(scenario_id.to_string())` on the root card entity alongside the other components

## 2. Implement Sort Logic in `apply_filter_and_sort`

- [x] 2.1 Add `Query<(Entity, &CardScenarioId)>` parameter to `apply_filter_and_sort` to map entities to scenario IDs
- [x] 2.2 Remove the `let _ = order;` placeholder and the TODO comment (toolbar.rs:683–686)
- [x] 2.3 After the filter pass, collect the list of visible card entities (those not hidden by the status filter)
- [x] 2.4 Implement sort comparator for `SortOrder::DateNewest`: sort by `scenario.started` descending; `None` timestamps sort last; tiebreak by `scenario.get_id()` ascending
- [x] 2.5 Implement sort comparator for `SortOrder::LossLowest`: sort by `scenario.summary.as_ref().map(|s| s.loss).unwrap_or(f32::MAX)` ascending; tiebreak by ID
- [x] 2.6 Implement sort comparator for `SortOrder::DiceHighest`: sort by `scenario.summary.as_ref().map(|s| s.dice).unwrap_or(f32::MIN)` descending (negate or reverse); tiebreak by ID
- [x] 2.7 Implement sort comparator for `SortOrder::Name`: sort by `scenario.comment` if non-empty else `scenario.get_id()`, case-insensitive (use `.to_lowercase()`); tiebreak by ID
- [x] 2.8 After sorting the visible entity list, call `commands.entity(grid_parent).replace_children(&sorted_entities)` to apply the new order

## 3. Trigger Re-sort on Relevant Changes

- [x] 3.1 Gate the sort pass with `if order.is_changed() || scenario_list.is_changed()` so it only runs when needed, not every frame
- [x] 3.2 Verify the system also re-sorts when the status filter changes (filter change already modifies the visible set, so ensure the sort runs after filter in the same system)

## 4. Validation

- [x] 4.1 Run `just check` and confirm zero clippy warnings and no compile errors
- [x] 4.2 Run `just lint` and confirm `clippy-tracing` passes (add `#[tracing::instrument(skip_all)]` to any new `pub fn` if needed)
- [x] 4.3 Run `just test` and confirm all tests pass
- [ ] 4.4 Manual smoke test: launch `just run`, open the explorer with multiple scenarios (including some with and without summaries), click each sort button, and verify the card order changes correctly for all four modes
