## Why

The current EGUI-based UI has no defined color theme and will be replaced by a Bevy native UI. To support a smooth incremental migration, we need a canonical Gruvebox Material color palette and a runtime toggle (`UiType`) that lets both UI backends coexist while the new Bevy UI is built out.

## What Changes

- Add a `colors` module under `src/ui/` that defines all Gruvebox Material palette constants as `bevy::prelude::Color` values
- Add a `UiType` resource (variants `EGui` and `Bevy`) that gates which set of UI systems runs each frame
- Existing EGUI systems run only when `UiType::EGui` is active; new Bevy UI systems (not yet implemented) will run when `UiType::Bevy` is active
- `UiType::EGui` is the default so the application behaves identically to today unless the resource is changed
- Pressing **F2** toggles `UiType` between `EGui` and `Bevy` at runtime, providing a quick development feedback loop

## Capabilities

### New Capabilities

- `ui-color-theme`: A named color palette derived from the Gruvebox Material theme, exposed as typed constants for use by both the EGUI and future Bevy UI layers
- `ui-type-resource`: A runtime resource that selects which UI backend (EGui or Bevy) is active, controlling system scheduling

### Modified Capabilities

- `ui-navigation`: The UI plugin's system registration must gate all existing EGUI draw systems on `UiType::EGui`, preserving identical runtime behavior by default

## Impact

- New file `src/ui/colors.rs` (color palette constants — no logic)
- `src/ui.rs` gains `UiType` resource initialization and run conditions on existing EGUI systems
- No changes to algorithm, simulation, or visualization code
- No breaking changes; default behavior is unchanged
