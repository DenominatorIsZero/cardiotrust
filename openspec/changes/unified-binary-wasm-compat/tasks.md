## Phase 1: Foundation — Feature Gating & Dependency Cleanup

- [ ] Align change docs with capability deltas (`scenario-storage`, `ui-home-view`, `ui-project-state`, `ui-scenario-view`, `results-gallery`).
- [ ] Add `"native"` feature to `Cargo.toml`, make it default.
- [ ] Gate `ocl` behind `"native"` feature.
- [ ] Gate `dirs` behind `"native"` feature.
- [ ] Gate `rfd` behind `"native"` feature.
- [ ] Gate `tracing-appender` behind `"native"` feature.
- [ ] Gate `nifti` behind `"native"` feature.
- [ ] Gate `ndarray-npy` behind `"native"` feature.
- [ ] Add `crossbeam-channel` as a direct dependency.
- [ ] Add `postcard` as a dependency; remove `bincode`.
- [ ] Add `rayon` as a dependency.
- [ ] Add `wasm-bindgen-rayon` as a WASM-only dependency.
- [ ] Add `include_dir` as a dependency.
- [ ] Verify `cargo check` passes with `--features native`.
- [ ] Verify `cargo check --no-default-features` compiles (will have errors initially; track them).

## Phase 2: Storage Abstraction

- [ ] Refactor `ScenarioStorage` from struct to enum: `Disk { project_root: PathBuf }` / `Memory { data: HashMap<String, Vec<u8>> }`.
- [ ] Implement `Disk` variant methods (`save_metadata`, `load_metadata`, `save_payload`, `load_payload`, `delete_scenario`).
- [ ] Implement `Memory` variant methods using in-memory `HashMap`.
- [ ] Ensure `ScenarioStorage` remains `Clone`.
- [ ] Update all call sites to work with the new enum (no behavioral change on native).
- [ ] Add `save_npy` to `Disk` variant only; `Memory` returns an error or no-op.

## Phase 3: Threading & Scheduler

- [ ] Replace `std::sync::mpsc` with `crossbeam_channel` in `ScenarioBundle`.
- [ ] Replace `std::thread::spawn` with `rayon::spawn` in scheduler.
- [ ] Replace `JoinHandle` in `ScenarioBundle` with a `crossbeam::channel::Receiver<()>` done-signal.
- [ ] Update `check_scenarios` to poll done-channel instead of `is_finished()`.
- [ ] Update `run::run` signature to accept crossbeam senders and a done-sender.
- [ ] Verify scheduler behavior on native is unchanged.

## Phase 4: Serialization Migration

- [ ] Replace all `bincode` serialize/deserialize calls with `postcard`.
- [ ] Update `ScenarioPayload` serialization in `ScenarioStorage`.
- [ ] Verify round-trip serialization produces identical data.
- [ ] Remove `bincode` from `Cargo.toml`.

## Phase 5: Target-Conditional UI & Home View

- [ ] Refactor Home view: native shows "Open Project" + "Recent Projects"; WASM shows demo project cards.
- [ ] Create demo project card component for WASM Home.
- [ ] Implement click-to-load for demo cards (hydrates `MemoryStorage` from embedded data).
- [ ] Make MRI model options visible but disabled in WASM scenario builder with tooltip.
- [ ] Conditionally compile `rfd` file dialog code.
- [ ] Conditionally compile `dirs` recent-projects code.

## Phase 6: Results & Plotting

- [ ] Refactor plot generation to use in-memory `BitMapBackend` (raw RGBA bytes).
- [ ] On native, additionally write PNG to disk.
- [ ] On WASM, upload raw bytes to `Assets<Image>` directly.
- [ ] Update async image loading in Results view to handle both file paths and in-memory bytes.
- [ ] Hide or disable native-only export actions on WASM while keeping gallery generation available.

## Phase 7: Algorithm & GPU Gating

- [ ] `#[cfg(feature = "native")]` the `ModelBasedGPU` variant in `AlgorithmType`.
- [ ] Gate all `ocl` imports and `to_gpu`/`update_from_gpu` methods behind `#[cfg(feature = "native")]`.
- [ ] Gate the entire `core::algorithm::gpu` module behind `#[cfg(feature = "native")]`.
- [ ] Ensure `AlgorithmType` deserialization handles missing `ModelBasedGPU` gracefully (or pre-bake WASM scenarios without GPU).
- [ ] Update UI algorithm selector to only show GPU option when `"native"` is enabled.

## Phase 8: Embedded Assets & Demo Projects

- [ ] Create `wasm-projects/` directory structure.
- [ ] Generate 3–4 demo projects locally (handcrafted, MRI pre-processed, tutorial, finished run).
- [ ] Add `include_dir` macro to embed `wasm-projects/` at compile time.
- [ ] Implement WASM startup system that deserializes embedded projects into `MemoryStorage`.
- [ ] Add `bevy_embedded_assets` for visual asset bundling.

## Phase 9: Logging & `main.rs` Refactoring

- [ ] Gate file logging (`tracing-appender`) behind `"native"` feature.
- [ ] On WASM, set up `tracing` to log to browser console.
- [ ] Gate `git rev-parse` hash lookup behind `#[cfg(feature = "native")]` (or use compile-time env).
- [ ] Ensure `main.rs` compiles for both targets.

## Phase 10: Build Tooling & Validation

- [ ] Add `wasm-run` and `wasm-build` commands to `justfile`.
- [ ] Add `.cargo/config.toml` target override for `wasm32-unknown-unknown` with atomics flags.
- [ ] Verify `cargo check --no-default-features --target wasm32-unknown-unknown` passes.
- [ ] Verify native `cargo run --bin main` still works.
- [ ] Run full test suite (`just test`) to catch regressions.

## Phase 11: Integration & Polish

- [ ] Test WASM build in browser with `wasm-server-runner`.
- [ ] Verify demo projects load and display correctly.
- [ ] Verify scenario creation and execution in WASM.
- [ ] Verify Results gallery image generation in WASM.
- [ ] Verify bundled-project switching replaces the active project cleanly on WASM.
- [ ] Verify MRI configuration is visible but disabled for newly created WASM scenarios.
- [ ] Measure `.wasm` binary size; assess if bundle splitting is needed.
- [ ] Update `AGENTS.md` or project docs with new build instructions.
