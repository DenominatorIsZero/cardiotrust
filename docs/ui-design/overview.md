# CardioTrust UI Redesign -- Overview

> Status: Design Draft
> Date: 2026-03-14

## Motivation

The current egui UI is functional but has several problems:

- **Explorer**: Dense table is hard to scan; no visual identity for scenarios
- **Scenario editor**: Wall of sliders in two columns is overwhelming and hard to navigate
- **Results**: Single dropdown with 30+ image types is clunky; no way to compare
- **Volumetric sidebar**: Cramped controls that are hard to discover
- **Navigation**: Flat topbar with text buttons isn't visually appealing
- **No project concept**: The app hardcodes `./results/` -- no way to switch between experiment sets

The app will be hosted as a WASM build on a personal portfolio site, so the UI needs to look polished and be immediately understandable to someone encountering it for the first time.

## Design Goals

1. **Portfolio-quality visuals** -- Gruvbox Material dark theme, consistent spacing, clean typography
2. **WASM-first** -- Every view must work in the browser; no native-only features in the core UI
3. **Scannable at a glance** -- Cards, badges, progress indicators instead of dense tables
4. **Progressive disclosure** -- Show the essentials first, details on demand
5. **Clear navigation** -- Always know where you are and how to get elsewhere

## Theme

Gruvbox Material dark palette (already defined in `src/ui/colors.rs`):

| Role | Color | Hex |
|------|-------|-----|
| Background (base) | bg0 | `#282828` |
| Background (elevated) | bg1 | `#32302F` |
| Background (surface) | bg2 | `#32302F` |
| Background (hover) | bg3 | `#45403D` |
| Foreground (primary) | fg0 | `#D4BE98` |
| Foreground (secondary) | fg1 | `#DDC7A1` |
| Accent (primary) | orange | `#E78A4E` |
| Accent (secondary) | aqua | `#89B482` |
| Success | green | `#A9B665` |
| Warning | yellow | `#D8A657` |
| Error | red | `#EA6962` |
| Info | blue | `#7DAEA3` |
| Muted text | grey1 | `#928374` |

## Views

The redesign introduces 6 views (up from 4), plus a persistent navigation shell:

| View | Purpose | New? |
|------|---------|------|
| **Home** | Project/folder selection, recent projects | Yes |
| **Explorer** | Browse scenarios in active project (card grid) | Redesigned |
| **Scenario** | Edit scenario configuration (tabbed + collapsible) | Redesigned |
| **Results** | View generated images (categorized gallery) | Redesigned |
| **Volumetric** | 3D visualization + controls | Redesigned |
| **Scheduler** | Monitor running jobs, progress, ETCs | Yes (extracted) |

## Navigation Model

A **left sidebar rail** replaces the top bar:

```
+------+------------------------------------------+
| [CT] |                                          |
|      |                                          |
| Home |            Content Area                  |
| Expl |                                          |
| Scen |                                          |
| Resu |                                          |
| Volu |                                          |
|      |                                          |
|      |                                          |
| Schd |                                          |
+------+------------------------------------------+
```

- Top: App logo/initials ("CT")
- Middle: View icons with labels (or icons-only when collapsed)
- Bottom: Scheduler icon (separated, since it's a utility view)
- Active view highlighted with accent color bar
- Context-dependent items (e.g., Scenario/Results/Volumetric) are greyed out when no scenario is selected, and hidden on the Home view

See [navigation.md](./navigation.md) for details.

## View Summaries

### Home View
Project selector with folder browser and recent projects list. Landing page for new visitors. On WASM, this shows bundled demo projects or a file upload mechanism.

See [home-view.md](./home-view.md).

### Explorer View
Card grid of scenarios with status badges, thumbnail previews, key metrics. Supports filtering/sorting. "New Scenario" as a prominent action card.

See [explorer-view.md](./explorer-view.md).

### Scenario View
Single-column editor with tabs (Simulation / Algorithm / Model) and collapsible sections within each tab. Section headers have icons. Status bar at top with scenario identity and actions.

See [scenario-view.md](./scenario-view.md).

### Results View
Categorized image gallery with tabs for image categories (Spatial Maps, Metrics, Losses, Time Functions). Thumbnail grid within each category. Click to view full-size. On-demand generation preserved.

See [results-view.md](./results-view.md).

### Volumetric View
3D viewport takes most of the screen. Controls move to a collapsible overlay panel (right side) instead of a fixed sidebar. Bottom plot remains but is resizable.

See [volumetric-view.md](./volumetric-view.md).

### Scheduler View
Dashboard showing: total progress, number of active/queued/completed jobs, ETC for current batch. Per-scenario rows with individual progress bars. Start/Stop controls.

See [scheduler-view.md](./scheduler-view.md).

## WASM Considerations

| Concern | Approach |
|---------|----------|
| File system access | Use `rfd` (rust file dialog) for native; for WASM, provide file upload + bundled demo data |
| Project persistence | Native: filesystem; WASM: IndexedDB or in-memory with download/upload |
| GPU compute (OpenCL) | Not available in WASM; CPU-only with WebWorker for background execution |
| 3D rendering | Bevy supports WebGPU/WebGL2 -- no change needed |
| Image generation | Same on-demand approach; images stored in memory (not disk) for WASM |

## Implementation Strategy

The new UI will be built under the `UiType::Bevy` path, using Bevy's native UI system (not egui). The existing egui UI remains functional under `UiType::EGui` and can be toggled with F2 during development. This allows incremental migration without breaking the working app.

Key phases:
1. **Navigation shell** -- Sidebar + view routing
2. **Home view** -- Project selection
3. **Explorer view** -- Card grid
4. **Scenario view** -- Tabbed editor
5. **Scheduler view** -- Job monitoring
6. **Results view** -- Gallery
7. **Volumetric view** -- Overlay controls
8. **Remove egui UI** -- Once Bevy UI is complete

## Document Index

- [overview.md](./overview.md) -- This document
- [navigation.md](./navigation.md) -- Navigation shell & layout
- [home-view.md](./home-view.md) -- Home / project selector
- [explorer-view.md](./explorer-view.md) -- Scenario explorer (card grid)
- [scenario-view.md](./scenario-view.md) -- Scenario editor
- [results-view.md](./results-view.md) -- Results gallery
- [volumetric-view.md](./volumetric-view.md) -- 3D visualization
- [scheduler-view.md](./scheduler-view.md) -- Scheduler dashboard
