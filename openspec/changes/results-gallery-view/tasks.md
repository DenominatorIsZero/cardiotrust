## 1. Plugin Scaffold & ECS Resources

- [ ] 1.1 Create `src/ui/results/mod.rs` declaring `ResultsViewPlugin`; confirm `generate.rs` is at `src/ui/results/generate.rs`
- [ ] 1.2 Register `OnEnter(UiState::Results) → spawn_results_view` and `OnExit(UiState::Results) → despawn_results_view` in `ResultsViewPlugin::build()`
- [ ] 1.3 Define `ResultImageState` enum: `Pending`, `Generating { handle: JoinHandle<Option<PathBuf>> }`, `Loading { handle: JoinHandle<Option<(Vec<u8>, u32, u32)>> }`, `Ready(Handle<Image>)`, `Failed(String)`
- [ ] 1.4 Define `ResultImageCache` resource: `HashMap<ImageType, ResultImageState>`; register with `init_resource`
- [ ] 1.5 Define `AnimPlaybackState` struct: `frames: Vec<Handle<Image>>`, `current_frame: usize`, `playing: bool`, `timer: Timer`
- [ ] 1.6 Define `AnimState` enum: `Pending`, `Generating { handle: JoinHandle<Option<PathBuf>> }`, `Loading { handles: Vec<JoinHandle<Option<(Vec<u8>, u32, u32)>>>, loaded: Vec<Option<Handle<Image>>> }`, `Ready(AnimPlaybackState)`, `Failed(String)`
- [ ] 1.7 Define `ResultAnimCache` resource: `HashMap<AnimType, AnimState>`; register with `init_resource`
- [ ] 1.8 Define `ResultsViewState` resource: `active_tab: GalleryTab`, `modal: Option<ModalTarget>`, `batch_total: usize`, `batch_done: usize`, `playback_speed: f32`
- [ ] 1.9 Define `GalleryTab` enum (`SpatialMaps`, `Metrics`, `Losses`, `TimeFunctions`) with `Default = SpatialMaps`
- [ ] 1.10 Define `ModalTarget` enum: `StaticImage(ImageType)`, `Animation(AnimType)`
- [ ] 1.11 Add reset system: on `SelectedSenario` change, clear `ResultImageCache`, `ResultAnimCache`, reset `ResultsViewState.modal` and batch counters (keep `active_tab` and `playback_speed`)
- [ ] 1.12 Remove old resources from `UiPlugin::build()`: `ResultImages`, `SelectedResultImage`, `PlaybackSpeed`

## 2. Animation Generation — PNG Frame Sequences

- [ ] 2.1 Replace `generate_gifs()` in `src/ui/results/generate.rs` with `generate_animation(scenario, anim_type, playback_speed) -> JoinHandle<Option<PathBuf>>` which writes frames to `results/{id}/img/anim/{anim_type}/frame_{n:04}.png` using `image::save_buffer_with_format(..., ImageFormat::Png)`
- [ ] 2.2 Refactor `states_spherical_plot_over_time()` in `src/vis/plotting/gif/states.rs` to yield individual frame `PngBundle`s rather than encoding into a GIF container; call `generate_animation()` to write them
- [ ] 2.3 Wire `matrix_over_slices_plot()` (`src/vis/plotting/gif/matrix.rs`) and `voxel_types_over_slices_plot()` (`src/vis/plotting/gif/voxel_type.rs`) into the animation dispatch in `generate.rs`
- [ ] 2.4 Remove all GIF container-writing code (`gif::Encoder`, `gif::Frame::from_rgb`, `gif::Repeat`) from `src/vis/plotting/gif/`; keep frame rendering functions
- [ ] 2.5 Remove `gif = "0.13.3"` from `Cargo.toml`
- [ ] 2.6 Add `detect_existing_frames(scenario, anim_type) -> Option<Vec<PathBuf>>`: reads the frame directory on disk and returns sorted frame paths if they exist, enabling loading without re-generating

## 3. Static Image Loading

