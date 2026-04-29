## Context

The current Results view (`src/ui/results.rs`) is implemented in egui — a `CentralPanel` with a `ComboBox` that picks one of 33 `ImageType` variants. It is the only part of the app still using egui; every other major view (Explorer, Home, Scenario) is built in the Bevy-native UI shell (`src/ui/bevy_shell/`). The Bevy shell provides the sidebar, breadcrumb bar, and content slot — Results just needs to spawn its view tree into `ContentSlot` on `OnEnter(UiState::Results)`, exactly as Explorer and Scenario do.

The shell already demonstrates every UI primitive needed:
- Tab bars with `Display::Flex/None` toggling — `scenario/tabs.rs`
- Scrollable grid with `RepeatedGridTrack::flex(n, 1.0)` — `explorer/mod.rs`
- `ImageNode::new(handle)` for texture display — `explorer/card/spawn.rs`
- PNG → `Handle<Image>` via `image` crate + `Assets<Image>::add` — `explorer/thumbnail.rs`
- Modal overlays at `ZIndex(200)` with semi-transparent backdrop — `scenario/header/actions.rs`
- Async generation + channel polling — `explorer/thumbnail.rs` (`InFlight` + `poll_thumbnail_tasks`)
- Disabled buttons via `Disabled` marker — `scenario/mod.rs`

The `image` crate (with `png` feature) is already in `Cargo.toml`. The `gif` crate can be removed. No new dependencies are required for any part of this change. `ffmpeg` is an optional runtime tool, not a Rust dependency.

## Goals / Non-Goals

**Goals:**
- Bevy-native Results view that replaces egui completely for this view
- 4-tab categorized gallery (Spatial Maps, Metrics, Losses, Time Functions)
- Per-card lazy generation, 4 states: not-generated / generating / done / failed
- Animation types as first-class gallery cards — no separate action-bar generate buttons
- PNG frame sequences as the internal animation storage format — no GIF container
- Real frame-by-frame animation playback using a Bevy timer system: play/pause, speed slider, frame scrubber (editable when paused, hidden when playing)
- Full-size modal overlay for static images (prev/next navigation, keyboard shortcuts)
- Full-size modal overlay for animations with the same playback controls
- Batch generation: "Generate All in Tab", "Generate All", live `N/M` progress counter
- Animation export: APNG (zero new dependencies) and MP4 via ffmpeg subprocess (graceful error if absent)
- Action bar: Export .npy, global playback speed, Export Animation (APNG / MP4)
- Responsive grid: 3 / 2 / 1 columns via `update_grid_columns` system watching `ComputedNode`

**Non-Goals:**
- Comparison mode (side-by-side) — deferred
- Keeping any egui code in `src/ui/results.rs`
- New image types beyond the existing 33 static + 2 animation types
- Keeping the `gif` crate — it is removed as part of this change

## Decisions

### D1: ResultsViewPlugin following the established OnEnter/OnExit pattern

**Chosen**: Create `ResultsViewPlugin` in `src/ui/results/mod.rs`. Register `OnEnter(UiState::Results) → spawn_results_view` and `OnExit(UiState::Results) → despawn_results_view`. The view root spawns into `ContentSlot` (the same anchor used by `ExplorerViewRoot` and `ScenarioViewRoot`).

**Alternative**: Keep a single flat `src/ui/results.rs` and bolt the Bevy systems onto it. Rejected — the existing file is tightly coupled to egui. A clean plugin boundary is better for this scope of rewrite.

**Rationale**: Identical structure to `ExplorerViewPlugin` and `ScenarioViewPlugin`; the plugin registers all ECS resources, systems, and entity lifecycle.

---

### D2: ResultImageCache resource mirroring ThumbnailCache

**Chosen**: A `ResultImageCache` resource (`HashMap<ImageType, ResultImageState>`) where `ResultImageState` is an enum:
```
Pending              — registered, not yet started
Generating           — JoinHandle<Option<PathBuf>> in flight
Ready(Handle<Image>) — GPU texture, displayable
Failed(String)       — error message
```
Animation types use a parallel `ResultAnimCache` resource (`HashMap<AnimType, AnimState>`) where `AnimState` holds the loaded frame handles:
```
Pending
Generating  — JoinHandle<Option<PathBuf>> in flight (writes frame PNGs to disk)
Loading     — Vec<JoinHandle<Option<(Vec<u8>, u32, u32)>>> for per-frame PNG loads in progress
Ready       — { frames: Vec<Handle<Image>>, current_frame: usize, playing: bool, timer: Timer }
Failed(String)
```

**Alternative**: Re-use and extend `ResultImages: HashMap<ImageType, ImageBundle>`. Rejected — `ImageBundle` mixes path strings with `JoinHandle` in a way that doesn't compose cleanly with the Bevy asset handle system.

**Rationale**: The `ThumbnailCache` pattern is proven; this is a direct extension of it.

