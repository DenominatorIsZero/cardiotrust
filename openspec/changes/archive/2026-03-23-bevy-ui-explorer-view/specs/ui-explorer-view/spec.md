## ADDED Requirements

### Requirement: Explorer view displays all scenarios as a card grid
The Explorer view SHALL present every scenario in the active project as a card in a responsive grid layout. The grid SHALL adapt its column count to the available width: four columns at ≥1400 px, three at 1000–1399 px, two at 700–999 px, and one at <700 px. Cards SHALL be separated by a 16 px gap.

#### Scenario: Grid renders correct column count at wide width
- **WHEN** the Explorer view is active and the content area is ≥1400 px wide
- **THEN** scenario cards are arranged in four columns

#### Scenario: Grid renders single column at narrow width
- **WHEN** the Explorer view is active and the content area is <700 px wide
- **THEN** scenario cards are arranged in a single full-width column

#### Scenario: All scenarios appear in the grid
- **WHEN** the Explorer view is active and the project contains N scenarios
- **THEN** N scenario cards plus one "New Scenario" action card are visible (subject to active filter)

### Requirement: Each scenario card surfaces status, thumbnail area, metrics, identity, and timestamp
Every scenario card SHALL contain, from top to bottom: a status badge, a thumbnail area, an optional metrics row, a title, an optional comment, and a creation timestamp. The status badge SHALL use the Gruvbox Material palette: green for Done, yellow for Running, blue for Queued, grey for Planning, and red for Failed.

#### Scenario: Done card shows thumbnail and metrics
- **WHEN** a scenario in Done state is rendered as a card
- **THEN** the card displays a thumbnail image in the thumbnail area and a metrics row showing Dice score and final loss

#### Scenario: Running card shows progress indicator and ETC
- **WHEN** a scenario in Running state is rendered as a card
- **THEN** the card displays a circular progress indicator with percentage and an estimated time to completion in the thumbnail area, updating each render frame

#### Scenario: Planning card shows placeholder
- **WHEN** a scenario in Planning state is rendered as a card
- **THEN** the card displays a muted placeholder icon in the thumbnail area and no metrics row

#### Scenario: Failed card shows error indicator
- **WHEN** a scenario in Failed state is rendered as a card
- **THEN** the card displays an error icon and a brief error summary in the thumbnail area

### Requirement: Clicking a scenario card sets it as the active scenario and navigates to Scenario view
The Explorer view SHALL treat a card click as a two-step action: (1) set the clicked scenario as the active scenario, and (2) navigate the application to the Scenario view.

#### Scenario: Card click activates scenario and navigates
- **WHEN** the user clicks a scenario card in the Explorer
- **THEN** that scenario becomes the active scenario and the application transitions to the Scenario view

### Requirement: A card visually indicates the active scenario with an orange border
The card representing the currently active scenario SHALL display a 2 px orange border to distinguish it from unselected cards.

#### Scenario: Active scenario card has orange border
- **WHEN** a scenario is the active scenario and the Explorer view is rendered
- **THEN** that scenario's card displays a 2 px orange border; all other cards do not

### Requirement: The Explorer toolbar allows filtering by status, sorting, and text search
A toolbar above the card grid SHALL provide: a status filter (All, Planning, Queued, Running, Done, Failed), a sort order selector (Date newest first, Loss lowest first, Dice highest first, Name), and a text search field that matches scenario ID and comment. Applying any filter or sort SHALL update the visible cards immediately.

#### Scenario: Status filter hides non-matching cards
- **WHEN** the user selects a specific status in the filter dropdown
- **THEN** only cards whose scenario status matches the selected status are shown

#### Scenario: Text search filters cards by ID and comment
- **WHEN** the user types a query in the search field
- **THEN** only cards whose scenario ID or comment contains the query text (case-insensitive) are shown

#### Scenario: Sort by Loss orders cards ascending
- **WHEN** the user selects "Loss (lowest first)" in the sort selector
- **THEN** cards are reordered so that Done scenarios with the lowest final loss appear first

### Requirement: A "New Scenario" action card is present at the end of the grid
The Explorer grid SHALL include a distinguished action card after all scenario cards. This card SHALL use a dashed border and display a "+" icon and "New Scenario" label. Clicking it SHALL create a new scenario in Planning state, set it as the active scenario, and replace the action card in-place with the new scenario card in an inline-edit state. The application SHALL remain on the Explorer view; navigation to the Scenario view only occurs via an explicit card click.

#### Scenario: New Scenario card creates a card in inline-edit state
- **WHEN** the user clicks the "New Scenario" card
- **THEN** a new scenario in Planning state is added to the project, becomes the active scenario, and a new scenario card appears in the grid with its name and comment fields in an editable state; the Explorer view remains active

#### Scenario: Confirming inline edit commits the new scenario name
- **WHEN** the user confirms the inline edit (pressing Enter or clicking away)
- **THEN** the scenario card transitions to its normal display state with the entered name and comment saved

### Requirement: The Explorer shows an empty state when the project has no scenarios
When the active project contains no scenarios (or all are filtered out), the Explorer SHALL display a centered message and a prominent "New Scenario" call-to-action button instead of the card grid.

#### Scenario: Empty project shows empty-state message
- **WHEN** the active project contains no scenarios
- **THEN** the Explorer displays a centered "No scenarios yet" message and a "New Scenario" button

#### Scenario: All-filtered-out shows empty-state message
- **WHEN** an active filter excludes all scenarios
- **THEN** the Explorer displays an empty-state message indicating no matches

### Requirement: Done scenario cards display on-demand thumbnails
The Explorer view SHALL generate a thumbnail image for each Done scenario the first time that scenario's card is displayed. Thumbnail generation SHALL be asynchronous. A loading indicator SHALL appear in the thumbnail area while generation is in progress. Generated thumbnails SHALL be cached in memory and reused on subsequent renders without regeneration.

#### Scenario: Thumbnail loads asynchronously on first display
- **WHEN** a Done scenario card is shown for the first time
- **THEN** a loading indicator appears in the thumbnail area while the thumbnail is generated in the background

#### Scenario: Cached thumbnail appears without regeneration
- **WHEN** a Done scenario card is shown after its thumbnail has been generated
- **THEN** the thumbnail image is displayed immediately without triggering a new generation task

### Requirement: Each scenario card provides a context menu with per-scenario actions
Right-clicking (or long-pressing) a scenario card SHALL open a context menu near the cursor with the following actions: Copy Scenario, Delete Scenario, Schedule (if Planning/Failed), Unschedule (if Queued), Open in Results (if Done), Open in Volumetric (if Done). Actions that do not apply to the scenario's current state SHALL be absent or disabled.

#### Scenario: Context menu shows state-appropriate actions
- **WHEN** the user right-clicks a card for a Done scenario
- **THEN** the context menu contains "Open in Results" and "Open in Volumetric" and does not show "Schedule"

#### Scenario: Context menu dismisses on outside click
- **WHEN** a context menu is open and the user clicks outside it
- **THEN** the context menu closes without performing any action

### Requirement: Card hover state provides a visual affordance
When the user hovers the cursor over a scenario card, the card SHALL display an elevated background color to indicate interactivity.

#### Scenario: Hovered card changes background
- **WHEN** the user moves the cursor over a scenario card
- **THEN** the card background changes to the hover surface color
