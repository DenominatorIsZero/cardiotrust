## Purpose

Governs the real-time 3-D volumetric visualization of cardiac simulation data — scene construction from scenario data, animated color-coded rendering of cardiac current density and activation times across ten distinct color modes, cutting-plane clipping, sensor position display, and color scaling behavior.

This spec is distinct from `ui-navigation` (which governs page routing and the 2-D results panel showing static plot images) and from `data-simulation` / `inverse-algorithm` (which govern the numerical data being visualized). The visualization domain has no side effects on simulation data; it is a read-only consumer of completed scenario results.

## ADDED Requirements

### Requirement: The 3-D scene represents only connectable cardiac voxels

Only voxels that can carry an electrical activation SHALL be rendered as 3-D elements in the cardiac scene. Voxels of non-connectable types (such as torso cavity, blood chamber, or empty space) SHALL be absent from the rendered scene.

#### Scenario: Only connectable voxels appear in the scene

- **WHEN** a cardiac model is loaded into the visualization
- **THEN** the number of rendered voxel elements equals the number of connectable voxels in the model

#### Scenario: Changing to a model with more connectable voxels renders more elements

- **WHEN** a scenario with more connectable voxels replaces a previously loaded scenario
- **THEN** the number of rendered voxel elements increases accordingly

### Requirement: Scene is fully rebuilt when a new scenario is loaded

When a scenario is selected for volumetric viewing, all previously rendered voxels and sensors SHALL be removed and replaced with elements corresponding to the newly selected scenario. No artifacts from a previous scenario SHALL persist after a full scene rebuild.

#### Scenario: Loading a new scenario removes all previous voxel entities

- **WHEN** a second scenario is loaded into the visualization after a first
- **THEN** no voxel entities from the first scenario remain visible

#### Scenario: Sensor positions are updated to match the new scenario

- **WHEN** a new scenario is loaded with a different sensor array configuration
- **THEN** sensor position markers reflect the new scenario's sensor geometry

### Requirement: Voxel colors update in real time as the animation advances

The visualization SHALL animate the time evolution of cardiac current density by cycling through time steps. At each animation step, every voxel's rendered color SHALL correspond to the quantity selected by the active color mode at the current time step. Animation advances continuously based on wall-clock time and the configured playback speed.

#### Scenario: Voxel colors change as time advances

- **WHEN** the animation is playing and at least one time step elapses
- **THEN** the colors of at least some voxels change to reflect the new time step

#### Scenario: Pausing animation freezes voxel colors

- **WHEN** the animation is paused
- **THEN** voxel colors remain constant across subsequent render frames until animation resumes

### Requirement: Ten distinct color modes are available for voxel rendering

The visualization SHALL support ten color modes covering anatomical tissue classification (from simulation and estimation), per-time-step current density magnitude (from simulation and estimation), per-voxel peak current density magnitude (from simulation, estimation, and their difference), and per-voxel activation time (from simulation, estimation, and their difference).

#### Scenario: Selecting an anatomical color mode renders fixed semantic colors per tissue type

- **WHEN** an anatomical voxel-type color mode is active
- **THEN** all voxels of the same tissue type share the same color, independent of the current time step

#### Scenario: Selecting a time-resolved color mode shows animation

- **WHEN** a per-time-step current density color mode is active
- **THEN** voxel colors change as the animation advances through time steps

#### Scenario: Selecting an activation-time color mode renders static colors

- **WHEN** an activation-time color mode is active
- **THEN** voxel colors reflect per-voxel activation time and do not change as the animation advances

### Requirement: A cutting plane clips the visible voxel set

The visualization SHALL provide a configurable cutting plane that hides all voxels on the far side of the plane. The plane's position and orientation SHALL be adjustable at runtime. Only voxels on the near side of the plane are visible; voxels on the far side are hidden regardless of their tissue type or color mode.

#### Scenario: Voxels behind the cutting plane are not visible

- **WHEN** the cutting plane is positioned such that it intersects the cardiac model
- **THEN** voxels on the far side of the plane are not rendered

#### Scenario: Disabling the cutting plane reveals all voxels

- **WHEN** the cutting plane is disabled
- **THEN** all connectable voxels that are not hidden by other visibility settings are rendered

### Requirement: Sensor positions reflect the current acquisition beat

For multi-position sensor array scenarios, the visualization SHALL display sensors at the positions corresponding to the currently selected acquisition beat. Changing the selected beat SHALL immediately reposition all sensor markers and the sensor array bracket.

#### Scenario: Sensors are at beat-specific positions

- **WHEN** a multi-beat scenario is loaded and a specific beat is selected
- **THEN** sensor markers are at the positions recorded for that beat

#### Scenario: Changing the selected beat repositions sensor markers

- **WHEN** the selected acquisition beat is changed from beat A to beat B
- **THEN** all sensor markers move to the positions corresponding to beat B

### Requirement: Relative color scaling normalizes colors to the current data range

When relative color scaling is enabled, the color-to-value mapping SHALL span the range of the currently displayed data — minimum to maximum — rather than an absolute fixed scale. This allows low-contrast features to be visible even when absolute values are small.

#### Scenario: Relative scaling adjusts when different scenarios are loaded

- **WHEN** relative color scaling is enabled and a second scenario with different data magnitude is loaded
- **THEN** the color scale adjusts to span the new scenario's data range

#### Scenario: Absolute scaling is unaffected by data range

- **WHEN** absolute color scaling is enabled
- **THEN** the same value always maps to the same color regardless of which scenario is loaded
