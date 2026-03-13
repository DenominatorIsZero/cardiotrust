## 1. Add Gruvebox Material Color Palette

- [x] 1.1 Create `src/ui/colors.rs` and declare it as a module in `src/ui.rs`
- [x] 1.2 Add `pub const BG_DIM: Color` — `#1B1B1B`
- [x] 1.3 Add `pub const BG0: Color` — `#282828`
- [x] 1.4 Add `pub const BG1: Color` — `#32302F`
- [x] 1.5 Add `pub const BG2: Color` — `#32302F`
- [x] 1.6 Add `pub const BG3: Color` — `#45403D`
- [x] 1.7 Add `pub const BG4: Color` — `#45403D`
- [x] 1.8 Add `pub const BG5: Color` — `#5A524C`
- [x] 1.9 Add `pub const BG_STATUSLINE1: Color` — `#32302F`
- [x] 1.10 Add `pub const BG_STATUSLINE2: Color` — `#3A3735`
- [x] 1.11 Add `pub const BG_STATUSLINE3: Color` — `#504945`
- [x] 1.12 Add `pub const BG_CURRENT_WORD: Color` — `#3C3836`
- [x] 1.13 Add `pub const BG_DIFF_RED: Color` — `#402120`
- [x] 1.14 Add `pub const BG_DIFF_GREEN: Color` — `#34381B`
- [x] 1.15 Add `pub const BG_DIFF_BLUE: Color` — `#0E363E`
- [x] 1.16 Add `pub const BG_VISUAL_RED: Color` — `#4C3432`
- [x] 1.17 Add `pub const BG_VISUAL_GREEN: Color` — `#3B4439`
- [x] 1.18 Add `pub const BG_VISUAL_BLUE: Color` — `#374141`
- [x] 1.19 Add `pub const BG_VISUAL_YELLOW: Color` — `#4F422E`
- [x] 1.20 Add `pub const BG_VISUAL_PURPLE: Color` — `#443840`
- [x] 1.21 Add `pub const FG0: Color` — `#D4BE98`
- [x] 1.22 Add `pub const FG1: Color` — `#DDC7A1`
- [x] 1.23 Add `pub const RED: Color` — `#EA6962`
- [x] 1.24 Add `pub const GREEN: Color` — `#A9B665`
- [x] 1.25 Add `pub const BLUE: Color` — `#7DAEA3`
- [x] 1.26 Add `pub const YELLOW: Color` — `#D8A657`
- [x] 1.27 Add `pub const PURPLE: Color` — `#D3869B`
- [x] 1.28 Add `pub const ORANGE: Color` — `#E78A4E`
- [x] 1.29 Add `pub const AQUA: Color` — `#89B482`
- [x] 1.30 Add `pub const GREY0: Color` — `#7C6F64`
- [x] 1.31 Add `pub const GREY1: Color` — `#928374`
- [x] 1.32 Add `pub const GREY2: Color` — `#A89984`
- [x] 1.33 Add `pub const BG_RED: Color` — `#EA6962`
- [x] 1.34 Add `pub const BG_GREEN: Color` — `#A9B665`
- [x] 1.35 Add `pub const BG_YELLOW: Color` — `#D8A657`
- [x] 1.36 Add `#[tracing::instrument]` (or `skip_all`) on any public function if required; for a constants-only module this may not be needed — verify with `just lint`

## 2. Add UiType State

- [x] 2.1 Define `pub enum UiType { EGui, Bevy }` in `src/ui.rs` (or a new `src/ui/ui_type.rs` submodule declared from `src/ui.rs`)
- [x] 2.2 Derive `States`, `Debug`, `Clone`, `Copy`, `Eq`, `PartialEq`, `Hash` — matching the `UiState` derive list
- [x] 2.3 Implement `Default` returning `UiType::EGui`
- [x] 2.4 Register `UiType` with `app.init_state::<UiType>()` in `UiPlugin::build`

## 3. Add F2 Toggle System

- [x] 3.1 Add a system `toggle_ui_type_on_f2(keys: Res<ButtonInput<KeyCode>>, ui_type: Res<State<UiType>>, mut next: ResMut<NextState<UiType>>)` in `src/ui.rs`
- [x] 3.2 In the system body: `if keys.just_pressed(KeyCode::F2) { next.set(match ui_type.get() { UiType::EGui => UiType::Bevy, UiType::Bevy => UiType::EGui }); }`
- [x] 3.3 Add `#[tracing::instrument(skip_all, level = "trace")]` to the toggle system
- [x] 3.4 Register the toggle system with `app.add_systems(Update, toggle_ui_type_on_f2)` in `UiPlugin::build`

## 4. Gate EGUI Systems on UiType::EGui

- [x] 4.1 Add `.run_if(in_state(UiType::EGui))` to the `draw_ui_topbar` system registration in `UiPlugin::build`
- [x] 4.2 Update `draw_ui_explorer` to `.run_if(in_state(UiState::Explorer).and(in_state(UiType::EGui)))`
- [x] 4.3 Update `draw_ui_scenario` to `.run_if(in_state(UiState::Scenario).and(in_state(UiType::EGui)))`
- [x] 4.4 Update `draw_ui_results` to `.run_if(in_state(UiState::Results).and(in_state(UiType::EGui)))`
- [x] 4.5 Update `draw_ui_volumetric` to `.run_if(in_state(UiState::Volumetric).and(in_state(UiType::EGui)))`

## 5. Verify

- [x] 5.1 Run `just check` — no compile errors or clippy warnings
- [x] 5.2 Run `just lint` — clippy-tracing span check passes for all new public functions
- [x] 5.3 Run `just test` — all existing tests pass
- [x] 5.4 Run `just run`, confirm application launches identically to before (default `UiType::EGui`)
- [x] 5.5 Press F2 in the running app and confirm the EGUI panels disappear; press F2 again and confirm they return
