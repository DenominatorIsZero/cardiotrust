## 1. Add `ProjectState` resource and refactor `ScenarioList` loading

- [ ] 1.1 Add `#[derive(Resource, Debug)] ProjectState` to `src/lib.rs` with fields `current_path: Option<PathBuf>` and `recent: Vec<PathBuf>` (cap 8). Add `Default` impl with both fields empty.
- [ ] 1.2 Add `ScenarioList::load_from(path: &Path) -> Result<Self>` by extracting the body of the current `load()`. Update `load()` to call `load_from(Path::new("./results"))`. Update `Default` to call `load_from("./results")` via `load()`.
- [ ] 1.3 Add `ProjectState::push_recent(path: PathBuf)` method: prepend `path`, remove duplicate, truncate to 8.
- [ ] 1.4 Add `ProjectState::save_recent(&self) -> Result<()>` that writes `recent` as a TOML list to `~/.config/cardiotrust/recent_projects.toml` (create dirs if needed). No-op on WASM (`#[cfg(not(target_arch = "wasm32"))]`).
- [ ] 1.5 Add `ProjectState::load_recent() -> Vec<PathBuf>` that reads the same TOML file; returns empty vec on any error. No-op returns empty vec on WASM.
- [ ] 1.6 Register `ProjectState` in `src/bin/main.rs`: replace `init_resource::<ScenarioList>()` with `insert_resource(ScenarioList::empty())` and add `insert_resource(ProjectState { recent: ProjectState::load_recent(), ..Default::default() })`.
- [ ] 1.7 Verify `cargo check` passes with no new warnings.

## 2. Change startup defaults to Bevy + Home

- [ ] 2.1 Change `UiState::default()` in `src/ui.rs` to return `Self::Home`.
- [ ] 2.2 Change `UiType::default()` in `src/ui.rs` to return `Self::Bevy`.
- [ ] 2.3 Verify the app compiles and opens in Bevy mode showing the empty content slot on the Home state (sidebar visible, no crash).

## 3. Implement project loading system

- [ ] 3.1 Create `src/ui/bevy_shell/project.rs` with a `load_project_on_path_change` system: reads `Changed<ProjectState>`, if `current_path` is `Some`, calls `ScenarioList::load_from`, logs warnings on error, inserts updated `ScenarioList` via `commands.insert_resource`, resets `SelectedSenario`, and calls `next_state.set(UiState::Explorer)`.
- [ ] 3.2 Register `load_project_on_path_change` in `BevyShellPlugin::build` under `in_state(UiType::Bevy)` in the `Update` schedule.
- [ ] 3.3 Add `mod project;` to `src/ui/bevy_shell/mod.rs`.
- [ ] 3.4 Add `#[tracing::instrument(skip_all)]` to `load_project_on_path_change`. Verify `just lint` passes.

## 4. Update sidebar to gate project-dependent items on `ProjectState`

- [ ] 4.1 In `sidebar.rs`, add `Res<ProjectState>` to `update_nav_item_visual_states` system parameters.
- [ ] 4.2 In `update_nav_item_visual_states`, when `project_state.current_path.is_none()`, apply `NavItemDisabled` to Explorer, Scenario, Results, Volumetric, and Scheduler items. Home item is never disabled by this rule.
- [ ] 4.3 Update `handle_nav_item_click` to also block items disabled by missing project (already handled by `NavItemDisabled` check if the above is correct — verify no additional changes needed).
- [ ] 4.4 Update `routing.rs` keyboard shortcut handler: guard shortcuts `2`–`6` when `project_state.current_path.is_none()`; shortcut `1` (Home) is always allowed.

## 5. Build the Home view Bevy UI

- [ ] 5.1 Create `src/ui/bevy_shell/home.rs` with a `HomeViewRoot` marker component.
- [ ] 5.2 Implement `spawn_home_view` system: spawns a centered column node (max-width ~800px) as a child of `ContentSlot`. Include:
  - Title node: "CardioTrust" text (`FG0`, large font size equivalent).
  - Subtitle node: "Cardiac Electrophysiological Simulation" (`GREY1`).
  - Open Project panel node (`BG1` background, dashed `GREY1` border, 8px radius): contains an "Open Project Folder" button.
  - Recent Projects panel node (`BG1`): contains up to 8 `RecentProjectEntry` button nodes built from `ProjectState::recent`, or a "No recent projects" text node when empty.
  - WASM-only demo placeholder panel (`#[cfg(target_arch = "wasm32")]`): three static cards labelled "Demo Project (Coming Soon)".
- [ ] 5.3 Register `spawn_home_view` to run `OnEnter(UiState::Home)` in `BevyShellPlugin`.
- [ ] 5.4 Implement `despawn_home_view` system that despawns all `HomeViewRoot` entities. Register it `OnExit(UiState::Home)`.
- [ ] 5.5 Add `mod home;` to `src/ui/bevy_shell/mod.rs`.

## 6. Wire the Open Project button

- [ ] 6.1 Add an `OpenProjectButton` marker component to the Open Project button node.
- [ ] 6.2 Add `handle_open_project_button` system in `home.rs`: on `Interaction::Pressed`, call `rfd::FileDialog::new().pick_folder()` (synchronous), if `Some(path)` is returned, call `project_state.push_recent(path.clone())`, save recent via `project_state.save_recent()` (log on error), and set `project_state.current_path = Some(path)`.
- [ ] 6.3 Register `handle_open_project_button` in `BevyShellPlugin` under `in_state(UiType::Bevy).and(in_state(UiState::Home))`.
- [ ] 6.4 Add `RecentProjectEntry` component with a `path: PathBuf` field.
- [ ] 6.5 Add `handle_recent_project_click` system: on `Interaction::Pressed` for `RecentProjectEntry` buttons, set `project_state.current_path = Some(entry.path.clone())` and call `project_state.push_recent` + `save_recent`.
- [ ] 6.6 Register `handle_recent_project_click` in `BevyShellPlugin` under the same conditions.

## 7. Final integration checks

- [ ] 7.1 Run `cargo check` with zero errors and zero warnings.
- [ ] 7.2 Run `just lint` — add `#[tracing::instrument(skip_all)]` to any public functions that are missing it.
- [ ] 7.3 Run `just fmt` to apply nightly rustfmt formatting.
- [ ] 7.4 Run `just test` to confirm no existing tests are broken.
- [ ] 7.5 Manually verify: app starts on Home view (Bevy mode); clicking Open Project opens a folder dialog; selecting `./results/` loads scenarios and transitions to Explorer; the sidebar shows Explorer as enabled; pressing F2 switches to EGui (Explorer still visible); pressing F2 again returns to Bevy Home view.
- [ ] 7.6 Verify recent projects: close and reopen the app; the previously opened folder appears in the Recent Projects list; clicking it loads the project.
