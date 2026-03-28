## ADDED Requirements

### Requirement: Sort by date
The explorer SHALL order scenario cards by creation date, most recent first, when the "Date" sort is active. Scenarios without a recorded creation date SHALL appear after all dated scenarios.

#### Scenario: Cards ordered newest first
- **WHEN** the user selects the "Date" sort
- **THEN** the card grid displays scenarios in descending order of creation date

#### Scenario: Undated scenarios go last
- **WHEN** the "Date" sort is active and some scenarios have no creation date
- **THEN** those scenarios appear after all scenarios that have a creation date

### Requirement: Sort by loss
The explorer SHALL order scenario cards by their training loss value, lowest first, when the "Loss" sort is active. Scenarios without a computed loss value SHALL appear after all scored scenarios.

#### Scenario: Cards ordered by ascending loss
- **WHEN** the user selects the "Loss" sort
- **THEN** the card grid displays scenarios in ascending order of loss value

#### Scenario: Unscored scenarios go last for loss sort
- **WHEN** the "Loss" sort is active and some scenarios have no summary metrics
- **THEN** those scenarios appear after all scenarios that have a loss value

### Requirement: Sort by Dice score
The explorer SHALL order scenario cards by their Dice coefficient, highest first, when the "Dice" sort is active. Scenarios without a computed Dice score SHALL appear after all scored scenarios.

#### Scenario: Cards ordered by descending Dice
- **WHEN** the user selects the "Dice" sort
- **THEN** the card grid displays scenarios in descending order of Dice score

#### Scenario: Unscored scenarios go last for Dice sort
- **WHEN** the "Dice" sort is active and some scenarios have no summary metrics
- **THEN** those scenarios appear after all scenarios that have a Dice score

### Requirement: Sort by name
The explorer SHALL order scenario cards alphabetically by display name, ascending and case-insensitive, when the "Name" sort is active. A scenario's display name is its user-provided label when one exists, otherwise its identifier.

#### Scenario: Cards ordered alphabetically by display name
- **WHEN** the user selects the "Name" sort
- **THEN** the card grid displays scenarios in ascending alphabetical order of their display names, ignoring letter case

#### Scenario: Identifier used when no label set
- **WHEN** the "Name" sort is active and a scenario has no user-provided label
- **THEN** that scenario is ordered using its identifier as the display name

### Requirement: Sort order is immediately reflected
The explorer SHALL update the card grid order immediately when the user clicks a sort button, without requiring any additional interaction.

#### Scenario: Immediate reorder on button click
- **WHEN** the user clicks a sort button
- **THEN** the card grid reorders visually within the same frame

### Requirement: Sort persists through filter changes
The active sort order SHALL continue to apply when the scenario status filter changes.

#### Scenario: Sort maintained after filter change
- **WHEN** the user changes the status filter while a non-default sort is active
- **THEN** the visible cards remain ordered according to the active sort

### Requirement: Stable tiebreaker
When two scenarios have equal values for the primary sort key, the explorer SHALL break the tie by scenario identifier in ascending alphabetical order.

#### Scenario: Equal-metric scenarios ordered by ID
- **WHEN** the active sort is "Loss" or "Dice" and two scenarios share the exact same metric value
- **THEN** the scenario whose identifier is alphabetically earlier appears first