- [ ] 3.1 Add `poll_image_generation` system: check `ResultImageState::Generating` handles; on `Some(path)` spawn a background thread loading the PNG via `image::open(path)` and returning `(Vec<u8>, u32, u32)`; transition to `Loading`
- [ ] 3.2 Add `poll_image_loading` system: check `ResultImageState::Loading` handles; on completion insert into `Assets<Image>` and transition to `Ready(handle)`, or `Failed` on error

## 4. Animation Frame Loading

- [ ] 4.1 Add `poll_anim_generation` system: check `AnimState::Generating` handles; on `Some(frame_dir)` read sorted frame paths, spawn one `JoinHandle` per frame loading PNG pixels; transition to `AnimState::Loading { handles, loaded: vec![None; frame_count] }`
- [ ] 4.2 Add `poll_anim_loading` system: each frame poll all `Loading` handles; insert completed pixel buffers into `Assets<Image>` in batches of 10; once all `loaded` slots are `Some`, build `Vec<Handle<Image>>` and transition to `AnimState::Ready(AnimPlaybackState { frames, current_frame: 0, playing: true, timer })`
- [ ] 4.3 On scenario load, call `detect_existing_frames()` for each `AnimType`; if frames found, skip generation and go directly to `AnimState::Loading`

## 5. View Spawn — Gallery Shell

- [ ] 5.1 Implement `spawn_results_view`: spawn `ResultsViewRoot` into `ContentSlot`; `FlexDirection::Column`, fills available space
- [ ] 5.2 Spawn tab bar row (`BG1`, fixed height, `FlexDirection::Row`): four `TabButton { tab: GalleryTab }` + `TabAccent` + `TabLabel` following `scenario/tabs.rs` pattern
- [ ] 5.3 Spawn toolbar row: "Generate All in Tab" button, "Generate All" button, `BatchProgressLabel` text (hidden by default)
- [ ] 5.4 Spawn scrollable gallery area (`overflow: Overflow::scroll_y()`, `flex_grow: 1.0`): four `GalleryTabBody { tab }` children, only `SpatialMaps` shown initially; each contains a `GalleryGrid` (`Display::Grid`, `RepeatedGridTrack::flex(3, 1.0)`, 12px gaps)
- [ ] 5.5 Spawn action bar bottom row (`BG1`, fixed height): "Export to .npy" button + playback speed slider + "Export as APNG" button (`Disabled` by default) + "Export as MP4" button (`Disabled` by default) + `ExportStatusLabel` (hidden)
- [ ] 5.6 Implement `handle_tab_click`: update `ResultsViewState.active_tab`; show/hide `GalleryTabBody` nodes; update `TabAccent` and `TabLabel` colors
- [ ] 5.7 Implement `update_grid_columns` system: watch `ComputedNode` width on `GalleryGrid`; set `grid_template_columns` to `RepeatedGridTrack::flex(3/2/1, 1.0)` based on ≥1200 / 800–1199 / <800 px thresholds

## 6. Card Spawning

- [ ] 6.1 Define static ordered card descriptor lists per `GalleryTab`: `(CardKind, &str title, &str subtitle)` where `CardKind = Static(ImageType) | Anim(AnimType)`
- [ ] 6.2 Implement `spawn_gallery_cards`: for each descriptor spawn a card into the appropriate `GalleryGrid` with `StaticImageCard { image_type }` or `AnimCard { anim_type }` marker
- [ ] 6.3 Card outer node: `FlexDirection::Column`, `BG1` bg, `BorderRadius::all(Val::Px(6.0))`, `BorderColor::all(Color::NONE)`; title `Text` (`FG0`); subtitle `Text` (`GREY1`); `CardThumbnailArea` (`AspectRatio(4.0/3.0)`, `Overflow::clip()`)
- [ ] 6.4 Inside `CardThumbnailArea`: `GenerateButton` (visible, `BG_DIM` bg, centered "Generate" label); `SpinnerNode` (hidden); `FailedStateNode` with "Retry" button (hidden); `CardImageNode` with `ImageNode::default()` (hidden)
- [ ] 6.5 For `AnimCard` only: controls row below thumbnail — `PlayPauseButton`; `ScrubberRow` (`Display::None`) containing `DecrementButton`, `FrameNumberInput`, `IncrementButton`

## 7. Card State Sync

