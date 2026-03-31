## ADDED Requirements

### Requirement: Full-size modal viewer opens on thumbnail click
Clicking a generated static image thumbnail SHALL open a full-size modal overlay displaying the image at maximum available resolution within the panel. GIF cards have their own modal behavior defined in the `results-gif-viewer` spec.

#### Scenario: Click generated thumbnail opens modal
- **WHEN** the user clicks on a static image card whose image is in the done state
- **THEN** a modal overlay SHALL appear displaying the image at full resolution, rendered on top of the gallery with a semi-transparent backdrop

#### Scenario: Click not-generated thumbnail does not open modal
- **WHEN** the user clicks on a card whose image is not in the done state
- **THEN** no modal SHALL open

---

### Requirement: Modal displays image title
The modal SHALL display the image type name and variant in its header.

#### Scenario: Title shown in modal header
- **WHEN** the modal is open
- **THEN** the header SHALL show the image type name and variant (e.g., "States Max (Algorithm)")

---

### Requirement: Modal maintains image aspect ratio
The image displayed in the modal SHALL fill the available space while maintaining its original aspect ratio.

#### Scenario: Image scales without distortion
- **WHEN** the modal is displayed at any size
- **THEN** the image SHALL scale to fill the modal content area while preserving its aspect ratio with no stretching or cropping

---

### Requirement: Navigation between images within the same tab
The modal SHALL allow the user to navigate to the previous or next image in the same category tab. Navigation skips GIF cards and skips cards that are not in the done state.

#### Scenario: Next image via button
- **WHEN** the user clicks the "Next" button
- **THEN** the modal SHALL display the next done static image in the current category tab

#### Scenario: Previous image via button
- **WHEN** the user clicks the "Prev" button
- **THEN** the modal SHALL display the previous done static image in the current category tab

#### Scenario: Next image via right arrow key
- **WHEN** the modal is open and the user presses the right arrow key
- **THEN** the modal SHALL advance to the next done static image in the tab

#### Scenario: Previous image via left arrow key
- **WHEN** the modal is open and the user presses the left arrow key
- **THEN** the modal SHALL go back to the previous done static image in the tab

#### Scenario: Navigation wraps at boundaries
- **WHEN** the user navigates past the last image in the tab
- **THEN** the modal SHALL wrap to the first done image; navigating before the first SHALL wrap to the last done image

---

### Requirement: Position indicator in modal footer
The modal SHALL display the current image's position within the navigable images in the category tab.

#### Scenario: Position indicator shows index
- **WHEN** the modal is open
- **THEN** the footer SHALL display the current position and total navigable count (e.g., "3 / 12")

---

### Requirement: Modal dismissal
The modal SHALL be dismissible via the X button or the Escape key.

#### Scenario: Close via X button
- **WHEN** the user clicks the X button in the modal header
- **THEN** the modal SHALL close and the gallery SHALL be visible again

#### Scenario: Close via Escape key
- **WHEN** the modal is open and the user presses the Escape key
- **THEN** the modal SHALL close and the gallery SHALL be visible again

---

### Requirement: Modal is an overlay, not a separate view
The modal SHALL render on top of the gallery without navigating away from the Results view.

#### Scenario: Gallery is accessible after closing modal
- **WHEN** the user closes the modal
- **THEN** the gallery SHALL be displayed in the same state it was in before the modal was opened (same tab, same scroll position)
