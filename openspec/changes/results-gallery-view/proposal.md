## Why

The current Results view is a single-image viewer with a dropdown to select one of 33 image types — you have to generate each image individually, can only see one at a time, and GIF playback has no viewer at all. Researchers need to quickly scan all outputs after a run, spot anomalies across categories, and compare variants (Algorithm / Simulation / Delta) side-by-side.

The view is also the only part of the app still implemented in egui, which creates a visual inconsistency with the Bevy-native shell used everywhere else. This change removes egui from Results entirely and implements the gallery in the Bevy-native UI system.

## What Changes

- Remove egui from `src/ui/results.rs` entirely; implement the Results view as a Bevy-native view following the same `OnEnter`/`OnExit` spawn/despawn pattern as Explorer and Scenario
- Replace the `ComboBox + single image` layout with a **categorized thumbnail gallery** (4 tabs: Spatial Maps, Metrics, Losses, Time Functions)
- Replace auto-generation-on-view with **on-demand lazy generation** — each card shows a "Generate" button until triggered
- Add **full-size modal viewer** with prev/next navigation, implemented as a `PositionType::Absolute` + `ZIndex` overlay (same pattern as the existing delete-confirm modal in the Scenario view)
- Add **batch generation** controls ("Generate All in Tab", "Generate All") with a live progress counter
- Animation types become **first-class gallery cards** — no separate "Generate GIF" buttons in an action bar; animation cards have the same four states as static image cards (not-generated / generating / done / failed)
- **Replace GIF as the internal animation format with PNG frame sequences**: each animation is stored as a directory of numbered PNGs (`results/{id}/img/anim/{type}/frame_0001.png`, …). Individual frames are the same plotters-generated PNGs already used everywhere else — no palette quantization, no container overhead, instant random access
- Add **real frame-by-frame animation playback** in-app: frames are loaded as `Handle<Image>` in Bevy's asset system and stepped by a timer system. Controls: play/pause toggle, playback speed, and a frame scrubber (number display + increment/decrement when paused, hidden during playback)
- Add **animation export** for sharing: "Export as APNG" (Animated PNG — built into the `image` crate already in `Cargo.toml`, no new dependency, full color, ~3–5× smaller than equivalent GIF) and "Export as MP4" (via `ffmpeg` subprocess — best compression, checks for ffmpeg first and shows a clear error if not found)
- Expose `matrix_over_slices_plot()` and `voxel_types_over_slices_plot()` animation generators which currently exist but are not wired into any UI
- Action bar: **Export to .npy**, global **playback speed** setting, and **Export Animation** (APNG / MP4) when an animation card is selected

## Capabilities

### New Capabilities
- `results-gallery`: Bevy-native categorized thumbnail grid with per-card lazy generation, generation state tracking (not-generated / generating / done / failed), responsive 3/2/1-column CSS Grid layout, hover highlight, batch generation with progress counter, and a persistent action bar (Export .npy, playback speed, Export Animation)
- `results-image-viewer`: Full-size Bevy-native modal overlay (`PositionType::Absolute`, `ZIndex(200)`, semi-transparent backdrop) showing a single static image at full resolution with prev/next navigation (keyboard + buttons) and Esc/X to close
- `results-animation-viewer`: Bevy-native animation playback using PNG frame sequences — frames loaded from disk into a `Vec<Handle<Image>>`, advanced by a per-card timer system. Thumbnail card shows the current frame live. Controls: play/pause, playback speed slider, frame counter (hidden during playback; editable with +/− buttons when paused). Modal opens with same playback controls at full size. Export actions: APNG (via `image` crate, no new dependency) and MP4 (via `ffmpeg` subprocess, graceful error if not installed).

### Modified Capabilities

## Impact

- **`src/ui/results.rs`** — rewritten as a Bevy-native plugin (`ResultsViewPlugin`) with `OnEnter(UiState::Results)` / `OnExit` systems; all egui dependencies removed from this module
- **`src/ui/results/`** — new submodules: `card.rs`, `gallery.rs`, `modal.rs`, `animation_player.rs`, `export.rs`, `generate.rs` (existing, extended)
- **`src/ui/bevy_shell/mod.rs`** — register `ResultsViewPlugin`
- **`src/ui/results/generate.rs`** — animation generation writes PNG frame sequences to `results/{id}/img/anim/{type}/`; `generate_animation()` replaces `generate_gifs()`; wire `matrix_over_slices_plot()` and `voxel_types_over_slices_plot()` into dispatch
- **`src/vis/plotting/gif/`** — existing GIF container-writing code replaced; frame rendering functions retained and reused by the PNG sequence generator
- **`src/ui/colors.rs`** — no new colors; existing palette covers all states
- **`Cargo.toml`** — `gif` dependency can be removed; `image` crate (already present, `png` feature already enabled) gains APNG usage; no net new dependencies. `ffmpeg` is an optional runtime dependency (not in `Cargo.toml`)
