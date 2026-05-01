## 1. Storage ownership refactor

- [ ] 1.1 Introduce a cloneable `ScenarioStorage` that owns scenario metadata IO, payload IO, deletion, and scenario output path resolution for one project root.
- [ ] 1.2 Refactor `Scenario` into a metadata-only type by removing persistence/path methods and making `Scenario::build()` pure in-memory construction.
- [ ] 1.3 Move stored-scenario operations onto `ScenarioBundle`, including create, metadata load/save, copy, delete, and explicit `load_payload()` that requires both data and results.

## 2. Project-root source of truth

- [ ] 2.1 Extend `ScenarioList` with `project_root: Option<PathBuf>` and make `ScenarioList::load_from(project_root)` metadata-only and tolerant of missing payload files.
- [ ] 2.2 Remove `ProjectState.current_path` and update project loading so opening a project constructs and inserts a fresh `ScenarioList` in one shot while resetting selection.
- [ ] 2.3 Update routing, sidebar, and any remaining UI guards to use `ScenarioList.project_root` as the only loaded-project source of truth.

## 3. Scenario CRUD and payload entry points

- [ ] 3.1 Update Home, Explorer, Scenario header, and any legacy UI CRUD flows to create/copy/save/delete through `ScenarioBundle` instead of calling scenario-owned persistence methods.
- [ ] 3.2 Route Results/Volumetric entry points through explicit bundle payload loading and surface payload failures as view-entry errors without changing a scenario's lifecycle status.
- [ ] 3.3 Ensure copied scenarios persist into the active project with a new identifier and Planning status, and ensure delete removes the stored scenario subtree from that project.

## 4. Project switching UX

- [ ] 4.1 Add a shared project-switching guard that blocks switching only when any loaded scenario is `Simulating` or `Running(_)`.
- [ ] 4.2 Disable the Home open-project button and recent-project entries while switching is blocked, and render the inline explanatory notice.
- [ ] 4.3 Keep project opening and re-opening behavior replacing the entire loaded scenario list only when switching is allowed.

## 5. Results and export path migration

- [ ] 5.1 Refactor results image generation helpers to accept plain scenario metadata/payload plus explicit output paths resolved by storage before worker spawn.
- [ ] 5.2 Refactor animation generation, existing-frame detection, save/open-folder actions, and APNG/MP4 export flows to use storage-resolved scenario directories instead of implicit global paths.
- [ ] 5.3 Refactor NPY export and any remaining results/export path users to write into the selected scenario's persisted output area through storage-owned path resolution.

## 6. Verification

- [ ] 6.1 Update unit and integration tests for metadata-only project loading, explicit all-or-nothing payload loading, and lifecycle-neutral payload failure behavior.
- [ ] 6.2 Add coverage for project switching guards and Home disabled-state behavior for `Simulating`/`Running(_)` versus `Scheduled` scenarios.
- [ ] 6.3 Add coverage for project-aware image, animation, and export path isolation so identical scenario IDs in different projects do not share outputs.
