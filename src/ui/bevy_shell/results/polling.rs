//! Polling systems: check background threads and advance image/anim states.

use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};

use super::{
    AnimPlaybackState, AnimState, ResultAnimCache, ResultImageCache, ResultImageState,
    ResultsViewState,
};

// ── Static image polling ──────────────────────────────────────────────────────

/// Polls in-flight image generation tasks.
///
/// When a `PngBundle` arrives (RGB bytes), converts to RGBA and uploads
/// directly to the GPU. No separate file-load step.
#[tracing::instrument(skip_all)]
pub fn poll_image_generation(
    mut image_cache: ResMut<ResultImageCache>,
    mut images: ResMut<Assets<Image>>,
) {
    let to_upload: Vec<_> = image_cache
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

    for (image_type, result) in to_upload {
        match result {
            Err(e) => {
                image_cache
                    .0
                    .insert(image_type, ResultImageState::Failed(e.to_string()));
            }
            Ok(bundle) => {
                let rgba = rgb_to_rgba(&bundle.data);
                let handle = upload_image(&mut images, &rgba, bundle.width, bundle.height);
                image_cache
                    .0
                    .insert(image_type, ResultImageState::Ready(handle));
            }
        }
    }
}

/// Polls in-flight image-loading tasks (pre-existing cached images from disk).
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
                let handle = upload_image(&mut images, &rgba_bytes, width, height);
                image_cache
                    .0
                    .insert(image_type, ResultImageState::Ready(handle));
            }
        }
    }
}

// ── Animation polling ─────────────────────────────────────────────────────────

/// Polls in-flight animation generation tasks.
///
/// When a `GifBundle` arrives, each RGB frame is converted to RGBA and
/// uploaded to the GPU. Goes directly to `Ready` — no file-load step.
#[tracing::instrument(skip_all)]
pub fn poll_anim_generation(
    mut anim_cache: ResMut<ResultAnimCache>,
    mut images: ResMut<Assets<Image>>,
    view_state: Res<ResultsViewState>,
) {
    let to_upload: Vec<_> = anim_cache
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

    for (anim_type, result) in to_upload {
        match result {
            Err(e) => {
                anim_cache
                    .0
                    .insert(anim_type, AnimState::Failed(e.to_string()));
            }
            Ok(bundle) => {
                let fps = view_state.playback_speed.max(1.0);
                let timer_duration = std::time::Duration::from_secs_f32(1.0 / fps);

                let mut frames: Vec<Handle<Image>> = Vec::with_capacity(bundle.data.len());
                for rgb_bytes in &bundle.data {
                    let rgba = rgb_to_rgba(rgb_bytes);
                    let handle = upload_image(&mut images, &rgba, bundle.width, bundle.height);
                    frames.push(handle);
                }

                let playback = AnimPlaybackState {
                    frames,
                    current_frame: 0,
                    playing: false,
                    timer: Timer::new(timer_duration, TimerMode::Repeating),
                };
                anim_cache
                    .0
                    .insert(anim_type, AnimState::Ready(playback));
            }
        }
    }
}

/// Polls in-flight animation frame loading (pre-existing cached frames from disk).
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
                        Err(_e) => {}
                        Ok((rgba_bytes, width, height)) => {
                            let handle = upload_image(&mut images, &rgba_bytes, width, height);
                            loaded[i] = Some(handle);
                        }
                    }
                }
            }
        }

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

/// Converts packed RGB bytes (3 bytes/pixel) to RGBA (4 bytes/pixel).
fn rgb_to_rgba(rgb: &[u8]) -> Vec<u8> {
    let pixel_count = rgb.len() / 3;
    let mut rgba = Vec::with_capacity(pixel_count * 4);
    for i in 0..pixel_count {
        rgba.push(rgb[i * 3]);
        rgba.push(rgb[i * 3 + 1]);
        rgba.push(rgb[i * 3 + 2]);
        rgba.push(255);
    }
    rgba
}

/// Creates a Bevy `Image` from raw RGBA bytes and uploads to the GPU.
fn upload_image(
    images: &mut Assets<Image>,
    rgba: &[u8],
    width: u32,
    height: u32,
) -> Handle<Image> {
    let bevy_image = Image::new(
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        rgba.to_vec(),
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    );
    images.add(bevy_image)
}
