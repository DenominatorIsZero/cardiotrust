## Context

Scenario persistence is currently spread across three layers that disagree about ownership. `Scenario` still creates, saves, loads, deletes, and exports itself under a hard-coded global `./results/<id>` tree. `ScenarioList::load_from` scans a project folder but immediately deserializes whole scenarios through `Scenario::load`, which couples project loading to scenario-owned persistence. The Bevy Home/project flow then keeps a second source of truth in `ProjectState.current_path`, while results generation and export code constructs output paths directly inside UI systems.

That split is now the main obstacle to project-aware storage. The current code can load scenarios from an arbitrary folder, but most create/save/delete/export/generate paths still assume a single process-global results directory. The refactor must therefore cut across `src/lib.rs`, `src/core/scenario.rs`, `src/core/scenario/persistence.rs`, `src/ui/bevy_shell/project.rs`, Home/explorer/scenario CRUD actions, and the whole `src/ui/bevy_shell/results/` path surface.

The user requirements narrow the design in three important ways:

1. No backward compatibility is required. The implementation can fully replace the old flat layout and old persistence entry points.
2. Project loading must stay lightweight. Listing scenarios from a project cannot require loading `data` and `results` payload blobs for every entry.
3. Background workers still need storage access. The new storage owner must be cheap to clone and safe to move into threads that generate images, animations, and exports.

## Goals / Non-Goals

**Goals:**
- Make project-aware storage the only persistence model for scenarios and scenario-owned outputs.
- Centralize file IO and path rules inside a cloneable `ScenarioStorage` abstraction instead of scattering them across `Scenario`, `ScenarioList`, and UI systems.
- Make `ScenarioBundle` the single stored-scenario seam for metadata, storage, create/load/copy/save/delete operations, explicit payload loading, and output-path lookup.
- Keep `ScenarioList::load_from(project_root)` metadata-only, tolerant of missing payload files, and able to replace the loaded project in one resource swap.
- Remove `ProjectState.current_path` and make `ScenarioList.project_root` the loaded-project source of truth used by Home, sidebar, routing, and project switching.
- Ensure project switching is blocked only while any scenario is `Simulating` or `Running(_)`, and surface that block clearly in the Home view.
- Refactor results generation/export/image/animation flows so workers receive explicit output paths/directories that storage resolves before the worker starts.

**Non-Goals:**
- Preserving the old `./results/<scenario-id>` layout or adding a migration layer.
- Redesigning the scenario lifecycle state machine beyond the payload-loading semantics called out in this change.
- Changing numerical simulation, estimation, plotting, or animation behavior.
- Introducing a new user-facing project model beyond the existing open-project and recent-project UX.

## Decisions

### 1. Split scenario metadata from persisted payload ownership

`Scenario` becomes a pure in-memory metadata object. It keeps lifecycle state, config, summary, timestamps, and comment, but it no longer knows where it lives on disk and no longer exposes persistence helpers such as `load`, `save`, `delete`, `load_data`, `load_results`, or `save_npy`.

`Scenario::build()` likewise becomes pure construction: it creates a Planning scenario with a generated identifier and default metadata, but it does not write anything.

The persistence behavior moves to the stored-scenario seam: `ScenarioBundle` plus `ScenarioStorage`.

Why this over keeping thin persistence methods on `Scenario`?
Because the existing bug is ownership confusion. Leaving path-aware IO on `Scenario` would keep the same leakage that currently lets UI code call scenario methods assuming a global layout.

### 2. Make `ScenarioStorage` the only owner of file IO and path rules

`ScenarioStorage` becomes a small cloneable value that represents one project root and can resolve scenario-specific locations for metadata, payload, generated images, animation frame directories, and exports. To keep cloning cheap for background jobs, it should hold the root path in shared immutable storage such as `Arc<PathBuf>` and expose path-resolution plus read/write/delete helpers.

This type should own operations like:
- creating a scenario directory
- loading and saving scenario metadata
- loading both payload blobs together
- deleting a scenario subtree
- resolving directories/files for generated images, animation frames, APNG/MP4 exports, and NPY export

Why this over keeping a path-only helper and separate IO functions elsewhere?
Because the repo already shows what happens when only path calculation is centralized: UI and domain code immediately reintroduce direct `Path::new("results")...` call sites. The storage type must own both path policy and the file IO that enforces it.

### 3. Make `ScenarioBundle` the stored-scenario boundary

Each `ScenarioBundle` will contain:
- scenario metadata (`Scenario`)
- the scenario's `ScenarioStorage`
- runtime-only execution fields already present today (`join_handle`, receivers)

`ScenarioBundle` becomes the only place that application code uses for stored-scenario operations: create, load metadata, save metadata, copy, delete, explicit payload load, and output-path lookup.

`load_payload()` stays explicit on the bundle seam and loads both `data` and `results` together. It returns an all-or-nothing payload snapshot for consumers such as Results and export jobs. If either side is missing or corrupt, the call fails and does not partially hydrate only half the payload.

Why this over making `ScenarioList` own the operations directly?
Because CRUD and payload operations are per-scenario concerns, while `ScenarioList` should stay the collection/root source of truth. Pushing stored-scenario behavior down to the bundle also gives threads a compact handle that already knows its storage root.

