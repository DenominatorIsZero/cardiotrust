## ADDED Requirements

### Requirement: A UI backend selector controls which UI systems are active

The application SHALL maintain a globally accessible UI backend selector with exactly two states: `EGui` and `Bevy`. At any given frame, only the systems belonging to the currently selected backend SHALL execute. The default selection SHALL be `EGui`.

#### Scenario: EGui systems run when EGui backend is selected

- **WHEN** the UI backend selector is set to `EGui`
- **THEN** all EGUI draw systems execute normally each frame

#### Scenario: Bevy systems run when Bevy backend is selected

- **WHEN** the UI backend selector is set to `Bevy`
- **THEN** Bevy native UI systems execute and EGUI draw systems do not

#### Scenario: Default selection is EGui

- **WHEN** the application starts without any explicit UI backend configuration
- **THEN** the UI backend selector is `EGui` and application behavior is identical to before this change

### Requirement: Switching UI backends takes effect on the next frame

Changing the UI backend selector SHALL take effect starting with the next rendered frame. There is no transition animation, loading state, or confirmation step.

#### Scenario: Backend switch is immediate

- **WHEN** the UI backend selector value is changed
- **THEN** the new backend's systems run beginning with the immediately following frame

### Requirement: The UI backend selector is accessible to any system

Any Bevy system SHALL be able to read or write the UI backend selector without special permissions or indirection. Both UI backend systems and non-UI systems (e.g., a debug panel or automated test harness) SHALL be able to inspect or modify the selector.

#### Scenario: Non-UI system can read the current backend

- **WHEN** any system queries the UI backend selector
- **THEN** it receives the current selection without error

### Requirement: Pressing F2 toggles the active UI backend

The application SHALL toggle the UI backend selector between `EGui` and `Bevy` each time the F2 key is pressed. This provides an immediate development feedback loop without requiring a restart or code change.

#### Scenario: F2 switches from EGui to Bevy

- **WHEN** the UI backend selector is `EGui` and the user presses F2
- **THEN** the UI backend selector becomes `Bevy` on the next frame

#### Scenario: F2 switches from Bevy to EGui

- **WHEN** the UI backend selector is `Bevy` and the user presses F2
- **THEN** the UI backend selector becomes `EGui` on the next frame