- [ ] 7.1 Implement `sync_static_card_state`: look up `ResultImageState` per `StaticImageCard`; show/hide `GenerateButton` / `SpinnerNode` / `CardImageNode` / `FailedStateNode`; set `ImageNode::image` when `Ready`
- [ ] 7.2 Implement `sync_anim_card_state`: look up `AnimState` per `AnimCard`; show/hide child nodes; when `Ready` set `CardImageNode.image` to `frames[current_frame]`; show/hide `ScrubberRow` based on `playing`
- [ ] 7.3 Implement `update_card_hover`: `BG2` / `GREY0` border on `Hovered`; `BG1` / transparent otherwise
- [ ] 7.4 Implement `handle_generate_button` (static): spawn generation thread, transition `ResultImageCache` entry to `Generating`
- [ ] 7.5 Implement `handle_generate_button` (anim): call `generate_animation()`, transition `ResultAnimCache` entry to `Generating`
- [ ] 7.6 Implement `handle_retry_button`: reset state to `Pending` and immediately re-trigger generation

## 8. Animation Playback

- [ ] 8.1 Implement `tick_animation_playback` system: for each `AnimState::Ready` where `playing == true`, tick timer; on `just_finished()` increment `current_frame = (current_frame + 1) % frames.len()`
- [ ] 8.2 After tick, update `CardImageNode.image` to `frames[current_frame]`; also update `AnimModalFrameNode` if the modal is open for this `AnimType`
- [ ] 8.3 Implement `handle_play_pause_button`: toggle `playing`; update button label; show/hide `ScrubberRow`
- [ ] 8.4 Implement `handle_frame_increment` / `handle_frame_decrement`: when paused, step `current_frame` with wraparound; update `FrameNumberInput` display
- [ ] 8.5 Implement `handle_frame_input_submit`: parse text as `usize`, clamp to `0..frames.len()-1`, set `current_frame`
- [ ] 8.6 Implement `update_playback_speed` system: when `ResultsViewState.playback_speed` changes, call `timer.set_duration(Duration::from_secs_f32(1.0 / speed))` for all `AnimState::Ready` entries

## 9. Batch Generation & Progress

- [ ] 9.1 Implement `handle_generate_all_in_tab`: trigger generation for all `Pending`/`Failed` cards in current tab; set `batch_total`, reset `batch_done`
- [ ] 9.2 Implement `handle_generate_all`: same across all tabs
- [ ] 9.3 Implement `update_batch_progress`: count newly-finished generation handles each frame; increment `batch_done`; show "Generating N/M..." in `BatchProgressLabel`; hide when `batch_done >= batch_total`

## 10. Static Image Modal

- [ ] 10.1 Implement `spawn_static_modal`: on `ModalTarget::StaticImage(_)` set, spawn backdrop (`PositionType::Absolute`, full viewport, `ZIndex(200)`, `Color::srgba(0,0,0,0.7)`) + centered `BG1` card
- [ ] 10.2 Modal header: title `Text`, `CloseModalButton` (X)
- [ ] 10.3 Modal content: `ModalImageNode` (`ImageNode`, `width: Percent(100)`, `height: Auto`)
- [ ] 10.4 Modal footer: `PrevButton`, position label ("N / M"), `NextButton`
- [ ] 10.5 Implement `update_static_modal`: update image handle, title, position label from `ResultsViewState.modal` + `ResultImageCache`
- [ ] 10.6 Implement `handle_modal_keyboard` (gated on `modal.is_some()`): `ArrowRight` → next done image (skip non-done, wrap); `ArrowLeft` → prev; `Escape` → close
- [ ] 10.7 Implement `handle_modal_close` on `CloseModalButton`: set `modal = None`, despawn modal tree
- [ ] 10.8 Implement `handle_static_card_click`: on `Ready` card press, set `modal = Some(ModalTarget::StaticImage(...))`

## 11. Animation Modal

