# Navigation & Layout Shell

## Structure

The app uses a persistent **sidebar rail** on the left + a **content area** that fills the rest of the screen. No top bar.

```
+--------+------------------------------------------------+
|  Logo  |                                                |
|        |                                                |
|  Home  |                                                |
|  ----  |                                                |
|  Expl  |            Content Area                        |
|  Scen  |          (active view fills this)              |
|  Resu  |                                                |
|  Vol   |                                                |
|        |                                                |
|        |                                                |
|  ----  |                                                |
|  Schd  |                                                |
+--------+------------------------------------------------+
```

## Sidebar Rail

### Dimensions
- **Width (expanded)**: 200px -- icon + label text
- **Width (collapsed)**: 56px -- icon only
- **Toggle**: Small chevron button at bottom of sidebar, or auto-collapse below a viewport width threshold (e.g., < 900px)

### Elements (top to bottom)

1. **Logo area** (top, 56px height)
   - Shows "CT" monogram or a small heart icon
   - Background: `bg1`
   - Clicking navigates to Home

2. **Separator line** (1px, `bg3`)

3. **Home** -- House icon
   - Always visible, always enabled

4. **Separator line** (subtle, 1px)

5. **Explorer** -- Grid/cards icon
   - Visible when a project is loaded
   - Badge: number of scenarios (small pill)

6. **Scenario** -- Sliders/config icon
   - Visible when a project is loaded
   - Greyed out when no scenario is selected
   - Shows truncated scenario name as subtitle when expanded

7. **Results** -- Chart/image icon
   - Greyed out unless selected scenario has `Status::Done`

8. **Volumetric** -- 3D cube icon
   - Greyed out unless selected scenario has `Status::Done`

9. **Flexible spacer** (pushes scheduler to bottom)

10. **Separator line**

11. **Scheduler** -- Play/activity icon
    - Always visible when a project is loaded
    - Badge: number of running jobs (if > 0), colored green when active

### Visual States

| State | Style |
|-------|-------|
| Default | Icon in `grey1`, label in `fg1` |
| Hover | Background `bg3`, icon brightens to `fg0` |
| Active (current view) | Left border 3px `orange`, icon in `orange`, label in `fg0`, background `bg1` |
| Disabled | Icon in `bg3`, not clickable, no hover effect |

### Breadcrumb / Context Bar

At the top of the content area (not in the sidebar), a thin context bar (32px height) shows:
- Current location path: e.g., `My Project > Scenario v8_Dr_2026-03-14 > Results`
- This provides orientation and allows quick navigation to parent views
- Background: `bg_dim`
- Text: `grey1`, with the final segment in `fg0`

## View Routing

```
Home ──> Explorer ──> Scenario ──> Results
                  │             └──> Volumetric
                  └──> Scheduler
```

### Transitions
- **Home -> Explorer**: User selects/opens a project folder
- **Explorer -> Scenario**: User clicks a scenario card
- **Scenario -> Results/Volumetric**: Buttons in scenario header, or sidebar
- **Any -> Scheduler**: Sidebar click (independent of scenario selection)
- **Any -> Home**: Sidebar logo or Home icon

### Back Navigation
- Breadcrumb links allow jumping back
- Browser back button works in WASM (push history state on view changes)
- Escape key: returns to the parent view (Results->Scenario, Scenario->Explorer, etc.)

## Responsive Behavior

| Viewport Width | Sidebar | Content |
|----------------|---------|---------|
| >= 1200px | Expanded (200px) | Full layout |
| 900-1199px | Collapsed (56px, icons only) | Adapts to wider space |
| < 900px | Hidden (hamburger toggle at top-left of content) | Full width |

For WASM embedded in a portfolio site, the app should work well at ~1200px minimum width, but gracefully handle smaller sizes.

## Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `1` | Go to Home |
| `2` | Go to Explorer |
| `3` | Go to Scenario |
| `4` | Go to Results |
| `5` | Go to Volumetric |
| `6` | Go to Scheduler |
| `Esc` | Go to parent view |
| `F2` | Toggle egui/Bevy UI (dev only) |

## Bevy Implementation Notes

- The sidebar is a Bevy UI `Node` with `FlexDirection::Column`, fixed width, `bg0` background
- Each nav item is a `Button` node with icon (image or text) + optional label
- The content area is a single `Node` that swaps children based on `UiState`
- View transitions can use simple opacity fade or instant swap
- The sidebar state (expanded/collapsed) is stored as a `Resource`
- The context bar is a separate `Node` at the top of the content column
