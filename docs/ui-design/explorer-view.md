# Explorer View -- Scenario Card Grid

## Purpose

Browse all scenarios in the active project at a glance. Replaces the dense table with a visual card grid that surfaces the most important information.

## Layout

```
+------------------------------------------------------------------+
| Context bar:  My Project > Explorer                              |
+------+---------+------------------------------------------------+
| [Filter/Sort toolbar]                          [+ New Scenario]  |
+------------------------------------------------------------------+
|                                                                  |
|  +------------------+  +------------------+  +------------------+|
|  | Status: Done     |  | Status: Running  |  | Status: Planning ||
|  |                  |  |                  |  |                  ||
|  | [Thumbnail]      |  | [Progress 47%]   |  | [Empty state]    ||
|  |                  |  | ETC: 2m 34s      |  |                  ||
|  | Dice: 0.87       |  |                  |  |                  ||
|  | Loss: 1.2e-4     |  | Loss: 3.4e-3     |  |                  ||
|  |                  |  |                  |  |                  ||
|  | "My first run"   |  | "Testing params" |  | "Draft"          ||
|  | 2026-03-14 09:30 |  | 2026-03-14 10:15 |  | 2026-03-14 10:45 ||
|  +------------------+  +------------------+  +------------------+|
|                                                                  |
|  +------------------+  +------------------+                      |
|  | ...              |  | ...              |                      |
|  +------------------+  +------------------+                      |
|                                                                  |
+------------------------------------------------------------------+
```

## Toolbar

A horizontal bar above the card grid:

- **Filter dropdown**: Status (All / Planning / Queued / Running / Done / Failed)
- **Sort dropdown**: Date (newest first), Loss (lowest first), Dice (highest first), Name
- **Search**: Text input that filters by scenario ID and comment
- **View toggle**: Grid / List (optional -- grid is default, list is compact fallback)
- **+ New Scenario** button: prominent, right-aligned, `orange` accent

## Scenario Card

Each card is a clickable surface. Clicking navigates to the Scenario view for that scenario.

### Card Dimensions
- Min-width: 280px, max-width: 360px
- Cards fill available width in a responsive grid (CSS grid / flex-wrap equivalent)
- Gap: 16px

### Card Structure (top to bottom)

1. **Status badge** (top-left corner)
   - Pill-shaped label
   - Colors: `green` Done, `yellow` Running, `blue` Queued, `grey1` Planning, `red` Failed
   - Text: status name

2. **Thumbnail area** (200px height)
   - **Done**: StatesMax heatmap thumbnail (generated on demand, cached)
   - **Running**: Large circular progress indicator with percentage, ETC text below
   - **Planning**: Muted placeholder icon (document/gear icon in `grey1`)
   - **Failed**: Error icon with brief error message
   - Background: `bg_dim`

3. **Metrics row** (only for Done scenarios)
   - Two key metrics displayed prominently:
     - Dice score (with label)
     - Final loss (scientific notation, with label)
   - Small text, `fg1` for values, `grey1` for labels

4. **Title / ID**
   - Truncated scenario ID or a user-given name
   - `fg0`, medium weight

5. **Comment** (if present)
   - 1-2 lines, `grey1`, truncated with ellipsis
   - Italic

6. **Timestamp**
   - Creation date, `grey1`, small text

### Card Hover State
- Background: `bg1` -> `bg2`
- Subtle elevation change (box-shadow or border brightening)
- Cursor: pointer

### Card Selected State
- Border: 2px `orange`
- This card's scenario is the "active" scenario for Scenario/Results/Volumetric views

### Context Menu (right-click or long-press)
- Copy scenario
- Delete scenario
- Schedule / Unschedule
- Open in Results (if done)
- Open in Volumetric (if done)

## "New Scenario" Card

A special card at the start or end of the grid:
- Dashed border, `bg1` background
- Large "+" icon in center
- Text: "New Scenario"
- On click: creates a new scenario with defaults, navigates to Scenario view

## Empty State

When the project has no scenarios:
- Centered message: "No scenarios yet"
- Subtitle: "Create your first scenario to get started"
- Prominent "New Scenario" button below
- Optional: link to documentation

## Thumbnail Generation

Thumbnails are generated on demand (like current image generation):
- When Explorer view loads, queue thumbnail generation for all `Done` scenarios that don't have one cached
- Generate at reduced resolution (e.g., 280x200) for fast loading
- Store alongside scenario data
- Show a small spinner in the thumbnail area while generating
- For WASM: generate in-memory, cache in a resource

## Responsive Behavior

| Width | Columns |
|-------|---------|
| >= 1400px | 4 cards |
| 1000-1399px | 3 cards |
| 700-999px | 2 cards |
| < 700px | 1 card (full width) |
