## Context

Project 3 (Unified Binary Architecture & WASM Compatibility) requires replacing the native-only filesystem-based `ScenarioStorage` with a WASM-compatible equivalent. WASM has no `std::fs`, so all scenario persistence must work in-memory. At the same time, native builds must keep full disk-backed project workflows. We also need to bundle pre-baked demo projects into the WASM binary.

## Decision

Use an **enum-based `ScenarioStorage`** with `Disk` and `Memory` variants (Option B), not a trait object (A) or target-gated separate structs (C).

- **Enum over trait**: Avoids the `Clone`/`Arc` complexity of `Box<dyn StorageBackend>`. `ScenarioStorage` is `Clone` (used in `ScenarioBundle::copy_as_planning`) and both variants are trivially cloneable.
- **Enum over target-gated structs**: Prevents API drift between native and WASM. A single implementation file with `match self` delegates keeps the surface area in one place.

For bundling demo projects:
- **Scenario data** (`wasm-projects/` TOML + bincode payloads) → embedded at compile time via `include_dir` → deserialized into `MemoryStorage` at WASM startup.
- **Visual assets** (PNG, GLTF, fonts) → handled by `bevy_embedded_assets` so Bevy's `AssetServer` works transparently on WASM.

## Consequences

- `ScenarioStorage` gains a `Memory` variant backed by `HashMap<String, Vec<u8>>` (or similar).
- Native builds default to `Disk` variant; WASM builds default to `Memory`.
- Pre-baked demo projects are compiled into the `.wasm` binary, increasing binary size. If load times become unacceptable, we can later switch to lazy fetching without changing the `MemoryStorage` API.
- No trait objects or dynamic dispatch in the hot path.

## Threading & Channel Infrastructure

### Decision

Replace `std::sync::mpsc` with `crossbeam-channel` throughout the scheduler and scenario runner. This provides a near drop-in replacement that works with both native threads and Rayon’s Web Worker pool on WASM.

Use `wasm-bindgen-rayon` for multi-threading on WASM, which requires:
- Rust target features: `+atomics,+bulk-memory,+mutable-globals`
- Server headers: `Cross-Origin-Opener-Policy: same-origin` and `Cross-Origin-Embedder-Policy: require-corp`

### Consequences

- `crossbeam-channel` becomes a direct dependency (it already may be transitive).
- All `std::sync::mpsc::Sender/Receiver` types in `ScenarioBundle` and `run::run` are replaced.
- Native behavior is unchanged; WASM gains real multi-threading for scenario execution.
- Deployment infrastructure must serve the correct COOP/COEP headers for `SharedArrayBuffer`.

### Scheduler Lifecycle Update

The `ScenarioBundle` currently stores `Option<std::thread::JoinHandle<()>>` to track running scenarios. Rayon does not provide `JoinHandle`s — `rayon::spawn` is fire-and-forget.

Replace `JoinHandle` with a **`crossbeam::channel` done-signal**: the scenario runner receives a `Sender<()>` clone and sends `()` when `run()` completes. The scheduler polls `done_rx.try_recv()` to detect completion. This works identically on native (Rayon threads) and WASM (Web Workers).

## Native Feature Gating

### Decision

Introduce a single umbrella `"native"` feature (enabled by default) that gates all desktop-only dependencies and capabilities:
- `ocl` (GPU/OpenCL)
- `dirs` (config directory for recent projects)
- `rfd` (native file dialogs)
- `tracing-appender` (file-based logging)

WASM builds compile with `--no-default-features`. There is no granular sub-feature decomposition for now — native always means "full desktop experience."

### Consequences

- `cargo run` continues to work unchanged on desktop.
- WASM build command becomes `cargo build --target wasm32-unknown-unknown --no-default-features`.
- Any future native-only dependency must be added to the `"native"` feature set.
- Simpler feature matrix at the cost of flexibility (no CPU-only native builds without also losing file dialogs/logging).

## Project State on WASM

### Decision

Treat bundled web projects as session-scoped active projects. A web user can activate one bundled project at a time, and that project behaves like the active project for browsing, editing, running, and viewing results during the current session. Recent-project history remains a native-only concept.

### Consequences

- Native builds continue to persist recent project history across launches.
- WASM builds do not promise durable project history across page reloads.
- Switching between bundled projects on the web replaces the loaded scenario collection the same way opening a different folder does on native.

## Results Image Generation

### Decision

Use `plotters` with an in-memory `BitMapBackend` (raw RGBA bytes) on all targets. On native (`"native"` feature enabled), additionally write PNG files to disk. On WASM, images are generated in-memory and uploaded directly to Bevy's `Assets<Image>` without filesystem access.

### Consequences

- The Results gallery works identically on native and WASM.
- No pre-baked plot images are needed for bundled demos.
- `plotters` backend selection moves from file-based to memory-based; disk I/O becomes an optional side effect.
- Native-only exports that require stable files remain unavailable on WASM until a browser download flow is introduced.

## MRI Model Handling on WASM

### Decision

MRI model options remain **visible but disabled** in the WASM scenario builder UI, with an explanatory tooltip: "Running MRI-based scenarios is not supported on the web. You can explore the pre-computed MRI example from the demo projects."

The pre-baked MRI demo scenario embeds **pre-processed model data** (voxel grid, geometry, etc.) as serialized structs, bypassing `.nii` parsing entirely on WASM. The `nifti` crate is gated behind the `"native"` feature.

### Consequences

- Users understand why MRI configuration is unavailable without feeling the feature is missing.
- The single MRI demo scenario demonstrates the capability without requiring NIfTI parsing in the browser.
- Native builds retain full MRI loading from `.nii` files.

## WASM Home View

### Decision

Keep the Home view on WASM as a **landing page** with demo project cards (MRI demo, handcrafted demo, tutorial project, etc.) instead of the native "Open Project / Recent Projects" panels. Users click a card to load that embedded project into the Explorer view.

### Consequences

- The existing `#[cfg(target_arch = "wasm32")] spawn_demo_projects_panel` placeholder is replaced with a real, functional project selection panel.
- Home remains the initial view on all targets, but its content is target-conditional.
- Demo project cards trigger in-memory project loading via `MemoryStorage`.

## Serialization & Additional Dependencies

### Decision

Replace `bincode` v2 with **`postcard`** for scenario payload serialization. `bincode` v2 is deprecated; `postcard` produces smaller binaries and is better suited for WASM.

Gate `ndarray-npy` behind the `"native"` feature — NPY export is native-only.

`plotters` compatibility with WASM will be addressed reactively if compilation issues arise. No premature feature gating.

### Consequences

- All bincode serialization/deserialization calls are migrated to `postcard`.
- `ndarray-npy` is removed from WASM builds, reducing binary size.
- If `plotters` default features fail on WASM, we will trim features at that point.