---

### D3: PNG loading via `image` crate → `Assets<Image>::add` (not AssetServer)

**Chosen**: Load static image PNGs off the main thread via `thread::spawn`, decode with `image::open()`, send pixels + dimensions back through an `mpsc::channel`, then on the main thread call `images.add(Image::new(...))` to get a `Handle<Image>`. Mirrors `thumbnail.rs` line 220–232 exactly.

**Alternative**: `AssetServer::load("results/{id}/img/{type}.png")`. Requires registering a custom `AssetSource` for paths outside the `assets/` folder. More infrastructure for no benefit; the channel approach is already proven in this codebase.

**Rationale**: Zero new infrastructure; consistent with the thumbnail pattern already reviewed and accepted.

---

### D4: PNG frame sequences replace GIF as the internal animation format

**Chosen**: Animation generation writes individual numbered PNG files to `results/{id}/img/anim/{anim_type}/frame_0001.png`, `frame_0002.png`, etc. The existing `states_spherical_plot_over_time()` function already generates frames one at a time — it currently encodes them into a GIF container. Instead, each rendered frame is saved directly as a PNG using `image::save_buffer_with_format(..., ImageFormat::Png)`, the same call used for static images. For playback, each frame PNG is loaded independently into `Assets<Image>` (same D3 pattern), producing a `Vec<Handle<Image>>`.

**Alternative**: Keep GIF as the container, decode frames at playback time. Rejected — GIF has a 256-colour palette limit (lossily degrades the plotters RGB output), LZW compression is worse than PNG deflate (larger files), and decoding requires the entire file to be read sequentially. A 200-frame animation at ~860×930 as GIF is typically 50–80 MB; as individual PNGs, the same content is typically 5–15 MB total (deflate compresses the flat-colour heatmap tiles extremely well).

**Alternative**: APNG as the on-disk storage format. Rejected for storage — APNG requires encoding the full sequence upfront, and re-encoding on playback start adds latency. PNG sequences let generation proceed frame-by-frame and load frames independently.

**Rationale**: PNG sequences are smaller, lossless, use existing code paths for both generation and loading, support instant random frame access, and remove the `gif` dependency entirely. The trade-off is a directory of files instead of one file — acceptable given `results/` is already a directory tree.

---

### D5: Animation playback driven by a Bevy `Timer` in `ResultAnimCache`

**Chosen**: Each `AnimState::Ready` holds a `bevy::time::Timer` configured with `TimerMode::Repeating` and a duration derived from `1.0 / playback_speed`. A `tick_animation_playback` system runs every `Update`, calls `timer.tick(time.delta())`, and on `timer.just_finished()` advances `current_frame = (current_frame + 1) % frames.len()`. The `ImageNode` on the thumbnail/modal is updated to `frames[current_frame]` each tick.

**Alternative**: Drive frame advancement from a fixed-step schedule. Rejected — playback speed needs to be user-adjustable at runtime; `Timer::set_duration()` handles this cleanly without schedule manipulation.

**Rationale**: `bevy::time::Timer` is idiomatic Bevy; `set_duration` lets the speed slider update propagate instantly.

---

### D6: Animation thumbnail card updates ImageNode handle directly via query

**Chosen**: Each animation thumbnail card has an `AnimCard { anim_type: AnimType }` marker component and a child `ImageNode` entity. The `tick_animation_playback` system queries `(AnimCard, &Children)`, finds the `ImageNode` child, and sets `image_node.image = frames[current_frame]`. Same approach in the modal (`AnimModalImage` marker).

**Alternative**: Despawn and respawn the `ImageNode` each frame. Rejected — O(entities) churn per frame is wasteful when a handle swap suffices.

**Rationale**: `Handle<Image>` is `Copy`; swapping it on an `ImageNode` is a cheap mutation with no GPU re-upload (the handle is replaced, not the asset data).

---

### D7: Frame scrubber shown only when paused; hidden during playback

**Chosen**: The frame scrubber row (current frame number input + `−` / `+` buttons) is a child node with `Display::None` when `playing == true` and `Display::Flex` when `playing == false`. When paused, the number is editable via keyboard (same `TextInputWidget` pattern from `scenario/widgets/text_input.rs`) or clickable `+`/`−` buttons.

**Alternative**: Always show a disabled scrubber during playback. Rejected — hidden during playback per spec; showing it clutters the card.

**Rationale**: `Display::None/Flex` toggling is the established show/hide pattern in this codebase (tabs, sections, empty state).

---

### D8: Modal overlay using PositionType::Absolute + ZIndex(200)

**Chosen**: When a card is clicked, set `ResultsViewState.modal = Some(ModalTarget)` and a `show_modal` / `update_modal` system spawns (or un-hides) the modal entity tree: a full-viewport backdrop at `ZIndex(200)`, semi-transparent `Color::srgba(0.0, 0.0, 0.0, 0.7)`, with a centered content card (`BG1`, `BorderRadius::all(8px)`, flexible height). Keyboard events (Left/Right/Esc) handled via `Res<ButtonInput<KeyCode>>` in the modal update system, gated on `modal.is_some()`.

