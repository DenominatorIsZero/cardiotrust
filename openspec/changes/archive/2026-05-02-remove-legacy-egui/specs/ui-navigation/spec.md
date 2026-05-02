## MODIFIED Requirements

### Requirement: Exactly one content view is active at all times

The application SHALL have exactly one active content view at any time, selected from: Home, Explorer, Scenario, Results, Volumetric, and Scheduler. The application SHALL start on the Home view. Switching views is instantaneous.

#### Scenario: Only one content panel renders per frame

- **WHEN** the application is in any active view state
- **THEN** exactly one content panel occupies the main content area of the screen

#### Scenario: Navigation shell is always visible

- **WHEN** the application is in any view state
- **THEN** the persistent navigation shell is rendered

#### Scenario: Application starts on Home view

- **WHEN** the application is launched
- **THEN** the initial view is Home
