## MODIFIED Requirements

### Requirement: The Explorer toolbar allows filtering by status, sorting, and text search

A toolbar above the card grid SHALL provide: a status filter (All, Planning, Queued, Running, Done, Failed), a sort order selector (Date newest first, Loss lowest first, Dice highest first, Name), and a text search field that accepts keyboard input and fuzzy-matches scenario ID and comment. The active filter and sort SHALL be visually indicated. Filtered-out cards SHALL collapse out of the grid (not merely hidden). Applying any filter or sort SHALL update the visible cards immediately. The search field SHALL support live input, a clear control, and matched-substring highlighting on cards as specified in the `explorer-scenario-search` capability.

#### Scenario: Status filter hides non-matching cards

- **WHEN** the user selects a specific status in the filter bar
- **THEN** only cards whose scenario status matches are shown; others collapse from the grid

#### Scenario: Active filter button is visually highlighted

- **WHEN** a filter or sort button is active
- **THEN** the button is displayed with an orange background to distinguish it from inactive buttons

#### Scenario: Text search filters cards by fuzzy match on ID and comment

- **WHEN** the user types a query in the search field
- **THEN** only cards whose scenario ID or comment fuzzy-matches the query (all query characters appear in order, case-insensitive) are shown; non-matching cards are removed from the grid layout

## MODIFIED Requirements

### Requirement: The Explorer shows an empty state when the project has no scenarios

When the active project contains no scenarios regardless of filter or search, the Explorer SHALL display a centered "No scenarios yet" message and a prominent "New Scenario" call-to-action button instead of the card grid. When the project has scenarios but none match the active search query, the Explorer SHALL instead display a search-specific empty state (see `explorer-scenario-search` capability). These two states SHALL never be shown simultaneously.

#### Scenario: Empty project shows empty-state message

- **WHEN** the active project contains no scenarios
- **THEN** the Explorer displays a centered "No scenarios yet" message and a "New Scenario" button

#### Scenario: Active search with no matches shows search-specific empty state

- **WHEN** the project contains at least one scenario but the active search query matches none of them
- **THEN** the Explorer displays a search-specific message referencing the query, not the generic "No scenarios yet" message
