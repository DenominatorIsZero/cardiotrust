## ADDED Requirements

### Requirement: A canonical color palette is available to all UI layers

The application SHALL expose a complete, named set of color constants derived from the Gruvebox Material theme. Every color in the palette SHALL be addressable by its semantic name (e.g., background levels, foreground levels, accent colors, diff colors). No UI component SHALL define an inline color value that duplicates a palette entry.

#### Scenario: Background levels are available

- **WHEN** a UI component needs a dark background color
- **THEN** it can reference a named constant for each background level (dim, base, raised, elevated, high, highest)

#### Scenario: Foreground levels are available

- **WHEN** a UI component needs a text or icon color
- **THEN** it can reference a named constant for each foreground level

#### Scenario: Accent colors are available

- **WHEN** a UI component needs a semantic accent color
- **THEN** it can reference named constants for red, green, blue, yellow, purple, orange, and aqua

#### Scenario: Status-line background variants are available

- **WHEN** a UI component needs a status bar or secondary background
- **THEN** it can reference named constants for the three status-line background levels

#### Scenario: Diff and visual highlight colors are available

- **WHEN** a UI component needs to indicate a diff, selection, or highlight region
- **THEN** it can reference named constants for diff backgrounds (red, green, blue) and visual selection backgrounds (red, green, blue, yellow, purple)

### Requirement: Palette constants are the single source of truth

All palette entries SHALL be defined exactly once. Duplicate definitions of the same color SHALL NOT exist anywhere in the codebase. Any future UI component SHALL reference the palette rather than hardcoding hex values.

#### Scenario: No duplicate color definitions

- **WHEN** the full UI codebase is inspected
- **THEN** each Gruvebox Material color appears in exactly one definition site