- [ ] 11.1 Implement `spawn_anim_modal`: same backdrop/card structure as static modal; content has `AnimModalFrameNode` (`ImageNode`) + controls row (play/pause + `ScrubberRow` when paused — same widget structure as card controls)
- [ ] 11.2 Store independent `AnimPlaybackState` for the modal in `ResultsViewState` (separate from thumbnail's `AnimState::Ready`)
- [ ] 11.3 `tick_animation_playback` ticks both thumbnail and modal states; updates both `CardImageNode` and `AnimModalFrameNode`
- [ ] 11.4 Implement `handle_anim_card_click`: on `Ready` card press, set `modal = Some(ModalTarget::Animation(...))`, clone `AnimPlaybackState` into modal state

## 12. Animation Export

- [ ] 12.1 Implement `handle_export_apng`: on button press (only when an animation is `Ready`), spawn background thread; assemble frames from pixel data (retrieve `Image` assets via `Assets<Image>::get`), encode to APNG using `image::codecs::png::PngEncoder` with repeat/frame metadata; write to `results/{id}/export/{anim_type}.apng`
- [ ] 12.2 Track APNG export state in `ResultsViewState.export_state: Option<ExportState>` where `ExportState = InProgress(JoinHandle<Result<PathBuf>>) | Done(PathBuf) | Failed(String)`
- [ ] 12.3 Poll export handle each frame; on completion update `ExportStatusLabel` text ("Exported to results/.../foo.apng") or show error; clear after 5 seconds
- [ ] 12.4 Implement `handle_export_mp4`: on button press, first check for `ffmpeg` via `std::process::Command::new("ffmpeg").arg("-version").output()`; if not found, set `ExportState::Failed("ffmpeg not found — install ffmpeg to export MP4")` immediately; if found, spawn background thread running `ffmpeg -framerate {fps} -i frame_%04d.png -c:v libx264 -pix_fmt yuv420p {output}.mp4` pointed at the frame sequence directory
- [ ] 12.5 Implement `update_export_buttons` system: enable/disable "Export as APNG" and "Export as MP4" buttons based on whether any `AnimState::Ready` exists; disable both while an export is `InProgress`

## 13. Action Bar Wiring

- [ ] 13.1 Wire "Export to .npy" button to existing npy export logic
- [ ] 13.2 Wire playback speed slider to `ResultsViewState.playback_speed`; on change trigger `update_playback_speed` (task 8.6)

## 14. Cleanup

- [ ] 14.1 Remove egui `draw_ui_results` system from `UiPlugin::build()`; remove `use bevy_egui` from `src/ui/results.rs`
- [ ] 14.2 Remove old `ResultImages`, `SelectedResultImage`, `PlaybackSpeed` resource declarations and registrations
- [ ] 14.3 Remove `gif = "0.13.3"` from `Cargo.toml`; remove all `use gif::` imports
- [ ] 14.4 Ensure all new public functions have `#[tracing::instrument(skip_all)]`
- [ ] 14.5 Replace any `unwrap()` in new code with `?` or `.expect("...")` with descriptive messages
- [ ] 14.6 Run `just fmt` to normalize imports

## 15. Verification

- [ ] 15.1 Run `just check` — zero warnings, zero clippy errors
- [ ] 15.2 Run `just lint` — clippy-tracing span check passes for all new public functions
- [ ] 15.3 Run `just test` — all existing tests pass
- [ ] 15.4 Manual smoke test: navigate to Results; verify four tabs; click Generate on one card per tab; verify spinner → thumbnail
- [ ] 15.5 Manual smoke test: "Generate All in Tab" on Spatial Maps; verify progress counter; all cards done
- [ ] 15.6 Manual smoke test: click static image; verify modal, arrow-key navigation, Esc close
- [ ] 15.7 Manual smoke test: generate an Algorithm States animation; verify `results/{id}/img/anim/` directory contains numbered PNGs; verify card plays animation; pause, scrub with +/− and direct input; verify scrubber hidden on resume
- [ ] 15.8 Manual smoke test: click animation card; verify modal with independent playback state
- [ ] 15.9 Manual smoke test: "Export as APNG" — verify `.apng` file written to `results/{id}/export/`; open in browser to confirm animation plays
- [ ] 15.10 Manual smoke test: "Export as MP4" with ffmpeg present — verify `.mp4` written; without ffmpeg — verify clear error message shown
- [ ] 15.11 Manual smoke test: switch scenario; verify all cards reset; no stale textures in modal
