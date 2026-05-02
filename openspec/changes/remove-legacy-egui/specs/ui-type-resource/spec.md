## REMOVED Requirements

### Requirement: A UI backend selector controls which UI systems are active

**Reason**: The legacy alternate UI backend is being removed, so there is no longer a user-visible or system-visible choice between UI backends.
**Migration**: Remove any behavior that branches on UI backend selection and run the single supported UI path unconditionally.

#### Scenario: Legacy backend selection is no longer available

- **WHEN** the application runs after this change
- **THEN** there is no selectable UI backend state exposed to the system or the user

### Requirement: Switching UI backends takes effect on the next frame

**Reason**: Runtime backend switching exists only to support toggling between the retained UI and the deleted legacy UI.
**Migration**: Remove runtime backend-toggle controls and keep the application on the single supported UI path.

#### Scenario: Runtime backend switching is unavailable

- **WHEN** the application is running after this change
- **THEN** there is no interaction that switches to an alternate UI backend on a later frame

### Requirement: The UI backend selector is accessible to any system

**Reason**: A globally accessible backend selector is unnecessary once the application no longer supports multiple UI backends.
**Migration**: Replace backend-conditional logic with the single-path UI behavior, and remove reads and writes of the deleted selector.

#### Scenario: Systems do not inspect backend selection

- **WHEN** a system participates in UI behavior after this change
- **THEN** it does not rely on a UI backend selector to determine whether the active UI should render

### Requirement: Pressing F2 toggles the active UI backend

**Reason**: The F2 shortcut only exists to move between the Bevy-native UI and the deleted legacy egui UI.
**Migration**: Remove the backend-toggle shortcut and any user guidance that advertises it.

#### Scenario: F2 no longer switches UI backend

- **WHEN** the user presses F2 after this change
- **THEN** the active UI backend does not change because no alternate backend exists
