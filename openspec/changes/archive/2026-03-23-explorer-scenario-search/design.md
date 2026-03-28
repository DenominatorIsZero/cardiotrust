## Context

The Explorer toolbar already has a `SearchQuery(String)` resource, a `SearchInputField` marker component, and the filtering gate in `apply_filter_and_sort` that checks `SearchQuery` — but the input handler (`handle_text_search_input`) is an empty stub and no keyboard events are ever routed to it. The card label rendering uses plain `Text` strings with no highlight capability.

Bevy 0.16 does not ship a built-in text-input widget, but the existing `handle_card_inline_edit` system demonstrates the established pattern: consume `EventReader<KeyboardInput>`, guard on a focus resource, branch on `KeyCode`, and append `event.text` characters to a `String` draft. The same mechanism applies here.

## Goals / Non-Goals

**Goals:**
- Make the search field focusable: clicking it sets a `SearchFocused(bool)` resource
- Capture keyboard events when focused and update `SearchQuery` live
- Fuzzy-match query against `scenario.get_id()` and `scenario.comment` (character subsequence, case-insensitive)
- Highlight the matched substring on `CardNameLabel` and `CardIdLabel` by splitting text into segments with distinct colors
- Show a clear ("×") button inside the search field when `SearchQuery` is non-empty; clicking it resets the query and removes focus
- Show a distinct empty-state message when the grid is empty because a non-empty search returned no matches vs. truly no scenarios
- Release focus (and optionally clear) when Escape is pressed while the field is focused

**Non-Goals:**
- Full cursor positioning / selection within the search field (cursor is always at end)
- Persisting the search query across sessions
- Searching any field other than ID and comment
- Animating the highlight (static color difference is sufficient)

## Decisions

### D1: Focus via `SearchFocused` resource, not Bevy `Focused` component

Bevy's UI focus model is still evolving. Following the inline-edit precedent (`CardEditMode`), a simple `SearchFocused(bool)` resource is used. When it is `true`, `handle_text_search_input` consumes keyboard events; when the inline-edit system runs and `CardEditMode.editing_index.is_some()`, it clears the reader first — the inverse should also hold: if `SearchFocused` is true, `handle_card_inline_edit` should skip keyboard event processing (guarded at the top of that system). This avoids double-consuming events.

**Alternative considered**: Using a `bevy_text_input` community crate. Rejected because: no crate currently targets Bevy 0.16 with a stable API, and the existing hand-rolled pattern in `card.rs` is sufficient and already understood.

### D2: Fuzzy match = character subsequence (not substring)

The user asked for "fuzzy searching". Character-subsequence matching is the minimal correct interpretation: query chars must appear in order inside the target, but need not be contiguous. This is implemented as a simple `O(n)` scan — no external crate needed. The match also returns the index range of the first contiguous run that satisfies the subsequence (used for highlighting the first matched span).

**Why not substring?** Substring is already in `apply_filter_and_sort`. Subsequence gives the "fuzzy" feel (e.g. "ycm" matches "2024-**y**ear-**c**ardiac-**m**odel").

**Highlighting the matched span**: The subsequence walk records the start/end byte offsets of the matching characters in the target string. These are passed down to `update_card_labels` via a new `CardMatchHighlight` component that stores `Option<(usize, usize)>` per card (start byte, end byte in the relevant string). The label render splits the text at those offsets and applies `colors::YELLOW` to the middle segment, `colors::FG0` to the surrounding segments, using `TextSpan` children (Bevy 0.16 `Text` supports multi-span via `TextSpan` child entities).

**Alternative**: Re-render card name as a single string with a Unicode bold/color escape. Rejected — Bevy `Text` renders spans natively and that approach would embed non-printable chars.

### D3: Highlight stored on a `CardMatchHighlight` component, computed in `apply_filter_and_sort`

`apply_filter_and_sort` already iterates every card and has access to `SearchQuery` + `ScenarioList`. It is the right place to:
1. Run fuzzy match
2. Set `Display::Flex / Display::None`
3. Insert/update a `CardMatchHighlight { name_range: Option<(usize, usize)>, id_range: Option<(usize, usize)> }` component on each card entity

`update_card_labels` then reads `CardMatchHighlight` to render spans. This keeps the two concerns separated and avoids an extra query pass.

### D4: Empty-state distinguishes "no scenarios" from "no search matches"

`toggle_empty_state` currently counts visible scenarios against `StatusFilter` only, and ignores `SearchQuery`. It needs to:
- Also react to `SearchQuery` changes
- When `SearchQuery` is non-empty and visible count is 0 after both filters, render an alternate message: `"No scenarios match \"<query>\""` with a sub-message `"Clear search to see all scenarios"` and a "Clear Search" button (sets `SearchQuery` to `""`)
- When `SearchQuery` is empty and visible count is 0, render the existing "No scenarios yet" message

The simplest approach: add a child node inside `EmptyStateNode` for each variant and toggle their `Display` based on whether the query is empty.

### D5: Clear button as a child entity inside `SearchInputField`

A `SearchClearButton` marker component tags a small `Button` entity (label `"×"`) spawned as the rightmost child of the search field container. Its `display` is toggled to `Display::None` when `SearchQuery` is empty, `Display::Flex` when non-empty. This avoids re-spawning nodes on every keystroke.

### D6: Search focus and inline edit are mutually exclusive

When `SearchFocused` is true:
- `handle_card_inline_edit` skips processing (guarded at top: `if search_focused.0 { keyboard.clear(); return; }`)

When `CardEditMode.editing_index.is_some()`:
- `handle_text_search_input` skips processing (mirrored guard)

Clicking a card while the search field is focused does not clear search focus — only a click on the search field or clear button, or Escape, changes `SearchFocused`.

## Risks / Trade-offs

- [Risk] Bevy `TextSpan` multi-span rendering requires restructuring label entities from a single `Text` to a parent `Text` with `TextSpan` children. If the current `update_card_labels` system queries `&mut Text` directly on `CardNameLabel`, it will need refactoring. → Mitigation: limit the multi-span rendering to a new `update_card_label_highlights` system that runs after `update_card_labels` and only modifies spans when `SearchQuery` is non-empty; when query is empty, render as before.
- [Risk] `handle_card_inline_edit` and `handle_text_search_input` both read from `EventReader<KeyboardInput>`. Bevy event readers share a cursor per `SystemParam` — if both systems run in the same schedule frame, the second will not see events the first consumed. → Mitigation: add explicit mutual-exclusion guards (as described in D6) and register them in the same `chain()` or with an explicit ordering constraint so only the active one consumes events.
- [Risk] Fuzzy highlight byte offsets may not align with char boundaries for multi-byte Unicode scenario IDs or comments. → Mitigation: the subsequence walk operates on `char` indices, converted to byte offsets using `str::char_indices()` before storing.

## Migration Plan

No data migration required. `SearchQuery` already exists as a resource; the change only adds behavior to populate it. All new components (`SearchFocused`, `CardMatchHighlight`, `SearchClearButton`) are registered fresh. The feature is purely additive and the existing filtering path degrades gracefully (empty query = all pass).
