## ADDED Requirements

### Requirement: The Explorer toolbar search field accepts keyboard input and updates the live search query

The Explorer toolbar search field SHALL become active when the user clicks it. While active, printable keyboard characters SHALL be appended to the search query, Backspace SHALL remove the last character, and Escape SHALL deactivate the field. The search query SHALL be applied live to the visible card set on every keystroke without requiring a confirmation action.

#### Scenario: Clicking the search field activates it

- **WHEN** the user clicks the search field in the Explorer toolbar
- **THEN** the search field is in an active/focused state and subsequent keyboard input is captured by the field

#### Scenario: Typing characters appends to the query

- **WHEN** the search field is active and the user types printable characters
- **THEN** those characters are appended to the search query and the card grid updates immediately to show only matching scenarios

#### Scenario: Backspace removes the last character

- **WHEN** the search field is active and the user presses Backspace
- **THEN** the last character is removed from the search query and the card grid updates immediately

#### Scenario: Escape deactivates the search field

- **WHEN** the search field is active and the user presses Escape
- **THEN** the search field is deactivated; the existing query is retained and the grid remains filtered

#### Scenario: Search and card inline-edit are mutually exclusive

- **WHEN** the search field is active
- **THEN** card inline-name-edit keyboard input is suppressed; and conversely, when a card is in inline-edit mode, search keyboard input is suppressed

### Requirement: The search field displays a clear button that resets the query

When the search query is non-empty, the search field SHALL display a clear control (e.g. an "×" indicator). Activating the clear control SHALL set the query to empty, deactivate the field, and restore the full unfiltered card grid. When the search query is empty, the clear control SHALL not be visible.

#### Scenario: Clear button appears when query is non-empty

- **WHEN** the search query contains at least one character
- **THEN** a clear control is visible within or adjacent to the search field

#### Scenario: Clear button is hidden when query is empty

- **WHEN** the search query is empty
- **THEN** no clear control is visible within the search field

#### Scenario: Activating the clear button empties the query

- **WHEN** the user activates the clear control
- **THEN** the search query is set to empty, the clear control disappears, and all scenarios (subject to the active status filter) are shown in the grid

### Requirement: The Explorer filters scenario cards using fuzzy matching on ID and comment

When a search query is active, the Explorer grid SHALL show only those scenario cards whose scenario ID or comment fuzzy-matches the query. Fuzzy matching means all characters of the query appear in order within the target string; the match is case-insensitive. Scenario cards that do not match SHALL be removed from the grid layout (not merely hidden in place).

#### Scenario: Cards with matching ID are shown

- **WHEN** the user types a query and a scenario's ID contains all query characters in order (case-insensitively)
- **THEN** that scenario's card is visible in the grid

#### Scenario: Cards with matching comment are shown

- **WHEN** the user types a query and a scenario's comment contains all query characters in order (case-insensitively)
- **THEN** that scenario's card is visible in the grid

#### Scenario: Cards with no match are removed from the grid

- **WHEN** the user types a query and a scenario's ID and comment both fail to fuzzy-match it
- **THEN** that scenario's card is absent from the grid layout and occupies no space

#### Scenario: Empty query shows all cards

- **WHEN** the search query is empty
- **THEN** all scenarios (subject to the active status filter) are visible in the grid

### Requirement: Matched substrings are highlighted on scenario cards

For each visible scenario card that passes the fuzzy search, the matched portion of the display name (comment if set, otherwise ID) and the secondary ID label SHALL be visually highlighted to distinguish it from non-matching text. The highlight SHALL be removed from all card labels when the search query is empty.

#### Scenario: Matched portion of name label is highlighted

- **WHEN** a scenario card is visible due to a fuzzy match on its display name
- **THEN** the matching portion of the name text is rendered in a distinct highlight color

#### Scenario: Matched portion of ID label is highlighted

- **WHEN** a scenario card is visible due to a fuzzy match on its ID
- **THEN** the matching portion of the ID text is rendered in a distinct highlight color

#### Scenario: No highlight when query is empty

- **WHEN** the search query is empty
- **THEN** all card labels are rendered in their normal (non-highlighted) colors

### Requirement: The Explorer shows a search-specific empty state when a non-empty query yields no matches

When the search query is non-empty and no scenario cards pass the combined status filter and fuzzy search, the Explorer SHALL display a message that attributes the empty grid to the active search query (e.g. `No scenarios match "<query>"`). This message SHALL be distinct from the generic "No scenarios yet" empty state and SHALL include a control to clear the search. The generic empty state SHALL only appear when the project has zero scenarios regardless of filter or search.

#### Scenario: Search-empty state appears when query matches nothing

- **WHEN** the search query is non-empty and no scenario matches it (under the active status filter)
- **THEN** the Explorer displays a message referencing the search query and a "Clear Search" control; the card grid is not shown

#### Scenario: Clear search from empty state restores the grid

- **WHEN** the search-empty state is displayed and the user activates the "Clear Search" control
- **THEN** the search query is cleared, the search-empty state disappears, and scenario cards are shown again

#### Scenario: Generic empty state is not shown while query is active

- **WHEN** the search query is non-empty and no scenarios match
- **THEN** only the search-specific empty state is shown, not the generic "No scenarios yet" message
