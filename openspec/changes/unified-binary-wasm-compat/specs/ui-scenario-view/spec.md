## ADDED Requirements

### Requirement: MRI anatomy selection is visible but unavailable for new web scenarios

When the application is running as a web build, the Scenario editor SHALL continue to show MRI-based anatomy as part of the model selection surface so users can understand that the capability exists. For newly created or edited web scenarios, MRI-based anatomy selection SHALL be visibly disabled and accompanied by guidance that bundled MRI examples can be explored but new MRI-based runs cannot be configured in the browser.

#### Scenario: Web scenario editor shows disabled MRI option
- **WHEN** the user opens the Scenario editor on a web build
- **THEN** the MRI-based anatomy option is visible but not selectable for a newly configured scenario

#### Scenario: Web MRI tooltip explains the limitation
- **WHEN** the user focuses or hovers the disabled MRI-based anatomy option on a web build
- **THEN** the editor SHALL explain that new MRI-based scenarios are unavailable in the browser and that bundled MRI examples can still be explored

#### Scenario: Native scenario editor keeps MRI option available
- **WHEN** the user opens the Scenario editor on a native build
- **THEN** MRI-based anatomy remains available subject to the normal configuration rules
