## Requirements

### Requirement: Animation types appear as first-class gallery cards
Animation types SHALL appear as cards in the gallery (in the Spatial Maps tab) with the same four states as static image cards: not-generated, generating, done, and failed. There SHALL be no separate "Generate Animation" buttons outside the card itself.

#### Scenario: Algorithm States animation card in gallery
- **WHEN** the Spatial Maps tab is active
- **THEN** the gallery SHALL include a card titled "Algorithm States" with subtitle "Animation"

#### Scenario: Simulation States animation card in gallery
- **WHEN** the Spatial Maps tab is active
- **THEN** the gallery SHALL include a card titled "Simulation States" with subtitle "Animation"

#### Scenario: Animation not yet generated
- **WHEN** an animation has not been generated
- **THEN** its card SHALL show a "Generate" button (same as static image not-generated state)

#### Scenario: Animation generating
- **WHEN** animation generation is in progress
- **THEN** its card SHALL show a spinner with "Generating..." text

#### Scenario: Animation generation complete
- **WHEN** animation generation completes and all frames are loaded into memory
- **THEN** its card SHALL transition to the done state and begin displaying the animation

---

### Requirement: Animation frames stored as a PNG sequence on disk
The system SHALL store each generated animation as a sequence of PNG frames in the selected scenario's persisted output area. Later visits to that same stored scenario within the same project SHALL be able to discover and reuse the existing frame sequence without regeneration.

#### Scenario: Frame files written to disk on generation
- **WHEN** animation generation completes successfully
- **THEN** the selected scenario's persisted output area SHALL contain sequentially numbered PNG files, one per frame

#### Scenario: Existing frame sequence detected on load
- **WHEN** the Results view loads for a stored scenario that already has a generated animation in the current project
- **THEN** the animation card SHALL be able to load the existing frame PNGs without re-generating

#### Scenario: Animation frames are isolated by project
- **WHEN** two different projects each contain a scenario with the same identifier
- **THEN** loading one project SHALL only expose animation frames generated for that scenario in that same project

---

### Requirement: Animation generation triggered from the card
The "Generate" button on an animation card SHALL trigger frame sequence generation in the background using the current playback speed setting.

#### Scenario: Generate animation from card button
- **WHEN** the user clicks the "Generate" button on an animation card
- **THEN** frame generation SHALL begin in the background using the current global playback speed value

---

### Requirement: Animation thumbnail shows live playback
When an animation card is in the done state, the thumbnail area SHALL display the animation playing in a loop at the current playback speed.

#### Scenario: Animated thumbnail loops
- **WHEN** an animation card is in the done state
- **THEN** the thumbnail area SHALL continuously cycle through all frames in order, looping back to frame 1 after the last frame

#### Scenario: Playback speed change updates animation rate
- **WHEN** the user changes the global playback speed setting
- **THEN** all playing animation thumbnails SHALL update their frame advance rate immediately

---

### Requirement: Animation card shows playback controls when done
When an animation card is in the done state, it SHALL display playback controls below the thumbnail: a play/pause toggle and, when paused, a frame scrubber.

#### Scenario: Play/pause toggle present
- **WHEN** the animation card is in the done state
- **THEN** the card SHALL show a play/pause toggle button below the thumbnail

#### Scenario: Toggle pauses playback
- **WHEN** the user clicks the toggle while the animation is playing
- **THEN** the animation SHALL pause on the current frame

#### Scenario: Toggle resumes playback
- **WHEN** the user clicks the toggle while the animation is paused
- **THEN** the animation SHALL resume from the current frame

---

### Requirement: Frame scrubber shown only when paused
When the animation is paused, the card SHALL display a frame scrubber. When playing, the frame scrubber SHALL be hidden.

#### Scenario: Scrubber hidden during playback
- **WHEN** the animation is playing
- **THEN** the frame scrubber area SHALL NOT be visible

