## MODIFIED Requirements

### Requirement: Exactly one content view is active at all times

The application SHALL have exactly one active content view at any time, selected from: Explorer, Scenario, Results, and Volumetric. The top navigation bar is always visible regardless of the active view. Switching views is instantaneous — the outgoing view is fully replaced by the incoming view. These behaviors apply only when the EGUI backend is active; when the Bevy backend is active, the view layout is governed by the Bevy UI systems instead.

#### Scenario: Only one content panel renders per frame

- **WHEN** the application is in any active view state and the EGUI backend is selected
- **THEN** exactly one content panel occupies the central area of the screen

#### Scenario: Navigation bar is always visible

- **WHEN** the application is in any view state and the EGUI backend is selected
- **THEN** the top navigation bar is rendered

#### Scenario: No EGUI panels render when Bevy backend is selected

- **WHEN** the UI backend selector is set to `Bevy`
- **THEN** no EGUI draw systems execute and no EGUI panels appear on screen