**Alternative**: Navigate to a new `UiState` (e.g., `UiState::ResultsModal`). Rejected — ephemeral overlay state is not navigation state.

**Rationale**: Identical to `spawn_delete_modal` in `scenario/header/actions.rs` lines 351–459 — proven pattern, copy-paste.

---

### D9: APNG export via `image` crate, MP4 export via ffmpeg subprocess

**Chosen**: Two export paths, both triggered from the action bar when an animation card is selected:

**APNG**: Use `image::codecs::png::PngEncoder` with `image::codecs::png::ApngFrame` support (available in `image` 0.25). Assemble frames from the already-loaded `Vec<Handle<Image>>` pixel data (retrieved via `Assets<Image>::get`) and write to `results/{id}/export/{anim_type}.apng`. Runs on a background thread; no new dependency.

**MP4**: Shell out to `ffmpeg -framerate {fps} -i frame_%04d.png -c:v libx264 -pix_fmt yuv420p {output}.mp4` pointing at the frame sequence directory. Before spawning, check `which ffmpeg` (or `where ffmpeg` on Windows); if not found, surface a clear error message in the UI ("ffmpeg not found — install ffmpeg to export MP4"). Runs on a background thread via `std::process::Command`.

**Alternative**: Pure-Rust H.264 encoder (`openh264`). Rejected — adds a C dependency, increases binary size by ~20 MB, and `ffmpeg` is nearly universal on research machines.

**Alternative**: Export only APNG, no MP4. Rejected — MP4 is what users actually need for presentations (PowerPoint, Keynote); APNG support in presentation software is still inconsistent on Windows.

**Rationale**: APNG covers the zero-dependency case; MP4 covers the presentation use case. The graceful-degradation pattern (check first, show clear error) makes the ffmpeg dependency opt-in without impacting users who don't have it.

## Risks / Trade-offs

- **[Risk] Large frame sequences on disk** — 200 PNG frames at ~50–150 KB each = 10–30 MB per animation sequence. Much better than GIF (50–80 MB), but still non-trivial. → Accepted; this is an order-of-magnitude improvement. Users can delete `results/` freely.

- **[Risk] Large in-memory frame buffers** — 200 frames × 860×930 × 4 bytes RGBA8 ≈ 640 MB peak. Each `Handle<Image>` also occupies GPU VRAM. → Mitigation: load frames in `Loading` state batched (10 per frame) to spread the stall; cap at a configurable max (default 500 frames). Typical runs are 20–50 frames.

- **[Risk] Frame handle swap causes GPU re-upload** — swapping `image_node.image` (a `Handle<Image>`) does not mutate the `Image` asset; no re-upload occurs. Only modifying via `Assets<Image>::get_mut` would trigger re-upload. Confirmed safe.

- **[Risk] `thread::spawn` flood on "Generate All"** — Mitigation: skip cards already in `Generating` or `Loading` state.

- **[Risk] ffmpeg not found** — Mitigated by the pre-check and explicit error UI. APNG is always available as a fallback.

- **[Risk] APNG compatibility in presentation software** — PowerPoint for Windows added APNG support in 2023; older versions will show only the first frame. → Documented in the UI tooltip for the APNG button. MP4 is the safer choice for presentations.

## Migration Plan

1. `ResultsViewPlugin` is registered in `src/ui/bevy_shell/mod.rs` alongside `ExplorerViewPlugin` and `ScenarioViewPlugin`.
2. The existing egui `draw_ui_results` system is removed from `UiPlugin::build()`.
3. `ResultImages`, `SelectedResultImage`, `PlaybackSpeed` Bevy resources are removed; replaced by `ResultImageCache`, `ResultAnimCache`, `ResultsViewState`.
4. `src/ui/results/generate.rs` is extended: `generate_gifs()` replaced by `generate_animation()` which writes PNG frame sequences; `matrix_over_slices_plot()` and `voxel_types_over_slices_plot()` wired in.
5. `src/vis/plotting/gif/` — GIF container-writing code removed; frame rendering functions (`states_spherical_plot_frame()`, etc.) retained and called by the new PNG sequence generator.
6. `Cargo.toml` — `gif = "0.13.3"` removed.
7. No serialized state changes; no scenario data format changes; no test data invalidation.

## Open Questions

- Animation card placement: inline at the end of Spatial Maps tab, or a dedicated 5th "Animations" tab? Current plan: inline in Spatial Maps. Easy to move.
- Max frame cap: 500 is a conservative default. Hardcoded for now; configurable later if needed.
- Batch generation concurrency: start unlimited; add `num_cpus`-bounded semaphore if profiling shows thrash.
