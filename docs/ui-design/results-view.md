# Results View -- Image Gallery

## Purpose

Browse and compare generated result images for a completed scenario. Replaces the single dropdown + single image viewer with a categorized thumbnail gallery.

## Layout

```
+------------------------------------------------------------------+
| Context bar: My Project > Scenario v8_Dr_2026-03-14 > Results    |
+------------------------------------------------------------------+
| [ Spatial Maps ] [ Metrics ] [ Losses ] [ Time Functions ]       |
+==================================================================+
|                                                                  |
|  +---------------+  +---------------+  +---------------+         |
|  | States Max    |  | States Max    |  | States Max    |         |
|  | (Algorithm)   |  | (Simulation)  |  | (Delta)       |         |
|  |               |  |               |  |               |         |
|  | [thumbnail]   |  | [thumbnail]   |  | [thumbnail]   |         |
|  |               |  |               |  |               |         |
|  +---------------+  +---------------+  +---------------+         |
|                                                                  |
|  +---------------+  +---------------+  +---------------+         |
|  | Activation T. |  | Activation T. |  | Activation T. |         |
|  | (Algorithm)   |  | (Simulation)  |  | (Delta)       |         |
|  |               |  |               |  |               |         |
|  | [  Generate  ]|  | [  Generate  ]|  | [  Generate  ]|         |
|  |               |  |               |  |               |         |
|  +---------------+  +---------------+  +---------------+         |
|                                                                  |
+------------------------------------------------------------------+
|  [ Export .npy ]    [ Gen. Algorithm GIF ]   [ Gen. Sim GIF ]    |
+------------------------------------------------------------------+
```

## Category Tabs

The 30+ image types are grouped into 4 categories:

### Spatial Maps
2D slice visualizations of the heart model.

| Image | Variants |
|-------|----------|
| States Max | Algorithm, Simulation, Delta |
| Activation Time | Algorithm, Simulation, Delta |
| Voxel Types | Algorithm, Simulation, Prediction |
| Average Delay | Simulation, Algorithm, Delta |
| Average Propagation Speed | Simulation, Algorithm |

### Metrics
Per-epoch metric plots.

| Image |
|-------|
| Dice |
| IoU |
| Recall |
| Precision |

### Losses
Training loss curves.

| Image |
|-------|
| Loss (full) |
| Loss (per epoch) |
| MSE Loss (full) |
| MSE Loss (per epoch) |
| Max Regularization (full) |
| Max Regularization (per epoch) |

### Time Functions
Signal waveforms over time.

| Image | Variants |
|-------|----------|
| Control Function | Algorithm, Simulation, Delta |
| State | Algorithm, Simulation, Delta |
| Measurement | Algorithm, Simulation, Delta |

## Thumbnail Cards

Each image type is represented as a card in the grid:

### Card structure
- **Title**: Image type name (e.g., "States Max"), `fg0`
- **Subtitle**: Variant in `grey1` (e.g., "Algorithm")
- **Thumbnail area** (fixed aspect ratio, e.g., 4:3):
  - If generated: the actual image scaled to fit
  - If not generated: "Generate" button centered on `bg_dim` background
  - If generating: spinner with "Generating..." text

### Card dimensions
- Width: fills grid columns (3 columns default)
- Thumbnail height: ~200px
- Gap: 12px

### Interaction
- **Click thumbnail** (if generated): opens full-size view (see below)
- **Click "Generate"**: triggers background image generation for that specific type
- **Hover**: subtle border highlight

## On-Demand Generation

Images are generated lazily to keep the UI responsive:

### Strategy
1. When the Results view loads, no images are generated automatically
2. Each card shows a "Generate" button
3. User clicks to generate specific images they want to see
4. A "Generate All" button in the toolbar generates everything in the tab

### Generation states per image
- **Not generated**: Shows "Generate" button
- **Generating**: Shows spinner, button disabled
- **Generated**: Shows thumbnail
- **Failed**: Shows error icon with retry button

### Batch generation
- "Generate All in Tab" button: generates all images in the current category tab
- "Generate All": generates every image type (useful before exporting)
- Generation runs in background threads (native) or via async tasks (WASM)
- Progress indicator in toolbar: "Generating 3/15..."

## Full-Size Image View

Clicking a generated thumbnail opens an expanded view:

```
+------------------------------------------------------------------+
| [<] States Max (Algorithm)                              [X]      |
+------------------------------------------------------------------+
|                                                                  |
|                                                                  |
|                    [Full resolution image]                        |
|                                                                  |
|                                                                  |
+------------------------------------------------------------------+
| [< Prev]              3 / 15                        [Next >]     |
+------------------------------------------------------------------+
```

### Behavior
- Image fills available space, maintaining aspect ratio
- Left/right arrow keys or buttons navigate between images in the same category
- `Esc` or `X` button closes and returns to gallery
- Title bar shows image type name
- This is a modal overlay on top of the gallery, not a separate view

## Action Bar

A bottom bar (or toolbar row below tabs):

- **Export to .npy** -- exports scenario data as NumPy arrays
- **Generate Algorithm GIF** -- creates animated GIF of algorithm results
- **Generate Simulation GIF** -- creates animated GIF of simulation results
- **Playback speed** slider (for GIF generation): 0.001 to 0.1

These are secondary actions, visually less prominent than the gallery.

## Responsive Behavior

| Width | Columns |
|-------|---------|
| >= 1200px | 3 thumbnails per row |
| 800-1199px | 2 per row |
| < 800px | 1 per row |

## Comparison Mode (Future Enhancement)

Not in the initial implementation, but worth considering:
- Select 2-3 images to compare side-by-side
- Useful for Algorithm vs. Simulation vs. Delta comparisons
- Could be a split-view toggle in the full-size viewer
