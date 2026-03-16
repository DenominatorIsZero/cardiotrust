## MODIFIED Requirements

### Requirement: Exactly one content view is active at all times

The application SHALL have exactly one active content view at any time, selected from: Home, Explorer, Scenario, Results, Volumetric, and Scheduler. When the EGUI backend is active, only the four original views (Explorer, Scenario, Results, Volumetric) are reachable; the top navigation bar is always visible. When the Bevy backend is active, all six views are reachable via the sidebar rail. Switching views is instantaneous — the outgoing view is fully replaced by the incoming view.

#### Scenario: Only one content panel renders per frame under EGui

- **WHEN** the application is in any active view state and the EGUI backend is selected
- **THEN** exactly one content panel occupies the central area of the screen

#### Scenario: Navigation bar is always visible under EGui

- **WHEN** the application is in any view state and the EGUI backend is selected
- **THEN** the top navigation bar is rendered

#### Scenario: No EGUI panels render when Bevy backend is selected

- **WHEN** the UI backend selector is set to `Bevy`
- **THEN** no EGUI draw systems execute and no EGUI panels appear on screen

#### Scenario: Only one view is active at any time under Bevy backend

- **WHEN** the UI backend selector is set to `Bevy`
- **THEN** exactly one of the six views (Home, Explorer, Scenario, Results, Volumetric, Scheduler) is the active view
