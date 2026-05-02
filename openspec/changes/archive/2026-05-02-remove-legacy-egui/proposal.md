## Why

The Bevy-native shell is now the primary CardioTrust UI, but the repository still carries the older `UiType::EGui` backend and its legacy views. Keeping both paths increases maintenance cost, preserves dead navigation behavior in the product contract, and keeps legacy UI dependencies around after the migration is effectively complete.

## What Changes

- Remove the legacy alternate UI backend so the application has one supported desktop/WASM UI path.
- Remove the old top-bar, explorer, scenario, and volumetric egui views and any startup or input behavior that exists only to support switching back to that legacy UI.
- Update the UI requirements so startup, navigation shell behavior, and developer-facing controls describe the Bevy-native experience only.
- Remove dependencies and supporting code that become unused after the legacy egui path is deleted, while retaining any scoped overlay dependencies still required by the Bevy-native volumetric and project-loading flows.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `ui-navigation`: The application shell contract no longer includes a legacy EGUI top bar or backend-specific startup behavior; navigation is defined only for the Bevy-native shell.
- `ui-type-resource`: The UI backend selector and runtime F2 toggle are removed because the application now has a single supported UI backend.

## Impact

- Affected code: UI plugin setup, legacy egui modules, startup/input wiring, and any cleanup of now-unused resources or systems.
- Affected dependencies: remove crates that were only needed by the deleted legacy egui path; keep crates still needed by scoped overlays or file-dialog behavior.
- Affected behavior: startup and navigation documentation become single-path and no longer promise runtime backend switching.
