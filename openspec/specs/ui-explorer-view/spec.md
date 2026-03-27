## Purpose

Governs the Bevy-native Explorer view — the card grid layout used to browse, filter, sort, and act on scenarios when the Bevy UI backend is active. This spec covers card rendering, interaction model, toolbar, empty state, thumbnails, and context menu behavior.

## Requirements

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

Every scenario card SHALL contain, from top to bottom: a status badge, a thumbnail area, an optional metrics row, a display name (comment if set, otherwise ID), a secondary ID label, and a creation timestamp. The status badge SHALL use the Gruvbox Material palette: green for Done, yellow for Running, blue for Queued, grey for Planning, and red for Failed. Quick-action buttons ([D] delete, [C] copy, [S] schedule) SHALL appear pinned to the bottom-right of each card.

#### Scenario: Done card shows thumbnail and metrics

- **WHEN** a scenario in Done state is rendered as a card
- **THEN** the card displays a thumbnail image in the thumbnail area and a metrics row showing Dice score and final loss

#### Scenario: Running card shows progress indicator and ETC

- **WHEN** a scenario in Running state is rendered as a card
- **THEN** the card displays a progress percentage and an estimated time to completion in the thumbnail area, updating each render frame

#### Scenario: Planning card shows placeholder

- **WHEN** a scenario in Planning state is rendered as a card
- **THEN** the card displays a muted placeholder icon in the thumbnail area and no metrics row

#### Scenario: Failed card shows error indicator

- **WHEN** a scenario in Failed state is rendered as a card
- **THEN** the card displays an error icon in the thumbnail area

### Requirement: Card interaction follows a single-click select, second-click edit, double-click navigate model

Single-clicking an unselected card selects it. Single-clicking an already-selected card enters inline comment/name edit mode. Double-clicking navigates: to the Results view if the scenario is Done, to the Scenario view otherwise.

#### Scenario: First click selects the card

- **WHEN** the user single-clicks an unselected scenario card
- **THEN** that scenario becomes the active scenario; no navigation occurs

#### Scenario: Second click on selected card enters inline edit

- **WHEN** the user single-clicks a card that is already selected
- **THEN** the card's name/comment label enters an inline edit mode showing a text cursor; keyboard input updates the draft

#### Scenario: Enter commits the inline edit

- **WHEN** the user presses Enter while a card is in inline edit mode
- **THEN** the draft is saved as the scenario comment and persisted to disk; the card returns to normal display

#### Scenario: Escape cancels the inline edit

- **WHEN** the user presses Escape while a card is in inline edit mode
- **THEN** the draft is discarded and the card returns to normal display with the original comment

#### Scenario: Double-click on Done card navigates to Results

- **WHEN** the user double-clicks a card for a Done scenario
- **THEN** the application navigates to the Results view

#### Scenario: Double-click on non-Done card navigates to Scenario

- **WHEN** the user double-clicks a card for a non-Done scenario
- **THEN** the application navigates to the Scenario view

### Requirement: A card visually indicates the active scenario with an orange border

The card representing the currently active scenario SHALL display a 2 px orange border and a rename hint on its name label to distinguish it from unselected cards.

#### Scenario: Active scenario card has orange border

- **WHEN** a scenario is the active scenario and the Explorer view is rendered
- **THEN** that scenario's card displays a 2 px orange border; all other cards do not

### Requirement: The Explorer toolbar allows filtering by status, sorting, and text search

A toolbar above the card grid SHALL provide: a status filter (All, Planning, Queued, Running, Done, Failed), a sort order selector (Date newest first, Loss lowest first, Dice highest first, Name), and a text search field that matches scenario ID and comment. The active filter and sort SHALL be visually indicated. Filtered-out cards SHALL collapse out of the grid (not merely hidden). Applying any filter or sort SHALL update the visible cards immediately.

#### Scenario: Status filter hides non-matching cards

- **WHEN** the user selects a specific status in the filter bar
- **THEN** only cards whose scenario status matches are shown; others collapse from the grid

#### Scenario: Active filter button is visually highlighted

- **WHEN** a filter or sort button is active
- **THEN** the button is displayed with an orange background to distinguish it from inactive buttons

#### Scenario: Text search filters cards by ID and comment

- **WHEN** the user types a query in the search field
- **THEN** only cards whose scenario ID or comment contains the query text (case-insensitive) are shown

### Requirement: A "New Scenario" action card is present at the end of the grid

The Explorer grid SHALL include a distinguished action card after all scenario cards. This card SHALL use a dashed border and display a "+" icon and "New Scenario" label. Clicking it SHALL create a new scenario in Planning state and set it as the active scenario. The Explorer view remains active.

#### Scenario: New Scenario card creates a planning scenario

- **WHEN** the user clicks the "New Scenario" action card
- **THEN** a new scenario in Planning state is added to the project, becomes the active scenario, and a new scenario card appears in the grid; the Explorer view remains active

### Requirement: The Explorer shows an empty state when the project has no scenarios

When the active project contains no scenarios, the Explorer SHALL display a centered message and a prominent "New Scenario" call-to-action button instead of the card grid.

#### Scenario: Empty project shows empty-state message

- **WHEN** the active project contains no scenarios
- **THEN** the Explorer displays a centered "No scenarios yet" message and a "New Scenario" button

### Requirement: Done scenario cards display on-demand thumbnails

The Explorer view SHALL generate a chart-style thumbnail image for each Done scenario the first time that scenario's card is displayed. Thumbnail generation SHALL not mark the ThumbnailCache resource as changed when there is no work to do. Generated thumbnails SHALL be cached in memory and reused on subsequent renders without regeneration.

#### Scenario: Thumbnail loads on first display

- **WHEN** a Done scenario card is shown for the first time
- **THEN** a thumbnail image is generated and displayed in the card's thumbnail area

#### Scenario: Cached thumbnail appears without regeneration

- **WHEN** a Done scenario card is shown after its thumbnail has been generated
- **THEN** the thumbnail image is displayed immediately without triggering a new generation task

### Requirement: Each scenario card provides a context menu with per-scenario actions

Right-clicking a scenario card SHALL open a context menu near the cursor containing: Edit Scenario, Copy Scenario, Delete Scenario, and state-appropriate actions (Schedule if Planning/Failed; Unschedule if Queued; Open in Results and Open in Volumetric if Done). The context menu SHALL dismiss on any left-click.

#### Scenario: Context menu shows state-appropriate actions

- **WHEN** the user right-clicks a card for a Done scenario
- **THEN** the context menu contains "Edit Scenario", "Open in Results", and "Open in Volumetric"

#### Scenario: Context menu dismisses on left-click outside

- **WHEN** a context menu is open and the user left-clicks outside it
- **THEN** the context menu closes without performing any action

### Requirement: Card hover state provides a visual affordance

When the user hovers the cursor over a scenario card, the card SHALL display an elevated background color to indicate interactivity. Background and border updates SHALL use change-detection guards to avoid unnecessary render churn.

#### Scenario: Hovered card changes background

- **WHEN** the user moves the cursor over a scenario card
- **THEN** the card background changes to the hover surface color
