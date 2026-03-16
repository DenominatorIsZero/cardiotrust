## Context

Phase 1 built the Bevy-native navigation shell (sidebar, breadcrumb bar, view routing) but left the content area empty and kept `UiType::EGui` as the startup default. The app still loads `ScenarioList` from a hardcoded `./results/` path at process startup and drops straight into the Explorer. Phase 2 makes the Bevy path the primary experience: the app now starts on the Home view where the user selects a project folder, and the selected folder drives all subsequent scenario loading.

Two structural problems must be solved together:

1. **Hardcoded path**: `ScenarioList::load()` and `ScenarioList::default()` both hard-wire `./results/`. There is no mechanism for the UI to tell the app "load from this other directory".
2. **Missing Home view**: There is no Bevy UI node tree for the Home state, so `UiState::Home` currently renders an empty content slot.

## Goals / Non-Goals

**Goals:**
- App starts in Bevy mode with the Home view visible.
- User can pick a project folder with a native file dialog; the app loads `ScenarioList` from that path and navigates to Explorer.
- Recent projects (up to 8) are persisted across sessions in a config file and shown on the Home view as one-click shortcuts.
- Opening a second project while one is already loaded replaces the first cleanly (clears `SelectedSenario`, drops old `ScenarioList`, loads new one).
- WASM: demo project cards appear as visible placeholders; they do nothing yet.
- EGui path continues to work unchanged (hardcoded `./results/`).

**Non-Goals:**
- Full WASM file-upload / zip ingestion (future phase).
- Explorer card-grid view (Phase 3).
- Any view other than Home in the Bevy path (Phase 3+).
- Drag-and-drop folder targets.

## Decisions

### Decision 1: Introduce `ProjectState` as a Bevy `Resource`

`ProjectState` holds `current_path: Option<PathBuf>` and `recent: Vec<PathBuf>` (capped at 8). The Home view reads and writes this resource; `ScenarioList` loading is triggered by a system that watches for changes to `ProjectState::current_path`.

**Alternatives considered:**
- Pass the path as an event: events are one-shot; a resource is easier to inspect from breadcrumb/sidebar systems that already run in Update.
- Embed the path inside `ScenarioList`: conflates identity (where the project lives) with content (what scenarios it contains). Keeping them separate lets sidebar check "is a project open?" without scanning the list.

### Decision 2: `ScenarioList::load_from(path: &Path)` + refactor existing `load()`

Add a new `load_from` method that accepts an explicit path. The existing `load()` and `Default` implementations become thin wrappers calling `load_from("./results")`. This keeps the egui path unchanged.

**Why not touch `Default`?** The EGui path relies on `init_resource::<ScenarioList>()` in `main.rs`, which calls `Default`. As long as that default still loads from `./results/`, no egui code needs to change.

### Decision 3: Project loading triggered by a Bevy system watching `ProjectState`

A system `load_project_on_path_change` runs in `Update`, detects `Changed<ProjectState>`, reads `current_path`, calls `ScenarioList::load_from`, and does a full resource replace via `commands.insert_resource(...)`. It also resets `SelectedSenario` and calls `NextState::<UiState>::set(UiState::Explorer)`.

**Why not trigger from the button handler directly?** Bevy systems cannot return `Result`; offloading the side-effectful I/O to a dedicated system keeps the button handler trivial and makes error handling (log warning, stay on Home) centralized.

### Decision 4: Recent projects stored in a TOML config file

Path: `~/.config/cardiotrust/recent_projects.toml` (native). Written every time `ProjectState::current_path` changes. Read at startup into `ProjectState::recent` before the first frame.

**WASM alternative**: `localStorage` key — acceptable for WASM builds, but since this phase focuses on native, WASM persistence is a no-op stub for now.

### Decision 5: Home view built directly inside `BevyShellPlugin`

Rather than adding a new plugin, a `home` submodule provides `spawn_home_view` (runs `OnEnter(UiState::Home)`) and `despawn_home_view` (runs `OnExit(UiState::Home)`). Follows the same pattern used by the Phase 1 root layout.

### Decision 6: Sidebar hides project-dependent items when no project is open

The existing `update_nav_item_visual_states` system in `sidebar.rs` already handles disabled states. Extend it: when `ProjectState::current_path.is_none()`, mark Explorer, Scenario, Results, Volumetric, and Scheduler as `NavItemDisabled`. This keeps the Home nav item always enabled.

### Decision 7: `UiType::Bevy` and `UiState::Home` become defaults

Change both `Default` implementations. The F2 toggle still switches to EGui (and back), preserving the developer escape hatch. When switching to EGui, `UiState` should remain as-is (EGui has its own Explorer default that is set by its topbar logic when no scenario is selected).

## Risks / Trade-offs

- **`ScenarioList::Default` still loads `./results/`** → On startup the resource is initialized with the hardcoded path before the Home view can show. This is a minor correctness issue: `main.rs` should not call `init_resource::<ScenarioList>()` when in Bevy mode. Mitigation: in `main.rs`, replace `init_resource::<ScenarioList>()` with `insert_resource(ScenarioList::empty())` so no disk I/O happens at boot; the Home view triggers the first real load.
- **`rfd::FileDialog` blocks the main thread** → `rfd` has an async variant, but Bevy's `bevy_tasks` integration for file dialogs is non-trivial. For this phase, use the synchronous `rfd::FileDialog::pick_folder()` call inside the button handler; for a desktop app this brief block (< 1s) is acceptable. Mitigation: document the limitation; async dialog is a future improvement.
- **Config file write failures** → Writing to `~/.config/cardiotrust/` can fail on restricted systems. Mitigation: log a warning and continue; recent-projects persistence is best-effort.

## Open Questions

- Should the Home view auto-select the most recent project and skip straight to Explorer if there is exactly one recent entry? (The design doc says "auto-load if only one project exists" — this can be implemented as a startup system checking `recent.len() == 1`.) Decision deferred to tasks phase; task list should include this as an optional sub-task.
