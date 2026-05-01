## ADDED Requirements

### Requirement: The Volumetric view prioritizes the 3-D viewport

When the Volumetric view is active, the main workspace SHALL devote all space not used by the context bar, control overlay, or signal plot to the 3-D viewport. The 3-D viewport SHALL remain the dominant visual element of the screen.

#### Scenario: Viewport fills the main workspace
- **WHEN** the Volumetric view is active and no fullscreen override is enabled
- **THEN** the 3-D viewport occupies the central workspace and expands to fill all remaining available space

#### Scenario: Control panel does not permanently reserve screen width when collapsed
- **WHEN** the control overlay is collapsed
- **THEN** only the section tab strip remains visible at the edge of the viewport and the viewport retains the rest of the workspace width

### Requirement: The Volumetric view provides a collapsible right-edge control overlay

The Volumetric view SHALL expose four control sections from a right-edge overlay: voxel coloring, visibility, cutting plane, and sensor bracket. The overlay SHALL be collapsed by default and SHALL expand to show one active section at a time.

#### Scenario: Activating a section expands the overlay
- **WHEN** the user activates one of the section tabs while the overlay is collapsed
- **THEN** the overlay expands and displays the controls for that section

#### Scenario: Activating the current section collapses the overlay
- **WHEN** the user activates the tab for the section that is already open
- **THEN** the overlay collapses back to the tab-only state

#### Scenario: Clicking outside the overlay closes it
- **WHEN** the overlay is open and the user activates a point outside the overlay
- **THEN** the overlay collapses

### Requirement: The control overlay exposes the current volumetric controls

The control overlay SHALL provide controls for the current volumetric scene state: color mode, relative coloring, playback speed, manual playback mode, sample selection, acquisition beat selection, sensor selection, visibility toggles for major scene elements, cutting-plane enablement and parameters, and sensor-bracket position and radius.

#### Scenario: Manual sample selection is gated by manual mode
- **WHEN** manual playback mode is disabled
- **THEN** the sample-selection control is visible but inactive

#### Scenario: Visibility toggles change scene visibility
- **WHEN** the user changes a visibility toggle for a scene element
- **THEN** that element's visible state updates immediately in the Volumetric view

#### Scenario: Cutting-plane parameter changes affect clipping
- **WHEN** the user changes the cutting-plane enabled state, origin, normal, or opacity
- **THEN** the visible scene updates to reflect the new cutting-plane configuration

### Requirement: The Volumetric view provides a resizable signal plot panel

The Volumetric view SHALL display a signal plot panel along the bottom edge of the workspace. The panel SHALL support vertical resizing and SHALL support collapsing to a minimized bar and restoring to its previous height.

#### Scenario: Plot starts expanded on wide screens
- **WHEN** the Volumetric view is opened on a screen at or above the narrow-screen breakpoint
- **THEN** the signal plot panel is visible at its default expanded height

#### Scenario: Plot starts collapsed on narrow screens
- **WHEN** the Volumetric view is opened on a screen below the narrow-screen breakpoint
- **THEN** the signal plot panel starts in its collapsed state

#### Scenario: Collapsing and restoring preserves usability
- **WHEN** the user collapses and later restores the signal plot panel
- **THEN** the plot returns and the viewport reclaims and then releases the corresponding vertical space

### Requirement: The signal plot supports manual sample inspection

The signal plot SHALL show the currently selected measurement waveform together with a cursor marking the current sample time. When manual playback mode is enabled, the user SHALL be able to select a sample time from the plot.

#### Scenario: Plot shows waveform and current cursor
- **WHEN** the Volumetric view is active and signal data is available
- **THEN** the plot displays the current waveform and a cursor indicating the current sample time

#### Scenario: Clicking the plot sets the current sample in manual mode
- **WHEN** manual playback mode is enabled and the user selects a point on the plot
- **THEN** the current sample time updates to the selected position

#### Scenario: Plot selection is inert outside manual mode
- **WHEN** manual playback mode is disabled and the user selects a point on the plot
- **THEN** the current sample time does not change

### Requirement: The Volumetric view supports fullscreen and responsive overlay behavior

The Volumetric view SHALL support a fullscreen mode that maximizes viewport space by hiding secondary panels. On narrower screens, the control overlay SHALL behave as an overlaid panel instead of a side-attached utility surface.

#### Scenario: Fullscreen hides secondary panels
- **WHEN** the user enables fullscreen mode
- **THEN** the control overlay and signal plot are hidden and the viewport expands to use the reclaimed space

#### Scenario: Leaving fullscreen restores prior panel state
- **WHEN** the user disables fullscreen mode after panels were hidden by fullscreen
- **THEN** the control overlay and signal plot return according to their prior collapsed or expanded states

#### Scenario: Narrow screens use an overlay-first control panel
- **WHEN** the Volumetric view is shown below the control-panel breakpoint
- **THEN** the control panel opens as an overlay over the viewport rather than reserving a side region
