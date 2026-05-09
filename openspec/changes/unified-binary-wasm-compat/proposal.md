## Problem

CardioTrust currently has a native-only application binary (`src/bin/main.rs`) that cannot compile for WASM due to hard dependencies on:
- Filesystem I/O (`std::fs`, `dirs`, `rfd`)
- Native threading (`std::thread`)
- OpenCL (`ocl`)
- File logging (`tracing-appender`)

There is no web deployment path. The goal is a single `main.rs` binary that compiles both natively (full desktop experience) and to WASM (browser demo with pre-baked projects).

## Goals

1. Make `main.rs` compile for `wasm32-unknown-unknown`.
2. Preserve native desktop functionality unchanged.
3. Enable WASM users to create, run, and explore scenarios in-memory.
4. Bundle pre-baked demo projects into the WASM binary.
5. Keep the door open for future web deployment (Project 5).

## Non-Goals

- WebGPU porting (Project 10)
- Interactive tutorials (Project 7)
- Results export to browser downloads (can be added later)
- Lazy loading of demo projects (bundle everything initially; optimize later if needed)

## Design Overview

See `design.md` in this folder for detailed decisions. Key architectural moves:

- **Enum-based `ScenarioStorage`**: `Disk` variant for native, `Memory` variant for WASM.
- **Threading via Rayon + `wasm-bindgen-rayon`**: Replaces `std::thread`/`std::sync::mpsc` with `rayon`/`crossbeam-channel`.
- **Umbrella `"native"` feature**: Gates `ocl`, `dirs`, `rfd`, `tracing-appender`, `nifti`, `ndarray-npy`.
- **In-memory plot generation**: `plotters` generates raw RGBA bytes; disk write is optional.
- **Compile-time embedding**: `include_dir` for scenario data, `bevy_embedded_assets` for visual assets.
- **WASM Home view**: Demo project cards instead of filesystem browsing.

## Capabilities

- **scenario-storage** (modified): support both durable project-backed storage and session-scoped bundled web projects without changing scenario semantics.
- **ui-home-view** (modified): replace placeholder web demo cards with functional bundled-project selection while preserving native project-opening behavior.
- **ui-project-state** (modified): distinguish durable recent-project history on native from session-scoped bundled project activation on web.
- **ui-scenario-view** (modified): surface MRI configuration as unavailable for new web scenarios while still allowing bundled MRI examples to be explored.
- **results-gallery** (modified): keep result-image generation available on web without requiring persisted output files, while retaining native-only export actions that depend on stable storage.

## Risks

- `plotters` may not compile on WASM without feature trimming.
- `wasm-bindgen-rayon` requires COOP/COEP headers — deployment complexity.
- Bundled demo projects may make the `.wasm` binary very large.
- Replacing `bincode` with `postcard` touches serialization across the codebase.
