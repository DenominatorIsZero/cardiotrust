## 1. Extend UiState and add SidebarState resource

- [x] 1.1 Add `Home` and `Scheduler` variants to the `UiState` enum in `src/ui.rs`
- [x] 1.2 Add `#[derive(Resource)] SidebarState` struct (`expanded: bool`, `width: f32`) to `src/ui.rs`
- [x] 1.3 Register `SidebarState` as a resource in `UiPlugin::build` (default: expanded=true, width=200.0)
- [x] 1.4 Verify `cargo check` passes with no new warnings or errors

## 2. Create the bevy_shell module skeleton

- [x] 2.1 Create `src/ui/bevy_shell/mod.rs` with a `BevyShellPlugin` struct that implements `Plugin`
- [x] 2.2 Create empty stub files: `sidebar.rs`, `breadcrumb.rs`, `content_area.rs`, `routing.rs` inside `src/ui/bevy_shell/`
- [x] 2.3 Declare `mod bevy_shell;` in `src/ui.rs` and add `BevyShellPlugin` to `UiPlugin::build`
- [x] 2.4 Verify `cargo check` passes

## 3. Implement the root layout and content area

- [x] 3.1 In `content_area.rs`, define a `spawn_root_layout` system that spawns the root `Node` (Row, 100%×100%) with sidebar placeholder and content column children
- [x] 3.2 Add the `BreadcrumbBar` marker component and spawn a 32 px height node at the top of the content column with `BG_DIM` background
- [x] 3.3 Add the `ContentSlot` marker component and spawn a `flex-grow: 1` node below the breadcrumb bar
- [x] 3.4 Register `spawn_root_layout` to run `OnEnter(UiType::Bevy)` in `BevyShellPlugin`
- [x] 3.5 Add a despawn system that runs `OnExit(UiType::Bevy)` to clean up the root layout entity
- [x] 3.6 Verify switching to Bevy mode (F2) spawns the layout without panic; switching back despawns it

## 4. Implement the sidebar rail

- [x] 4.1 In `sidebar.rs`, define `NavItem` component with a `target: UiState` field and `NavItemDisabled` marker component
- [x] 4.2 Spawn the sidebar `Node` (Column, `BG0`, width from `SidebarState`) as a child of the root row in `spawn_root_layout`
- [x] 4.3 Spawn the logo area node (56 px height, `BG1`, "CT" text) at the top of the sidebar with a `Button` for Home navigation
- [x] 4.4 Spawn separator nodes (1 px height, `BG3` background) at the correct positions
- [x] 4.5 Spawn `NavItem` button nodes for Home, Explorer, Scenario, Results, Volumetric in order, with icon placeholder text and label text children
- [x] 4.6 Spawn a spacer node (`flex-grow: 1`) and then a Scheduler `NavItem` button at the bottom
- [x] 4.7 Spawn the collapse/expand chevron button at the very bottom of the sidebar

## 5. Implement sidebar visual states

- [x] 5.1 In `sidebar.rs`, add `update_nav_item_visual_states` system that reads `Changed<Interaction>` on `NavItem` buttons and applies colors from `ui::colors` to match the state table in design.md
- [x] 5.2 Add logic to the same system to check `UiState` and mark the matching `NavItem` as active
- [x] 5.3 Add logic to apply `NavItemDisabled` to items that fail precondition guards (Scenario: no selection; Results/Volumetric: scenario not Done)
- [x] 5.4 Add `handle_nav_item_click` system that reads `Interaction::Pressed` on non-disabled `NavItem` buttons and calls `NextState::<UiState>::set`
- [x] 5.5 Register both systems in `BevyShellPlugin` under `in_state(UiType::Bevy)`

## 6. Implement sidebar collapse/expand and auto-collapse

- [x] 6.1 Add `handle_chevron_click` system in `sidebar.rs` that toggles `SidebarState::expanded` on chevron button press and updates `SidebarState::width` (200.0 ↔ 56.0)
- [x] 6.2 Add `apply_sidebar_width` system that reads `Changed<SidebarState>` and updates the sidebar `Node`'s `width` style value
- [x] 6.3 Add `auto_collapse_on_narrow_viewport` system in `sidebar.rs` that runs in `PreUpdate`, reads `Window` width, and writes `SidebarState` if the viewport crosses the 900 px threshold
- [x] 6.4 Verify that shrinking the window collapses the sidebar and widening it restores the prior state

## 7. Implement breadcrumb bar updates

- [x] 7.1 In `breadcrumb.rs`, define a `BreadcrumbText` marker component for the text node inside the bar
- [x] 7.2 Add `update_breadcrumb` system that runs on `StateTransitionEvent<UiState>` and updates the breadcrumb text to reflect the current view hierarchy path
- [x] 7.3 Register `update_breadcrumb` in `BevyShellPlugin`
- [x] 7.4 Verify breadcrumb text changes correctly when navigating between views

## 8. Implement keyboard shortcuts

- [x] 8.1 In `routing.rs`, add `handle_keyboard_shortcuts` system that reads `KeyCode` digit presses 1–6 and maps each to the corresponding `UiState` variant
- [x] 8.2 Apply precondition guards inside the system (same rules as nav items: Scenario needs selection, Results/Volumetric need Done status)
- [x] 8.3 Add Escape handling: map current `UiState` to its parent and call `NextState::set`
- [x] 8.4 Register `handle_keyboard_shortcuts` in `BevyShellPlugin` under `in_state(UiType::Bevy)`
- [x] 8.5 Verify each digit key navigates to the correct view; verify Escape traverses the parent chain

## 9. Final integration checks

- [x] 9.1 Run `cargo check` with zero errors and zero warnings
- [x] 9.2 Run `just lint` (clippy-tracing span check) — add `#[tracing::instrument(skip_all)]` to any public functions that are missing it
- [x] 9.3 Run `just fmt` to apply nightly rustfmt formatting
- [x] 9.4 Run `just test` to confirm no existing tests are broken
- [x] 9.5 Manually switch to Bevy mode (F2), verify sidebar renders, nav items change on click, breadcrumb updates, keyboard shortcuts work, and switching back to EGui mode (F2) shows the original egui UI unchanged
