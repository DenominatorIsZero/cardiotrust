## Why

The Explorer toolbar has a search bar placeholder that currently does nothing — the `handle_text_search_input` system is an empty stub and `SearchQuery` is never populated, making the widget misleading to users. Fuzzy search over scenario IDs and comments is the most direct way to locate a scenario in a large project, and the filtering infrastructure is already wired up and ready to be activated.

## What Changes

- Replace the non-functional search placeholder in the Explorer toolbar with a working text input that captures keyboard events and populates `SearchQuery`
- Implement fuzzy matching of the query against `scenario.get_id()` and `scenario.comment`
- Filter out non-matching scenario cards from the grid (consistent with existing status-filter behavior — cards collapse, not merely hide)
- Highlight the matched substring on the card's name label and ID label so users can see why a card is included
- Add a clear ("×") button inside/adjacent to the search field that resets `SearchQuery` to `""`; the button is only visible when the query is non-empty
- Display an explicit "No scenarios match your search" message when the grid would otherwise be empty due to an active non-empty query (prevents confusion when users forget they have an active search)

## Capabilities

### New Capabilities

- `explorer-scenario-search`: Fuzzy text search in the Explorer toolbar — keyboard-driven input, fuzzy matching on ID and comment, highlight of matched substring on cards, clear button, and empty-search notice

### Modified Capabilities

- `ui-explorer-view`: The text search requirement (Req: toolbar allows filtering … text search) transitions from a stub to a fully implemented capability; the empty-state requirement gains a second trigger (active search with no matches)

## Impact

- `src/ui/bevy_shell/explorer/toolbar.rs` — `handle_text_search_input`, `SearchInputField`, `SearchQuery`, toolbar spawn
- `src/ui/bevy_shell/explorer/card.rs` — card name/ID label rendering for match highlight; `sync_cards_to_scenarios` rebuild trigger
- `src/ui/bevy_shell/explorer/empty_state.rs` — conditionally render "no search matches" variant when `SearchQuery` is non-empty
- `src/ui/bevy_shell/explorer/mod.rs` — ensure search-clear and highlight systems are registered
- No new dependencies required; fuzzy matching implemented inline (character-subsequence style) to avoid pulling in a crate
