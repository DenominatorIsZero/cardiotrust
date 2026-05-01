## MODIFIED Requirements

### Requirement: Action bar with secondary operations

A persistent action bar SHALL provide access to export the selected scenario's payload as NumPy arrays, a global playback speed setting, and animation export options. All exports SHALL be written into the selected scenario's persisted output area for the currently opened project.

#### Scenario: Export to .npy
- **WHEN** the user activates "Export to .npy"
- **THEN** the selected scenario's payload SHALL be exported as NumPy arrays into that scenario's persisted output area

#### Scenario: Playback speed setting
- **WHEN** the user adjusts the playback speed control
- **THEN** the selected speed value (range 0.001 to 0.1) SHALL be used for all subsequent animation generation and playback operations

#### Scenario: Export Animation controls active when animation card is done
- **WHEN** an animation card is in the done state
- **THEN** the action bar SHALL make "Export as APNG" and "Export as MP4" actions available

#### Scenario: Export Animation controls inactive when no done animation exists
- **WHEN** no animation card is in the done state
- **THEN** the "Export as APNG" and "Export as MP4" actions SHALL be disabled