### 4. Keep project loading metadata-only and make payload failure lifecycle-neutral

`ScenarioList::load_from(project_root)` will scan the selected project, create bundle entries from readable metadata, set `project_root: Option<PathBuf>`, and sort entries as today. It must not attempt to read payload blobs during project load.

That allows project switching to stay fast and tolerant of partial payload damage. A scenario whose metadata says `Done` remains `Done` even if later `load_payload()` fails because `data` or `results` is missing or corrupt. The failure is treated as a separate entry/view error for Results or Volumetric access, not a lifecycle demotion.

Why this over eagerly loading payloads at project-open time?
Because eager payload loading would make project-open brittle, slower, and inconsistent with the existing Results workflow, which only needs payload when entering result-driven views or starting export/generation work.

### 5. Make `ScenarioList` the only loaded-project source of truth

`ScenarioList` gains `project_root: Option<PathBuf>`. When the user opens a project, the app constructs a fresh `ScenarioList::load_from(project_root)` and inserts it as a whole resource, resetting selection in the same transaction. The existing list is discarded rather than mutated entry-by-entry.

`ProjectState.current_path` is removed. `ProjectState` remains only for recent-project history persistence. UI systems that only need to know whether a project is loaded should read `ScenarioList.project_root` instead.

This affects:
- Home project-open actions
- the Bevy project loading system
- sidebar/routing guards that currently check `current_path`
- any create/copy/save/delete path that currently takes `ProjectState`

Why this over keeping both `current_path` and `ScenarioList.project_root` in sync?
Because two mutable sources of truth are exactly what this refactor is trying to eliminate. The loaded collection already has to know which project it came from; duplicating that state elsewhere only reintroduces drift.

### 6. Gate project switching in Home by runtime status, not by queued work

Project switching is blocked only if any loaded scenario is currently `Simulating` or `Running(_)`. `Scheduled` scenarios do not block switching. The Home view must compute this from the current `ScenarioList` entries, disable both the native open-project button and recent-project entries while blocked, and show a clear inline notice explaining that project switching is unavailable until active computation finishes.

Why this over blocking on any non-Planning work?
Because `Scheduled` is only queued intent, not active work holding resources. Blocking on queued scenarios would be unnecessarily restrictive and would conflict with the user's requested behavior.

### 7. Resolve result and export paths before spawning background work

The current Results systems build scenario output paths inside worker setup closures using hard-coded global directories. After this change, the main-thread caller must first obtain the required output path(s) or directory from `ScenarioBundle`/`ScenarioStorage`, then pass those explicit paths into generation/export helpers.

This applies to:
- static image generation
- animation frame generation and detection
- APNG export
- MP4 export
- NPY export
- save/open-in-file-manager actions

Generation helpers should take plain metadata plus payload structs and explicit output locations, not a storage-owning `Scenario` that secretly decides where files go.

Why this over letting worker functions ask storage for their own paths?
Pre-resolving paths keeps worker APIs explicit, avoids hidden filesystem policy in plotting helpers, and makes it easy to verify that each job writes into the selected scenario's storage area even when identical scenario IDs exist in different projects.

## Risks / Trade-offs

- [Large refactor surface across UI and core modules] -> Keep the change mechanical: first move persistence ownership, then redirect CRUD call sites, then convert results/export path users.
- [Background jobs may capture stale selection or wrong project state] -> Pass cloned bundle/storage handles and resolved output paths into each job at spawn time rather than looking up global resources inside the thread.
- [Metadata-only loading may hide payload damage until later] -> Surface payload load failures as explicit Results/Volumetric entry errors while preserving lifecycle status so the user sees both facts separately.
- [Removing `ProjectState.current_path` may break navigation guards] -> Update sidebar, routing, and Home logic in the same change to read `ScenarioList.project_root` exclusively.
- [Deleting backward compatibility increases cut-over risk] -> Update all create/load/save/delete/export/generate call sites in the same implementation slice and remove the old scenario-owned persistence APIs immediately so stale usage fails at compile time.

## Migration Plan

1. Introduce `ScenarioStorage` and refactor `Scenario` to metadata-only construction/state with no filesystem methods.
2. Move stored-scenario operations onto `ScenarioBundle`, including metadata save/load, copy, delete, and explicit all-or-nothing payload loading.
3. Extend `ScenarioList` with `project_root`, make `load_from` metadata-only, and replace `ProjectState.current_path` usages with list-root checks.
4. Update Home/project switching to construct and insert a whole new `ScenarioList`, apply switching guards for `Simulating`/`Running(_)`, and show the inline blocked notice.
5. Refactor results/export/image/animation call sites to request resolved paths from storage before spawning workers, and make generation/export helpers accept payload data plus explicit output paths.
6. Remove the old flat-layout persistence code and update tests to assert the new project-aware storage semantics.

Rollback is intentionally not a goal for this change. Because no backward compatibility is required, the safe path is to keep this work on the change branch until the whole storage cut is coherent and the old API surface is gone.

## Open Questions

- None. The requested behavior is specific enough to make the change implementation-ready without further product decisions.