#### Scenario: Scrubber visible when paused
- **WHEN** the animation is paused
- **THEN** the card SHALL display the current frame number, a decrement button (-), and an increment button (+)

#### Scenario: Increment button advances one frame
- **WHEN** the animation is paused and the user clicks the increment (+) button
- **THEN** the displayed frame SHALL advance by one; wrapping from the last frame to frame 1

#### Scenario: Decrement button retreats one frame
- **WHEN** the animation is paused and the user clicks the decrement (-) button
- **THEN** the displayed frame SHALL retreat by one; wrapping from frame 1 to the last frame

#### Scenario: Frame number is directly editable when paused
- **WHEN** the animation is paused and the user clicks the frame number display
- **THEN** the display SHALL become an editable text input accepting a numeric frame index

#### Scenario: Entering a valid frame number jumps to that frame
- **WHEN** the user submits a frame number within the valid range
- **THEN** the animation SHALL display that frame

#### Scenario: Entering an out-of-range frame number is clamped
- **WHEN** the user submits a frame number outside the valid range
- **THEN** the value SHALL be clamped to the nearest valid frame index

---

### Requirement: Animation opens in full-size modal with playback controls
Clicking a done animation card SHALL open the modal viewer at full size, with the same playback controls (play/pause, frame scrubber when paused) available in the modal.

#### Scenario: Click animation card opens modal
- **WHEN** the user clicks an animation card in the done state
- **THEN** the modal SHALL open displaying the animation at full available size

#### Scenario: Modal playback state is independent of thumbnail
- **WHEN** the modal is open
- **THEN** the modal's play/pause state and frame position SHALL be independent of the thumbnail card's state

#### Scenario: Modal frame scrubber mirrors thumbnail behavior
- **WHEN** the animation in the modal is paused
- **THEN** the modal SHALL display the frame scrubber (frame number, - and + buttons, editable input) with identical behavior to the thumbnail card

---

### Requirement: Animation export as APNG
The system SHALL allow the user to export a generated animation as an Animated PNG file into the selected scenario's persisted output area.

#### Scenario: Export as APNG succeeds
- **WHEN** the user activates "Export as APNG" for a done animation card
- **THEN** the system SHALL assemble all frames into an APNG file and write it into the selected scenario's persisted output area

#### Scenario: APNG export runs in background
- **WHEN** APNG export is triggered
- **THEN** the UI SHALL remain responsive and show an export-in-progress indicator

#### Scenario: APNG export completion notified
- **WHEN** APNG export completes
- **THEN** the UI SHALL indicate success and show the output file path

---

### Requirement: Animation export as MP4 via ffmpeg
The system SHALL allow the user to export a generated animation as an MP4 file into the selected scenario's persisted output area when ffmpeg is available on the system.

#### Scenario: Export as MP4 with ffmpeg available
- **WHEN** the user activates "Export as MP4" and ffmpeg is installed on the system
- **THEN** the system SHALL invoke ffmpeg with the frame sequence as input and write an MP4 file into the selected scenario's persisted output area

#### Scenario: Export as MP4 without ffmpeg
- **WHEN** the user activates "Export as MP4" and ffmpeg is not found on the system
- **THEN** the system SHALL display a clear error message indicating that ffmpeg is not installed and is required for MP4 export

#### Scenario: MP4 export runs in background
- **WHEN** MP4 export is triggered and ffmpeg is available
- **THEN** the UI SHALL remain responsive and show an export-in-progress indicator

---

### Requirement: Animation generation failure is surfaced to the user
If animation generation fails, the card SHALL show the failed state with a retry option.

#### Scenario: Animation generation fails
- **WHEN** an animation generation job encounters an error
- **THEN** the card SHALL transition to the failed state displaying an error indicator and a retry button

#### Scenario: Retry animation generation
- **WHEN** the user clicks the retry button on a failed animation card
- **THEN** animation generation SHALL be re-triggered in the background
