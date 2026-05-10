## Phase 1: Foundation — Feature Gating & Dependency Cleanup

- [x] Align change docs with capability deltas (`scenario-storage`, `ui-home-view`, `ui-project-state`, `ui-scenario-view`, `results-gallery`).
- [x] Add `"native"` feature to `Cargo.toml`, make it default.
- [x] Gate `ocl` behind `"native"` feature.
- [x] Gate `dirs` behind `"native"` feature.
- [x] Gate `rfd` behind `"native"` feature.
- [x] Gate `tracing-appender` behind `"native"` feature.
- [x] Gate `nifti` behind `"native"` feature.
- [x] Gate `ndarray-npy` behind `"native"` feature.
- [x] Add `crossbeam-channel` as a direct dependency.
- [x] Add `postcard` as a dependency; remove `bincode`.
- [x] Add `rayon` as a dependency.
- [x] Add `wasm-bindgen-rayon` as a WASM-only dependency.
- [x] Add `include_dir` as a dependency.
- [x] Verify `cargo check` passes with `--features native`.
- [x] Verify `cargo check --no-default-features` compiles.

## Phase 2: Storage Abstraction

- [x] Refactor `ScenarioStorage` from struct to enum: `Disk { project_root: PathBuf }` / `Memory { data: HashMap<String, Vec<u8>> }`.
- [x] Implement `Disk` variant methods (`save_metadata`, `load_metadata`, `save_payload`, `load_payload`, `delete_scenario`).
- [x] Implement `Memory` variant methods using in-memory `HashMap`.
- [x] Ensure `ScenarioStorage` remains `Clone`.
- [x] Update all call sites to work with the new enum (no behavioral change on native).
- [x] Add `save_npy` to `Disk` variant only; `Memory` returns an error.

## Phase 3: Threading & Scheduler

- [x] Replace `std::sync::mpsc` with `crossbeam_channel` in `ScenarioBundle`.
- [x] Replace `std::thread::spawn` with `rayon::spawn` in scheduler.
- [x] Replace `JoinHandle` in `ScenarioBundle` with a `crossbeam::channel::Receiver<()>` done-signal.
- [x] Update `check_scenarios` to poll done-channel instead of `is_finished()`.
- [x] Update `run::run` signature to accept crossbeam senders and a done-sender.
- [x] Verify scheduler behavior on native (133 tests pass).

## Phase 4: Serialization Migration

- [x] Replace all `bincode` serialize/deserialize calls with `postcard`.
- [x] Update `ScenarioPayload` serialization in `ScenarioStorage`.
- [x] Verify round-trip serialization produces identical data (test suite passes).
- [x] Remove `bincode` from `Cargo.toml`.

## Phase 5: Target-Conditional UI & Home View

- [x] Home view: native shows "Open Project" + "Recent Projects"; WASM shows demo project cards (structure in place).
- [x] Create `DemoProjectEntry` component and spawn interactive demo cards showing scenario names, status, and detail counts from the embedded `ScenarioList`.
- [x] Implement click-to-load for demo cards — sets `SelectedSenario.index` and transitions to `UiState::Explorer`, bypassing filesystem `PendingProjectLoad`.
- [x] MRI path text input replaced with "Unavailable" label on WASM (tooltip explains pre-computed MRI demo exists).
- [x] Conditionally compile `rfd` file dialog code (`#[cfg(feature = "native")]`).
- [x] Conditionally compile `dirs` recent-projects code (`#[cfg(feature = "native")]`).

## Phase 6: Results & Plotting

- [x] Refactor plot generation to use in-memory `BitMapBackend` (raw RGBA bytes).
- [x] On native, additionally write PNG to disk.
- [x] On WASM, upload raw bytes to `Assets<Image>` directly.
- [x] Update async image loading in Results view to handle both file paths and in-memory bytes.
- [x] Hide or disable native-only export actions on WASM while keeping gallery generation available.

## Phase 7: Algorithm & GPU Gating

- [x] `#[cfg(feature = "native")]` the `ModelBasedGPU` variant in `AlgorithmType`.
- [x] Gate all `ocl` imports and `to_gpu`/`update_from_gpu` methods behind `#[cfg(feature = "native")]`.
- [x] Gate the entire `core::algorithm::gpu` module behind `#[cfg(feature = "native")]`.
- [x] `AlgorithmType` deserialization: `ModelBasedGPU` variant absent on WASM (scenarios must be pre-baked without GPU).
- [x] Update UI algorithm selector to only show GPU option when `"native"` is enabled.

## Phase 8: Embedded Assets & Demo Projects

- [x] Create `wasm-projects/` directory structure.
- [x] Generate 3–4 demo projects locally (handcrafted, MRI pre-processed, tutorial, finished run).
- [x] Add `include_dir` macro to embed `wasm-projects/` at compile time.
- [x] Implement WASM startup system that deserializes embedded projects into `MemoryStorage`.
- [x] Bundle 3D model assets (`bed.glb`, `room.glb`, `torso.glb`, `RoundArrow.obj`/`.mtl`, `sensor_array.glb`) at compile time with `include_bytes!`; inject as default `AssetSource` via Bevy 0.18's `memory::Dir`/`MemoryAssetReader` before `AssetPlugin` builds.

## Phase 9: Logging & `main.rs` Refactoring

- [x] Gate file logging (`tracing-appender`) behind `"native"` feature.
- [x] On WASM, route `tracing` to browser console via `tracing-wasm`.
- [x] Gate `git rev-parse` hash lookup behind `#[cfg(feature = "native")]`; WASM uses `option_env!("GIT_HASH")`.
- [x] Ensure `main.rs` compiles for both targets.

## Phase 10: Build Tooling & Validation

- [x] Add `wasm-run` and `wasm-build` commands to `justfile`.
- [x] Add `.cargo/config.toml` target override for `wasm32-unknown-unknown` with atomics flags.
- [x] Verify `cargo check --no-default-features --target wasm32-unknown-unknown` passes (needs wasm32 toolchain installed).
- [x] Verify native `cargo run --bin main` compilation (`cargo check --features native` passes).
- [x] Run full test suite (`cargo test --features native --lib`): 133 passed, 0 failed.

## Phase 11: Integration & Polish

- [ ] Test WASM build in browser with `wasm-server-runner`.
- [ ] Verify demo projects load and display correctly.
- [ ] Verify scenario creation and execution in WASM.
- [ ] Verify Results gallery image generation in WASM.
- [ ] Verify bundled-project switching replaces the active project cleanly on WASM.
- [ ] Verify MRI configuration is visible but disabled for newly created WASM scenarios.
- [ ] Measure `.wasm` binary size; assess if bundle splitting is needed.
- [x] Update `AGENTS.md` or project docs with new build instructions.
