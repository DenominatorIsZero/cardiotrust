## ADDED Requirements

### Requirement: A persistent sidebar rail is always visible when the Bevy UI backend is active

The application SHALL render a sidebar rail on the left edge of the screen whenever the Bevy UI backend is selected. The sidebar SHALL contain, from top to bottom: an app logo area, a separator, navigation items for each view, a flexible spacer, a separator, and a Scheduler navigation item pinned to the bottom.

#### Scenario: Sidebar appears when Bevy backend is active

- **WHEN** the UI backend selector is set to `Bevy`
- **THEN** a sidebar rail is visible on the left side of the screen

#### Scenario: Sidebar does not appear when EGui backend is active

- **WHEN** the UI backend selector is set to `EGui`
- **THEN** no Bevy sidebar rail is rendered

### Requirement: The sidebar rail supports expanded and collapsed display modes

The sidebar SHALL have two width modes: expanded (showing icon and label text) and collapsed (showing icon only). A chevron toggle button at the bottom of the sidebar switches between these modes.

#### Scenario: Expanded sidebar shows icon and label

- **WHEN** the sidebar is in expanded mode
- **THEN** each navigation item displays both an icon and a text label

#### Scenario: Collapsed sidebar shows icon only

- **WHEN** the sidebar is in collapsed mode
- **THEN** each navigation item displays only an icon and the text labels are hidden

#### Scenario: Toggle chevron switches between modes

- **WHEN** the user activates the collapse/expand chevron button
- **THEN** the sidebar transitions between expanded and collapsed modes

### Requirement: The sidebar auto-collapses when the viewport is narrow

When the viewport width falls below a defined threshold, the sidebar SHALL automatically switch to collapsed (icon-only) mode. When the viewport width rises above the threshold, the sidebar SHALL restore the user's last explicit expanded/collapsed preference.

#### Scenario: Narrow viewport triggers auto-collapse

- **WHEN** the viewport width drops below 900 px
- **THEN** the sidebar switches to collapsed mode regardless of the prior user preference

#### Scenario: Wide viewport restores previous mode

- **WHEN** the viewport width rises above 900 px after an auto-collapse
- **THEN** the sidebar returns to the mode the user had set before the auto-collapse occurred

### Requirement: Navigation items reflect enabled, disabled, hover, and active visual states

Each navigation item in the sidebar SHALL display distinct visual treatment for four states: default (inactive and enabled), hovered, active (the currently shown view), and disabled (preconditions not met).

#### Scenario: Active navigation item is visually distinguished

- **WHEN** a navigation item corresponds to the currently active view
- **THEN** that item is rendered with an accent-colored left border and accent-colored icon and label

#### Scenario: Hovered navigation item shows hover style

- **WHEN** the pointer moves over an enabled navigation item that is not the active view
- **THEN** the item's background brightens to the hover color

#### Scenario: Disabled navigation item is visually muted and non-interactive

- **WHEN** a navigation item's preconditions are not met
- **THEN** the item is rendered with a muted color and pointer interaction produces no effect

### Requirement: The logo area in the sidebar navigates to the Home view on activation

The top of the sidebar SHALL display the application logo or monogram. Activating the logo area SHALL navigate to the Home view.

#### Scenario: Logo area activation navigates home

- **WHEN** the user activates the logo area at the top of the sidebar
- **THEN** the application navigates to the Home view
