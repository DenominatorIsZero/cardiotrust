## Why

Scenario persistence is currently split across `Scenario`, `ScenarioList`, and multiple UI/result modules, with hard-coded `results/<scenario-id>/...` paths and project switching driven by a separate active-path resource. The storage refactor is needed now so project loading, scenario CRUD, payload loading, and result/export/image/animation outputs all follow one project-aware contract before more UI and background-task behavior builds on the old layout.

## What Changes

- Introduce a project-aware scenario storage contract where every scenario lives under `<project>/<scenario-id>/...` and all scenario-owned files, images, animations, and exports resolve through storage.
- Make `ScenarioBundle` the single stored-scenario seam for create, load, copy, save, delete, payload loading, and output-path access.
- Remove persistence behavior from `Scenario` entirely; `Scenario::build()` becomes pure in-memory construction only and no longer writes to disk.
- Expand `ScenarioStorage` so it owns file IO and path rules, not just path calculation, and keep it cheap to clone so UI systems and background threads can pass it around safely.
- Make `ScenarioList` hold `project_root: Option<PathBuf>` as the source of truth for the loaded project and replace the whole resource in one shot when opening a project.
- Reduce `ProjectState` to recent-project UX only and remove the separate current-project path record.
- Block project switching while any scenario is `Simulating` or `Running(_)`, but allow switching when scenarios are only `Scheduled`; surface that block directly in the Home view by disabling open controls and showing an inline reason.
- Keep project loading metadata-only at list load time, with payload loading explicit at the `ScenarioBundle::load_payload()` seam.
- Require payload loading to load both data and results together and fail if either is missing or corrupted.
- Preserve a scenario's `Done` lifecycle status even when later payload loading fails; that becomes a separate view-entry error rather than mutating lifecycle state.
- Refactor results generation and export flows so they accept plain scenario payload data plus explicit output paths/directories that storage resolves ahead of time.
- Remove backward-compatibility behavior for the old flat `results/<scenario-id>/...` layout. **BREAKING**

## Capabilities

### New Capabilities
- `scenario-storage`: Project-aware scenario storage, metadata loading, payload loading, scenario CRUD persistence, and storage-owned output path resolution.

### Modified Capabilities
- `scenario-lifecycle`: Scenario persistence is removed from the lifecycle object, payload loading becomes explicit and strict, and lifecycle state must stay stable when payload viewing fails.
- `ui-project-state`: The loaded project source of truth moves from a separate active-path record to the scenario list resource, and opening a project replaces the whole loaded scenario set.
- `ui-home-view`: Project-open controls must be disabled with an inline notice when project switching is blocked by active simulation or running work.
- `results-gallery`: Results generation and export actions must use storage-resolved scenario output paths rather than implicit global directories.
- `results-gif-viewer`: Animation generation, reload, and export path behavior must follow the project-aware storage layout.

## Impact

- `src/lib.rs` resources for `ScenarioBundle`, `ScenarioList`, and `ProjectState`.
- `src/core/scenario.rs` and `src/core/scenario/persistence.rs`, especially `build`, `load`, `save`, `delete`, and payload/export helpers.
- Project loading and home/explorer/scenario UI flows that create, copy, save, delete, and switch projects.
- Results generation, animation generation, save/open-folder actions, and export paths across `src/ui/bevy_shell/results/`.
- Background-thread call sites that need cloneable storage handles and pre-resolved output directories.
