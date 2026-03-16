## ADDED Requirements

### Requirement: The content area fills the space to the right of the sidebar

When the Bevy UI backend is active, the content area SHALL occupy all horizontal space not taken by the sidebar rail and SHALL span the full viewport height. The content area layout SHALL not be affected by whether the sidebar is expanded or collapsed — it always fills the remaining space.

#### Scenario: Content area expands when sidebar collapses

- **WHEN** the sidebar transitions to collapsed mode
- **THEN** the content area widens to fill the space vacated by the sidebar

#### Scenario: Content area shrinks when sidebar expands

- **WHEN** the sidebar transitions to expanded mode
- **THEN** the content area narrows to maintain the full-screen coverage invariant

### Requirement: A breadcrumb context bar occupies the top of the content area

The content area SHALL display a thin context bar at its top. The context bar SHALL show the current location as a path of view names, with the final segment visually distinguished. Each non-final segment in the path SHALL be activatable to navigate directly to that ancestor view.

#### Scenario: Breadcrumb reflects the active view

- **WHEN** the active view changes
- **THEN** the breadcrumb bar updates to show a path ending with the new active view name

#### Scenario: Activating a breadcrumb ancestor navigates to that view

- **WHEN** the user activates a non-final segment of the breadcrumb path
- **THEN** the application navigates to the corresponding ancestor view

### Requirement: The content slot below the breadcrumb bar hosts the active view's content

Below the breadcrumb bar there SHALL be a content slot that fills the remainder of the content area. The content slot's children SHALL reflect the active view. When no view-specific content is registered for the active view, the content slot SHALL display an empty placeholder.

#### Scenario: Content slot is empty for views not yet implemented

- **WHEN** the Bevy backend is active and the active view has no registered Bevy content
- **THEN** the content slot is blank (no error, no crash, no egui panels)

#### Scenario: Content slot updates when the active view changes

- **WHEN** the active view transitions to a different view
- **THEN** the content slot reflects the new view's registered content
