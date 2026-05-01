## MODIFIED Requirements

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

## MODIFIED Requirements

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

## MODIFIED Requirements

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
