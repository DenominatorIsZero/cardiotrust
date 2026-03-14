## Context

CardioTrust uses Bevy as its runtime with egui overlaid via `bevy_egui` for all current UI. The app has a `UiType` state (EGui / Bevy) and a `UiState` enum (Explorer, Scenario, Results, Volumetric). Phase 1 of the redesign builds the Bevy-native navigation shell that will house all future Bevy views; it runs only when `UiType::Bevy` is active, leaving egui entirely undisturbed.

The sidebar rail, breadcrumb bar, and content area wrapper are pure Bevy UI node trees — no egui involvement. Visual constants are already available in `src/ui/colors.rs`.

## Goals / Non-Goals

**Goals:**
- Render a left sidebar rail as a Bevy UI node tree (logo, nav items, spacer, scheduler item).
- Express six-view routing (`Home`, `Explorer`, `Scenario`, `Results`, `Volumetric`, `Scheduler`) through the existing `UiState` enum.
- Support sidebar expanded (200 px) / collapsed (56 px) states with a toggle chevron.
- Auto-collapse below 900 px viewport width.
- Show a breadcrumb/context bar at the top of the content column.
- Wire keyboard shortcuts 1–6 and Escape for view switching.
- All new systems gated on `in_state(UiType::Bevy)`.

**Non-Goals:**
- Implementing any content view (Home, Explorer, etc.) — the content slot shows an empty placeholder.
- Modifying or removing egui systems.
- Animation or transition effects between views.
- WASM-specific file-system or IndexedDB work.

## Decisions

### 1. New module: `src/ui/bevy_shell/`

All shell code lives under a new sub-module of `src/ui`:

```
src/ui/bevy_shell/
  mod.rs          — BevyShellPlugin; spawns the root layout node and registers systems
  sidebar.rs      — Sidebar node tree, SidebarState resource, nav item components
  breadcrumb.rs   — BreadcrumbBar node and update system
  content_area.rs — Content placeholder node; swapped by future view phases
  routing.rs      — Keyboard shortcut system + precondition guard helpers
```

**Rationale**: Mirrors the existing flat file layout (`topbar.rs`, `explorer.rs`, …) but uses a sub-directory because the shell comprises multiple collaborating files. Keeps egui files completely separate.

### 2. Extend `UiState` with `Home` and `Scheduler`

Add two new variants to the existing `UiState` enum. The existing `Explorer`, `Scenario`, `Results`, and `Volumetric` variants keep their indices; new variants append. Default changes from `Explorer` to `Home`.

**Alternative considered**: Separate `BevyUiState` enum.  
**Rejected**: Two routing enums would require every precondition guard to branch on `UiType`. A single enum is simpler; egui systems already gate on `UiType::EGui`, so the two new variants are simply unreachable under egui.

### 3. `SidebarState` as a `Resource`

```rust
#[derive(Resource)]
pub struct SidebarState {
    pub expanded: bool,
    pub width: f32,          // 200.0 or 56.0
}
```

Auto-collapse is driven by a system that reads `Window` width each frame and writes `SidebarState`. The sidebar node's `Style::width` is updated to match via a `Changed<SidebarState>` query.

**Alternative**: Store expanded/collapsed as part of `UiState`.  
**Rejected**: `UiState` represents *which view is shown*, not sidebar geometry; conflating them would complicate precondition guards.

### 4. Bevy UI node tree structure

```
Root (Node, width: 100%, height: 100%, FlexDirection::Row)
├── Sidebar (Node, FlexDirection::Column, width: SidebarState.width, bg0)
│   ├── LogoArea (Node, height: 56px, bg1) — "CT" text
│   ├── Separator
│   ├── NavItem::Home
│   ├── Separator
│   ├── NavItem::Explorer
│   ├── NavItem::Scenario
│   ├── NavItem::Results
│   ├── NavItem::Volumetric
│   ├── Spacer (flex-grow: 1)
│   ├── Separator
│   ├── NavItem::Scheduler
│   └── CollapseChevron (Button)
└── ContentColumn (Node, FlexDirection::Column, flex-grow: 1)
    ├── BreadcrumbBar (Node, height: 32px, bg_dim)
    └── ContentSlot (Node, flex-grow: 1) — placeholder; future phases replace children
```

### 5. NavItem visual states via `Interaction` component

Bevy's built-in `Interaction` component (`None`, `Hovered`, `Pressed`) drives color changes via a system that reads `Changed<Interaction>` on nav item buttons. Active state is derived from comparing the item's `UiState` marker component against the current `UiState`.

| State | Left border | Background | Icon/text tint |
|---|---|---|---|
| Default | none | transparent | `GREY1` / `FG1` |
| Hover | none | `BG3` | `FG0` |
| Active | 3px `ORANGE` | `BG1` | `ORANGE` / `FG0` |
| Disabled | none | transparent | `BG3` |

Disabled nav items use `Interaction::None` forcibly and ignore click events via a marker component `NavItemDisabled`.

### 6. Precondition guards live in `routing.rs`

A single `update_nav_item_enabled` system runs after `UiState` transitions, computing which items should be disabled:

- `Home`: always enabled.
- `Explorer`, `Scheduler`: enabled when a project is loaded (deferred to later phases; always enabled for now).
- `Scenario`: enabled when `SelectedSenario::index.is_some()`.
- `Results`, `Volumetric`: enabled when selected scenario has `Status::Done`.

### 7. Keyboard shortcuts

A `handle_keyboard_shortcuts` system reads `KeyCode` input and calls `NextState::<UiState>::set(...)` subject to the same precondition guards. Shortcut map: 1=Home, 2=Explorer, 3=Scenario, 4=Results, 5=Volumetric, 6=Scheduler, Escape=parent view.

Escape parent mapping: Results→Scenario, Scenario→Explorer, Volumetric→Scenario, all others→Home.

## Risks / Trade-offs

- **Bevy UI layout quirks** → Node flex layout in Bevy can be sensitive to missing size constraints. Mitigation: set explicit `width`/`height` on fixed-size nodes (logo area, breadcrumb bar); use `flex_grow` only on the spacer and content slot.
- **`UiState` default change (Explorer → Home)** → The egui Explorer system currently starts as the default landing view. Changing the default to `Home` means switching to `UiType::EGui` will start on `Home`, which has no egui implementation. Mitigation: keep `UiState::Explorer` as default for now; the `BevyShellPlugin` transitions to `Home` as its `OnEnter(UiType::Bevy)` setup step.
- **Two `UiState` values with no egui handler** → `Home` and `Scheduler` variants won't match any egui draw system, so the egui central panel will be empty if reached under egui. Mitigation: document the invariant; add `#[allow(dead_code)]` if Clippy warns.
- **Sidebar width as a float resource** → If the window resize event and the sidebar render happen in the same frame, there's a one-frame flicker. Mitigation: run the auto-collapse system in `PreUpdate` before layout.

## Open Questions

- Should the content slot be despawned and respawned when views change, or should all view nodes be spawned once and hidden/shown via `Visibility`? (Recommend spawn/despawn for simplicity in Phase 1; revisit in Phase 3 when Explorer view is implemented.)
- Exact icon assets: SVG, embedded font glyphs, or Unicode text placeholders for Phase 1? (Recommend Unicode text placeholders to avoid asset pipeline work; replace in a later phase.)
