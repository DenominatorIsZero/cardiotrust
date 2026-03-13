## Context

CardioTrust's UI is currently implemented entirely in EGUI. Project 6 (Complete UI Overhaul) plans to replace it with Bevy native UI, but that is a multi-week effort. This change establishes two foundational prerequisites for that migration:

1. **A canonical color palette** — the Gruvebox Material theme seen in the reference images, expressed as typed color constants. Without a single source of truth, the new Bevy UI components will use ad-hoc colors that won't match each other or the website design language.
2. **A `UiType` state** — a runtime switch that lets both UI backends coexist during the migration. Each system is registered with a `run_if(in_state(UiType::EGui))` condition (matching the existing `UiState` pattern), so the EGUI systems continue running unchanged while new Bevy UI systems are added one at a time.

The current `src/ui.rs` plugin registers EGUI systems unconditionally. The `UiType` state adds an explicit selection layer without changing the default behavior.

## Goals / Non-Goals

**Goals:**
- Define all Gruvebox Material palette entries as named `Color` constants in `src/ui/colors.rs`
- Introduce `UiType` as a Bevy `States` enum with variants `EGui` and `Bevy`, defaulting to `EGui`
- Gate all existing EGUI draw systems (`draw_ui_topbar`, `draw_ui_explorer`, `draw_ui_scenario`, `draw_ui_results`, `draw_ui_volumetric`) behind `.run_if(in_state(UiType::EGui))`, consistent with the existing `UiState` pattern
- Systems that already have a `UiState` run condition use `.run_if(in_state(UiState::Foo).and(in_state(UiType::EGui)))` to combine both state guards
- Keep `UiType::EGui` as the default so the application is functionally identical to today unless the state is explicitly changed

**Non-Goals:**
- Implementing any Bevy UI components (that is Project 6)
- Changing EGUI system logic or visual output
- Hot-reloading or persisting `UiType` across runs
- Any GPU, simulation, or visualization code changes

## Decisions

### Colors as `const` values in a dedicated module
The palette is static — it never changes at runtime. Using `const` Bevy `Color` values in `src/ui/colors.rs` makes them zero-cost, IDE-discoverable, and straightforwardly referenced by any UI code. No struct, no registry, no indirection.

**Alternative considered:** A `ColorPalette` resource injected via Bevy. Rejected — adds runtime overhead and indirection for data that never changes. Constants are simpler and equally shareable.

### `UiType` as a `States` enum, consistent with `UiState`
`UiType` controls which UI backend's systems run. Using `States` (via `app.init_state::<UiType>()`) matches the existing `UiState` pattern exactly, making the codebase consistent and giving Bevy the information it needs to optimise state-dependent scheduling. The F2 toggle uses `NextState<UiType>` to request the transition, same as any other state change.

**Alternative considered:** `UiType` as a plain `Resource` with a custom `run_if` condition. Rejected — inconsistent with `UiState`; `States` is already the project pattern for mode selection and is no more complex to use.

### Run conditions: `in_state(UiType::EGui)` and combined with `in_state(UiState::Foo)`
EGUI systems that have no `UiState` guard (e.g. `draw_ui_topbar`) get `.run_if(in_state(UiType::EGui))`. Systems that are already gated on a `UiState` variant combine both guards: `.run_if(in_state(UiState::Explorer).and(in_state(UiType::EGui)))`. The `.and()` combinator is the idiomatic Bevy 0.15 way to require multiple states simultaneously.

**Alternative considered:** A separate `run_if(ui_type_is_egui)` helper function. Rejected — `in_state` is already imported and understood; an extra helper adds indirection without benefit.

## Risks / Trade-offs

- [Risk: Color constant naming diverges from Gruvebox source] If the palette names in `colors.rs` don't match the reference image labels exactly, future contributors will have to look them up → Mitigation: name constants using the exact tokens from the reference (e.g., `BG_DIM`, `FG0`, `RED`, `AQUA`).
- [Risk: State transition overhead] `States` adds enter/exit scheduling machinery compared to a plain resource → negligible; the state changes at most once per F2 keypress, not every frame.
- [Risk: UiType accidentally set to Bevy before any Bevy systems exist] The application would render nothing → Mitigation: default is `EGui`; only the F2 key triggers a transition in this change.

## Open Questions

None. Both deliverables are straightforward and fully constrained by the proposal and existing codebase patterns.
