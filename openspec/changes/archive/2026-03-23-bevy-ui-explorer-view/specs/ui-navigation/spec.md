## MODIFIED Requirements

### Requirement: The Explorer lists all known scenarios with their current status
The Explorer view SHALL display all loaded scenarios in a responsive card grid. Each card SHALL show the scenario's identifier, lifecycle status, a thumbnail area (thumbnail image for Done, progress indicator for Running, placeholder for Planning/Queued, error indicator for Failed), final performance metrics when available (Dice score and final loss), a free-text comment when present, and a creation timestamp. For Running scenarios, the card SHALL show live progress as a fraction of completion with an estimated time remaining, updating each render frame.

#### Scenario: Running scenario shows animated progress
- **WHEN** a scenario is in the Running state
- **THEN** the Explorer displays that scenario's completion fraction and estimated time remaining, updating each render frame

#### Scenario: Completed scenario shows final summary metrics
- **WHEN** a scenario is in the Done state
- **THEN** the Explorer displays the scenario's final loss decomposition and segmentation scores

#### Scenario: New scenario is created from Explorer
- **WHEN** the user activates the "New" action in the Explorer
- **THEN** a new scenario in Planning state is added to the list, selected, and a new card enters inline-edit state; the Explorer view remains active
