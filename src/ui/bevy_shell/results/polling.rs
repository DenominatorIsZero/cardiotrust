//! Polling systems: check background threads and advance image/anim states.

use std::path::PathBuf;

use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};

use super::{
    new_channel, AnimPlaybackState, AnimState, ResultAnimCache, ResultImageCache, ResultImageState,
    ResultsViewState,
};

// ── Static image polling ──────────────────────────────────────────────────────

/// Polls in-flight image generation tasks.
///
/// When a task finishes and produced a `PathBuf`, transitions state to
/// `Loading` and spawns the byte-loading thread.
#[tracing::instrument(skip_all)]
pub fn poll_image_generation(mut image_cache: ResMut<ResultImageCache>) {
    // Collect which image types need to transition.
    let to_load: Vec<_> = image_cache
        .0
        .iter()
        .filter_map(|(image_type, state)| {
            if let ResultImageState::Generating { channel } = state {
                if let Ok(mut guard) = channel.try_lock() {
                    if let Some(result) = guard.take() {
                        return Some((*image_type, result));
                    }
                }
            }
            None
        })
        .collect();

    for (image_type, result) in to_load {
        match result {
            Err(e) => {
                image_cache
                    .0
                    .insert(image_type, ResultImageState::Failed(e.to_string()));
            }
            Ok(path) => {
                // Spawn a thread to load PNG bytes from disk.
                let channel = new_channel::<(Vec<u8>, u32, u32)>();
                let channel_writer = channel.clone();
                std::thread::spawn(move || {
                    let result = load_image_bytes(&path);
                    if let Ok(mut guard) = channel_writer.lock() {
                        *guard = Some(result);
                    }
                });
                image_cache
                    .0
                    .insert(image_type, ResultImageState::Loading { channel });
            }
        }
    }
}

/// Polls in-flight image loading tasks.
///
/// When bytes arrive, uploads the image to the GPU via `Assets<Image>`.
#[tracing::instrument(skip_all)]
pub fn poll_image_loading(
    mut image_cache: ResMut<ResultImageCache>,
    mut images: ResMut<Assets<Image>>,
) {
    let to_ready: Vec<_> = image_cache
        .0
        .iter()
        .filter_map(|(image_type, state)| {
            if let ResultImageState::Loading { channel } = state {
                if let Ok(mut guard) = channel.try_lock() {
                    if let Some(result) = guard.take() {
                        return Some((*image_type, result));
                    }
                }
            }
            None
        })
        .collect();

    for (image_type, result) in to_ready {
        match result {
            Err(e) => {
                image_cache
                    .0
                    .insert(image_type, ResultImageState::Failed(e.to_string()));
            }
            Ok((rgba_bytes, width, height)) => {
                let bevy_image = Image::new(
                    Extent3d {
                        width,
                        height,
                        depth_or_array_layers: 1,
                    },
                    TextureDimension::D2,
                    rgba_bytes,
                    TextureFormat::Rgba8UnormSrgb,
                    RenderAssetUsages::RENDER_WORLD,
                );
                let handle = images.add(bevy_image);
                image_cache
                    .0
                    .insert(image_type, ResultImageState::Ready(handle));
            }
        }
    }
}

// ── Animation polling ─────────────────────────────────────────────────────────

/// Polls in-flight animation generation tasks.
#[tracing::instrument(skip_all)]
pub fn poll_anim_generation(
    mut anim_cache: ResMut<ResultAnimCache>,
    view_state: Res<ResultsViewState>,
) {
    let to_load: Vec<_> = anim_cache
        .0
        .iter()
        .filter_map(|(anim_type, state)| {
            if let AnimState::Generating { channel } = state {
                if let Ok(mut guard) = channel.try_lock() {
                    if let Some(result) = guard.take() {
                        return Some((*anim_type, result));
                    }
                }
            }
            None
        })
        .collect();

    for (anim_type, result) in to_load {
        match result {
            Err(e) => {
                anim_cache
                    .0
                    .insert(anim_type, AnimState::Failed(e.to_string()));
            }
            Ok(anim_dir) => {
                // Discover frame files in the directory.
                let frames: Vec<PathBuf> =
                    super::generate::detect_existing_frames(&anim_dir).unwrap_or_default();

                if frames.is_empty() {
                    anim_cache.0.insert(
                        anim_type,
                        AnimState::Failed("No frames found after generation".to_string()),
                    );
                    continue;
                }

                // Spawn loading threads for each frame.
                let fps = view_state.playback_speed.max(1.0);
                let channels: Vec<_> = frames
                    .iter()
                    .map(|path| {
                        let channel = new_channel::<(Vec<u8>, u32, u32)>();
                        let writer = channel.clone();
                        let p = path.clone();
                        std::thread::spawn(move || {
                            let result = load_image_bytes(&p);
                            if let Ok(mut guard) = writer.lock() {
                                *guard = Some(result);
                            }
                        });
                        channel
                    })
                    .collect();

                let loaded = vec![None; channels.len()];
                let _ = fps; // will use when creating timer
                anim_cache
                    .0
                    .insert(anim_type, AnimState::Loading { channels, loaded });
            }
        }
    }
}

/// Polls in-flight animation frame loading.
#[tracing::instrument(skip_all)]
pub fn poll_anim_loading(
    mut anim_cache: ResMut<ResultAnimCache>,
    mut images: ResMut<Assets<Image>>,
    view_state: Res<ResultsViewState>,
) {
    let fps = view_state.playback_speed.max(1.0);
    let timer_duration = std::time::Duration::from_secs_f32(1.0 / fps);

    let anim_types: Vec<_> = anim_cache.0.keys().copied().collect();

    for anim_type in anim_types {
        let state = anim_cache.0.get_mut(&anim_type);
        let Some(AnimState::Loading { channels, loaded }) = state else {
            continue;
        };

        for (i, channel) in channels.iter().enumerate() {
            if loaded[i].is_some() {
                continue;
            }
            if let Ok(mut guard) = channel.try_lock() {
                if let Some(result) = guard.take() {
                    match result {
                        Err(_e) => {
                            // Leave as None — will surface as incomplete
                        }
                        Ok((rgba_bytes, width, height)) => {
                            let bevy_image = Image::new(
                                Extent3d {
                                    width,
                                    height,
                                    depth_or_array_layers: 1,
                                },
                                TextureDimension::D2,
                                rgba_bytes,
                                TextureFormat::Rgba8UnormSrgb,
                                RenderAssetUsages::RENDER_WORLD,
                            );
                            let handle = images.add(bevy_image);
                            loaded[i] = Some(handle);
                        }
                    }
                }
            }
        }

        // Check if all frames are loaded.
        let all_loaded = loaded.iter().all(Option::is_some);
        if all_loaded {
            let frames: Vec<Handle<Image>> = loaded.iter().filter_map(Clone::clone).collect();
            let playback = AnimPlaybackState {
                frames,
                current_frame: 0,
                playing: false,
                timer: Timer::new(timer_duration, TimerMode::Repeating),
            };
            anim_cache.0.insert(anim_type, AnimState::Ready(playback));
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Loads a PNG or RGB image from `path` and returns `(rgba_bytes, width, height)`.
#[tracing::instrument(level = "trace", skip_all)]
fn load_image_bytes(path: &std::path::Path) -> anyhow::Result<(Vec<u8>, u32, u32)> {
    let img = image::open(path)
        .map_err(|e| anyhow::anyhow!("Failed to open image {}: {}", path.display(), e))?;
    let rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();
    Ok((rgba.into_raw(), w, h))
}
