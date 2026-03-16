## Why

Phase 1 established the Bevy-native navigation shell, but the app still hardcodes `./results/` as the project root, starts in Explorer via the EGui backend, and has no Home view for project selection. Phase 2 makes the Bevy path the primary experience by introducing the Home view, a `ProjectState` resource that replaces the hardcoded path, and the logic to open/reload a project folder — including replacing an already-open one without a restart.

## What Changes

- **Start in Bevy mode**: `UiType` default changes from `EGui` to `Bevy`; `UiState` default changes from `Explorer` to `Home`. The EGui path remains available via F2.
- **`ProjectState` resource**: Replaces the hardcoded `./results/` path. Holds the current project folder path and recent-project history (up to 8 entries). Persists recent-project list to a config file.
- **Home view (`src/ui/bevy_shell/home.rs`)**: Bevy UI node tree showing the title area, Open Project panel (folder browse button via `rfd::FileDialog`), and Recent Projects list. WASM demo project cards are placeholder tiles with TODO labels.
- **Project loading system**: On folder selection (or recent-project click), reloads `ScenarioList` from the new path, clears `SelectedSenario`, and transitions to `UiState::Explorer`. If a project is already open it is fully replaced.
- **`ScenarioList::load_from(path)`**: New method that accepts an explicit path, extracted from the existing `load()` implementation which becomes a thin wrapper calling `load_from("./results")`.
- **Sidebar conditional on project**: Sidebar nav items (Explorer, Scenario, Results, Volumetric, Scheduler) are shown only when `ProjectState` has an active project; on the Home view they are hidden/disabled.
- **WASM stub**: On WASM builds the Open Project panel shows three placeholder "Demo Project" cards that do nothing yet (labelled "Coming soon").

## Capabilities

### New Capabilities

- `ui-home-view`: The Home view Bevy UI node — title/hero area, Open Project panel (native: folder dialog; WASM: placeholder cards), Recent Projects list. Handles project open and project reload/replace.
- `ui-project-state`: The `ProjectState` resource — current project path, recent-project list, serialization to a config file. Provides the contract between the Home view and the rest of the app for which folder is currently loaded.

### Modified Capabilities

- `ui-navigation`: The Bevy backend now starts on the Home view instead of Explorer. Sidebar items for project-dependent views (Explorer through Volumetric) are disabled and non-interactive when no project is loaded. The F2 shortcut still toggles backends.

## Impact

- `src/ui.rs` — Change `UiState::default()` to `Home` and `UiType::default()` to `Bevy`. Register `ProjectState` as a resource.
- `src/lib.rs` — Add `ScenarioList::load_from(path: &Path)`. Refactor `ScenarioList::load()` and `Default` to call it.
- `src/ui/bevy_shell/` — New `home.rs` module. Update `mod.rs` to add `HomePlugin` (or extend `BevyShellPlugin`). Update `sidebar.rs` to gate project-dependent items on `ProjectState`.
- `src/ui/bevy_shell/routing.rs` — Keyboard shortcut `1` navigates to `Home`; guard project-dependent shortcuts (2–6) when no project is loaded.
- `src/bin/main.rs` — Initialize `ProjectState` resource. Do not pre-load `ScenarioList` at startup; let the Home view trigger loading.
- `Cargo.toml` — `rfd` is already listed as a dependency; no new additions required.
- Existing egui systems — untouched; they continue to work under `UiType::EGui` using the hardcoded `./results/` path that is unchanged by `ScenarioList::load()`.
