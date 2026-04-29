## Requirements

### Requirement: Gallery shows all image types grouped in category tabs
The results view SHALL display all image types organized into four category tabs: Spatial Maps, Metrics, Losses, and Time Functions. Each tab SHALL show only the image types belonging to that category.

#### Scenario: Navigating to Spatial Maps tab
- **WHEN** the user selects the Spatial Maps tab
- **THEN** the gallery SHALL show cards for States Max (Algorithm/Simulation/Delta), Activation Time (Algorithm/Simulation/Delta), Voxel Types (Algorithm/Simulation/Prediction), Average Delay (Simulation/Algorithm/Delta), Average Propagation Speed (Simulation/Algorithm), and the two GIF animation cards (Algorithm States, Simulation States)

#### Scenario: Navigating to Metrics tab
- **WHEN** the user selects the Metrics tab
- **THEN** the gallery SHALL show cards for Dice, IoU, Recall, and Precision

#### Scenario: Navigating to Losses tab
- **WHEN** the user selects the Losses tab
- **THEN** the gallery SHALL show cards for Loss (full), Loss (per epoch), MSE Loss (full), MSE Loss (per epoch), Max Regularization (full), and Max Regularization (per epoch)

#### Scenario: Navigating to Time Functions tab
- **WHEN** the user selects the Time Functions tab
- **THEN** the gallery SHALL show cards for Control Function (Algorithm/Simulation/Delta), State (Algorithm/Simulation/Delta), and Measurement (Algorithm/Simulation/Delta)

---

### Requirement: Each image is represented by a thumbnail card
Each image type SHALL be displayed as a card containing a title (image type name), a subtitle (variant name), and a thumbnail area.

#### Scenario: Card displays title and subtitle
- **WHEN** a card is rendered
- **THEN** the title SHALL show the image type name and the subtitle SHALL show the variant in a secondary color

#### Scenario: Card thumbnail area has consistent dimensions
- **WHEN** any card is rendered
- **THEN** the thumbnail area SHALL maintain a fixed 4:3 aspect ratio and a height of approximately 200px

---

### Requirement: Cards reflect per-image generation state
Each card SHALL display one of four states: not-generated, generating, done, or failed.

#### Scenario: Not-generated state
- **WHEN** an image has not been generated
- **THEN** the card SHALL show a "Generate" button centered on a dimmed background

#### Scenario: Generating state
- **WHEN** an image generation is in progress
- **THEN** the card SHALL show a spinner with "Generating..." text and the Generate button SHALL be disabled

#### Scenario: Done state
- **WHEN** an image has been successfully generated
- **THEN** the card SHALL show the image (or current animation frame for GIFs) in the thumbnail area

#### Scenario: Failed state
- **WHEN** an image generation has failed
- **THEN** the card SHALL show an error icon with a retry button

---

### Requirement: On-demand generation — no images auto-generate on view load
When the Results view is opened for a scenario, no images SHALL be generated automatically.

#### Scenario: Results view opens with no images
- **WHEN** the user navigates to the Results view for any scenario
- **THEN** all cards SHALL be in the not-generated state (showing "Generate" buttons)

#### Scenario: Switching scenarios resets generation state
- **WHEN** the user switches to a different scenario
- **THEN** all cards SHALL reset to not-generated state

---

### Requirement: Individual on-demand generation
The user SHALL be able to trigger generation for a single image by clicking its "Generate" button.

#### Scenario: Generate button triggers background generation
- **WHEN** the user clicks the "Generate" button on a not-generated card
- **THEN** the card SHALL immediately transition to the generating state and generation SHALL proceed in the background without blocking the UI

#### Scenario: Completion transitions card to done
- **WHEN** background generation for an image completes successfully
- **THEN** the card SHALL transition to the done state and display the thumbnail

---

### Requirement: Batch generation within a tab
A "Generate All in Tab" action SHALL generate all not-yet-generated images in the currently active category tab.

#### Scenario: Generate All in Tab triggers all pending cards
- **WHEN** the user activates "Generate All in Tab"
- **THEN** all cards in the current tab that are in not-generated or failed state SHALL transition to generating state

#### Scenario: Already-generating or done cards are skipped
- **WHEN** the user activates "Generate All in Tab" while some cards are already generating or done
- **THEN** those cards SHALL NOT be re-triggered

---

### Requirement: Global batch generation
A "Generate All" action SHALL generate all not-yet-generated images across all category tabs.

#### Scenario: Generate All triggers all pending images
- **WHEN** the user activates "Generate All"
- **THEN** all cards across all tabs that are in not-generated or failed state SHALL transition to generating state

---

### Requirement: Generation progress indicator
While any generation is in progress, the UI SHALL show a live counter indicating how many images have completed out of the total being generated.

#### Scenario: Progress counter displays during batch generation
- **WHEN** batch generation is in progress
- **THEN** the toolbar SHALL display a counter in the format "Generating N/M..." where N is completed and M is total in the current batch

#### Scenario: Progress counter disappears when done
- **WHEN** all images in the batch have completed (successfully or with failure)
- **THEN** the progress counter SHALL no longer be displayed

---

### Requirement: Responsive grid layout
The gallery grid SHALL adapt its column count based on available panel width.

#### Scenario: Wide panel shows 3 columns
- **WHEN** the available panel width is 1200px or more
- **THEN** the gallery SHALL display 3 cards per row

#### Scenario: Medium panel shows 2 columns
- **WHEN** the available panel width is between 800px and 1199px
- **THEN** the gallery SHALL display 2 cards per row

#### Scenario: Narrow panel shows 1 column
- **WHEN** the available panel width is less than 800px
- **THEN** the gallery SHALL display 1 card per row

---

### Requirement: Card hover highlight
Cards SHALL provide a subtle visual affordance on hover to indicate interactivity.

#### Scenario: Hovering a card
- **WHEN** the user moves the cursor over a card
- **THEN** the card SHALL display a highlighted border or background change

---

### Requirement: Action bar with secondary operations
A persistent action bar SHALL provide access to Export to .npy, a global playback speed setting, and animation export options.

#### Scenario: Export to .npy
- **WHEN** the user activates "Export to .npy"
- **THEN** the scenario data SHALL be exported as NumPy arrays to the scenario's output directory

#### Scenario: Playback speed setting
- **WHEN** the user adjusts the playback speed control
- **THEN** the selected speed value (range 0.001 to 0.1) SHALL be used for all subsequent animation generation and playback operations

#### Scenario: Export Animation controls active when animation card is done
- **WHEN** an animation card is in the done state
- **THEN** the action bar SHALL make "Export as APNG" and "Export as MP4" actions available

#### Scenario: Export Animation controls inactive when no done animation exists
- **WHEN** no animation card is in the done state
- **THEN** the "Export as APNG" and "Export as MP4" actions SHALL be disabled
