# Home View -- Project Selector

## Purpose

The landing page. Users select a project folder (directory containing scenario results) before doing anything else. On WASM, this also serves as a first-impression portfolio piece.

## Layout

```
+------------------------------------------------------------------+
|                                                                  |
|                         CardioTrust                              |
|              Cardiac Electrophysiological Simulation             |
|                                                                  |
|  +---------------------------+  +-----------------------------+  |
|  |                           |  |                             |  |
|  |    Open Project Folder    |  |       Recent Projects       |  |
|  |                           |  |                             |  |
|  |   [  Browse...  ]         |  |   results/                  |  |
|  |                           |  |   experiment-2026-03/       |  |
|  |   Drop a folder here     |  |   mri-study-v2/             |  |
|  |   or click to browse      |  |                             |  |
|  |                           |  |                             |  |
|  +---------------------------+  +-----------------------------+  |
|                                                                  |
|  +------------------------------------------------------------+  |
|  |                    Demo Projects (WASM)                     |  |
|  |                                                             |  |
|  |  [ Handcrafted Heart ]  [ MRI Model ]  [ Parameter Sweep ] |  |
|  |                                                             |  |
|  +------------------------------------------------------------+  |
|                                                                  |
+------------------------------------------------------------------+
```

## Sections

### Hero / Title Area
- App name "CardioTrust" in large text (`fg0`, ~32px equivalent)
- Subtitle in `grey1`: "Cardiac Electrophysiological Simulation"
- Optional: subtle animated heart icon or gradient background accent
- Minimal -- this is a tool, not a marketing page

### Open Project Panel
- **Native**: A drop zone / browse button that opens `rfd::FileDialog` to select a directory
- **WASM**: File upload zone (drag & drop a zip of scenario files) or path input
- Visual: Dashed border card (`bg1` background, `grey1` dashed border), icon in center
- On hover: border becomes `orange`, background becomes `bg2`

### Recent Projects
- List of previously opened project paths (stored in app config / localStorage for WASM)
- Each entry shows:
  - Folder name (bold, `fg0`)
  - Full path below in `grey1` (truncated with ellipsis)
  - Number of scenarios if known (small badge)
  - Last opened timestamp
- Click to open immediately
- "Clear recent" link at bottom in `grey1`
- Max 5-8 entries

### Demo Projects (WASM only)
- Pre-bundled scenario sets that ship with the WASM build
- Displayed as cards with:
  - Title (e.g., "Handcrafted Heart Model")
  - Brief description (1 line)
  - Small preview image if available
- Clicking loads the demo data into memory and navigates to Explorer
- This ensures the WASM version is immediately usable without file uploads

## Behavior

1. On app start, if only one project exists (e.g., `./results/` in native), auto-load it and go to Explorer
2. If multiple recent projects exist, show Home
3. After selecting a project: load `ScenarioList` from the directory, transition to Explorer
4. If the selected folder has no valid scenarios: show Explorer with empty state + "New Scenario" prompt

## WASM-Specific

- `rfd` supports WASM file dialogs (single file upload), but directory picking is limited
- Alternative: let users upload a `.zip` file containing scenario TOML + result binaries
- Demo projects are embedded in the WASM binary as compressed assets
- Recent projects stored in `localStorage` (just names/keys, not data)

## Styling

- Content is centered, max-width ~800px, vertically centered in the viewport
- Generous padding (40-60px)
- Cards use `bg1` with subtle `bg3` border, 8px border-radius
- Hover states on all interactive elements
- Overall feel: clean, spacious, professional
