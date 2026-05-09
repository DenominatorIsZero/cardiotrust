## MODIFIED Requirements

### Requirement: Individual on-demand generation
The user SHALL be able to trigger generation for a single image by clicking its "Generate" button.

#### Scenario: Generate button triggers background generation
- **WHEN** the user clicks the "Generate" button on a not-generated card
- **THEN** the card SHALL immediately transition to the generating state and generation SHALL proceed in the background without blocking the UI

#### Scenario: Completion transitions card to done
- **WHEN** background generation for an image completes successfully
- **THEN** the card SHALL transition to the done state and display the thumbnail

#### Scenario: Web generation does not require persisted files
- **WHEN** the application is running as a web build and the user generates a result image
- **THEN** the generated thumbnail SHALL become viewable in the gallery without requiring a durable output file to exist

### Requirement: Action bar with secondary operations
A persistent action bar SHALL provide access to export the selected scenario's payload as NumPy arrays, a global playback speed setting, and animation export options where those actions are supported by the current build's storage capabilities. All durable exports SHALL be written into the selected scenario's persisted output area for the currently opened project.

#### Scenario: Export to .npy on durable-storage builds
- **WHEN** the user activates "Export to .npy" while working in a project with durable storage
- **THEN** the selected scenario's payload SHALL be exported as NumPy arrays into that scenario's persisted output area

#### Scenario: Export to .npy is unavailable on web builds
- **WHEN** the application is running as a web build without durable project storage
- **THEN** the "Export to .npy" action SHALL be hidden or disabled

#### Scenario: Playback speed setting
- **WHEN** the user adjusts the playback speed control
- **THEN** the selected speed value (range 0.001 to 0.1) SHALL be used for all subsequent animation generation and playback operations

#### Scenario: Export Animation controls active when animation card is done and export is supported
- **WHEN** an animation card is in the done state and the current build supports durable animation export
- **THEN** the action bar SHALL make "Export as APNG" and "Export as MP4" actions available

#### Scenario: Export Animation controls inactive when no done animation exists
- **WHEN** no animation card is in the done state
- **THEN** the "Export as APNG" and "Export as MP4" actions SHALL be disabled

#### Scenario: Export Animation controls are unavailable on web builds without durable export support
- **WHEN** the application is running as a web build without durable animation export support
- **THEN** the animation export actions SHALL be hidden or disabled regardless of card state
