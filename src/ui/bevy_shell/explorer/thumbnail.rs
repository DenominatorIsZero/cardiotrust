//! Thumbnail cache resource and async generation for Done scenarios.
//!
//! `ThumbnailCache` maps scenario ID → `ThumbnailState`. Generation is
//! dispatched asynchronously (one task per frame max); results are polled
//! back into the cache each frame.

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use bevy::prelude::*;

use crate::{core::scenario::Status, ScenarioList};

// ── Types ──────────────────────────────────────────────────────────────────────

/// State of a thumbnail for one scenario.
#[derive(Debug, Clone)]
pub enum ThumbnailState {
    /// Generation has been requested but not yet started.
    Pending,
    /// Generation task is in flight.
    Generating,
    /// Thumbnail image is ready.
    Ready(Handle<Image>),
    /// Generation failed (error message).
    Failed(String),
}

/// Shared channel value written by the async generation task.
type ThumbnailResult = Result<Handle<Image>, String>;

/// In-flight generation task for one scenario.
struct InFlight {
    id: String,
    result: Arc<Mutex<Option<ThumbnailResult>>>,
}

/// Bevy resource that holds per-scenario thumbnail state and in-flight tasks.
#[derive(Resource, Default)]
pub struct ThumbnailCache {
    pub states: HashMap<String, ThumbnailState>,
    in_flight: Vec<InFlight>,
}

impl std::fmt::Debug for ThumbnailCache {
    #[tracing::instrument(skip_all)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ThumbnailCache")
            .field("states_count", &self.states.len())
            .field("in_flight_count", &self.in_flight.len())
            .finish()
    }
}

// ── Systems ────────────────────────────────────────────────────────────────────

/// Iterates Done scenarios that don't yet have a `Ready` (or `Generating`)
/// entry and queues async thumbnail generation (one per frame max).
#[tracing::instrument(skip_all)]
pub fn queue_thumbnail_generation(
    mut cache: ResMut<ThumbnailCache>,
    scenario_list: Res<ScenarioList>,
    mut images: ResMut<Assets<Image>>,
) {
    // Read phase: determine what work is needed WITHOUT touching the ResMut.
    // `bypass_change_detection` gives immutable access that does not mark the
    // resource changed, preventing a per-frame rebuild of the card grid.
    let cache_read = cache.bypass_change_detection();

    let mut needs_pending: Vec<String> = Vec::new();
    let mut needs_generation: Option<String> = None;

    for entry in &scenario_list.entries {
        let scenario = &entry.scenario;
        if scenario.get_status() != &Status::Done {
            continue;
        }
        let id = scenario.get_id().clone();

        match cache_read.states.get(&id) {
            Some(
                ThumbnailState::Generating | ThumbnailState::Ready(_) | ThumbnailState::Failed(_),
            ) => continue,
            Some(ThumbnailState::Pending) | None => {}
        }

        if needs_generation.is_some() {
            needs_pending.push(id);
        } else {
            needs_generation = Some(id);
        }
    }

    // Nothing to do — return without ever writing to cache, so change-detection
    // stays clean and sync_cards_to_scenarios is not triggered every frame.
    if needs_generation.is_none() && needs_pending.is_empty() {
        return;
    }

    // Write phase: we have real work, mutations here correctly mark changed.
    for id in needs_pending {
        cache.states.insert(id, ThumbnailState::Pending);
    }
    let Some(id) = needs_generation else { return };

    cache.states.insert(id.clone(), ThumbnailState::Generating);

    // Generate a simple chart-like placeholder image (280×160).
    let width = 280_u32;
    let height = 160_u32;
    {

        // Derive a deterministic accent colour from the scenario ID.
        let hash: u64 = id.bytes().fold(0u64, |acc, b| {
            acc.wrapping_mul(31).wrapping_add(u64::from(b))
        });
        // Keep accent bright: clamp each channel to [100, 220].
        let accent_r = 100u8 + ((hash >> 16) & 0x77) as u8;
        let accent_g = 100u8 + ((hash >> 8) & 0x77) as u8;
        let accent_b = 100u8 + (hash & 0x77) as u8;

        // Colour constants (Gruvebox-dark-ish).
        let bg = [0x28u8, 0x28, 0x28, 0xFF];       // BG0
        let axis = [0x66u8, 0x5c, 0x54, 0xFF];     // GREY
        let grid_c = [0x3cu8, 0x38, 0x36, 0xFF];  // BG1

        // Margins (pixels): leave room for axes.
        let left = 24_u32;
        let bottom_px = 16_u32;
        let top_px = 12_u32;
        let right_px = 12_u32;
        let plot_w = width - left - right_px;
        let plot_h = height - bottom_px - top_px;

        let mut pixels = vec![0u8; (width * height * 4) as usize];

        let set_pixel = |pixels: &mut Vec<u8>, x: u32, y: u32, col: [u8; 4]| {
            if x < width && y < height {
                let idx = ((y * width + x) * 4) as usize;
                pixels[idx] = col[0];
                pixels[idx + 1] = col[1];
                pixels[idx + 2] = col[2];
                pixels[idx + 3] = col[3];
            }
        };

        // Fill background.
        for i in 0..(width * height) {
            let idx = (i * 4) as usize;
            pixels[idx] = bg[0];
            pixels[idx + 1] = bg[1];
            pixels[idx + 2] = bg[2];
            pixels[idx + 3] = bg[3];
        }

        // Draw horizontal grid lines (4 lines).
        for step in 0..=4u32 {
            let y = top_px + (step * plot_h) / 4;
            for x in left..(width - right_px) {
                set_pixel(&mut pixels, x, y, grid_c);
            }
        }

        // Draw vertical grid lines (6 lines).
        for step in 0..=6u32 {
            let x = left + (step * plot_w) / 6;
            for y in top_px..(height - bottom_px) {
                set_pixel(&mut pixels, x, y, grid_c);
            }
        }

        // Draw axes (left and bottom).
        for y in top_px..=(height - bottom_px) {
            set_pixel(&mut pixels, left, y, axis);
        }
        for x in left..=(width - right_px) {
            set_pixel(&mut pixels, x, height - bottom_px, axis);
        }

        // Draw a synthetic decreasing "loss" curve using the accent colour.
        // Uses a simple exponential decay: y_frac = exp(-5 * t) where t in [0,1].
        let n_points = plot_w as usize;
        let mut prev: Option<(u32, u32)> = None;
        #[allow(
            clippy::cast_precision_loss,
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss
        )]
        for i in 0..n_points {
            let t = i as f32 / (n_points - 1) as f32;
            // Decay with some hash-seeded noise for visual variety.
            let noise = (((hash.wrapping_mul(i as u64 + 1)) >> 8) & 0xF) as f32 / 256.0;
            let y_frac = (-4.0 * t).exp() + noise * (1.0 - t);
            let y_frac = y_frac.clamp(0.0, 1.0);
            // Map: 0.0 → bottom of plot, 1.0 → top. Casts are intentional pixel math.
            let py = (height - bottom_px - 1).saturating_sub((y_frac * (plot_h - 2) as f32) as u32);
            #[allow(clippy::cast_possible_truncation)]
            let px = left + i as u32;

            // Draw thick line (3 px vertical segment between prev and cur).
            if let Some((ppx, ppy)) = prev {
                let min_y = py.min(ppy);
                let max_y = py.max(ppy);
                for y in min_y..=max_y {
                    for dx in 0..2u32 {
                        set_pixel(&mut pixels, ppx + dx, y, [accent_r, accent_g, accent_b, 255]);
                    }
                }
            }
            set_pixel(&mut pixels, px, py, [accent_r, accent_g, accent_b, 255]);
            prev = Some((px, py));
        }

        let image = Image::new(
            bevy::render::render_resource::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            bevy::render::render_resource::TextureDimension::D2,
            pixels,
            bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
            bevy::render::render_asset::RenderAssetUsages::RENDER_WORLD,
        );
        let handle = images.add(image);
        cache.states.insert(id, ThumbnailState::Ready(handle));
    }
}

/// Polls completed async tasks and transitions their state to `Ready` or `Failed`.
#[tracing::instrument(skip_all)]
pub fn poll_thumbnail_tasks(mut cache: ResMut<ThumbnailCache>) {
    // Read phase without marking changed.
    let cache_read = cache.bypass_change_detection();
    if cache_read.in_flight.is_empty() {
        return;
    }

    let mut completed = Vec::new();
    for (i, task) in cache_read.in_flight.iter().enumerate() {
        if let Ok(mut guard) = task.result.try_lock() {
            if let Some(result) = guard.take() {
                completed.push((i, task.id.clone(), result));
            }
        }
    }

    if completed.is_empty() {
        return;
    }

    // Write phase — only reached when there are actual completions.
    for (i, id, result) in completed.into_iter().rev() {
        cache.in_flight.swap_remove(i);
        let state = match result {
            Ok(handle) => ThumbnailState::Ready(handle),
            Err(msg) => ThumbnailState::Failed(msg),
        };
        cache.states.insert(id, state);
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Verifies the `ThumbnailState` state-machine transitions without
    /// requiring any Bevy asset server.
    #[test]
    fn thumbnail_state_transitions() {
        let mut cache = ThumbnailCache::default();

        // Initially no entry.
        assert!(!cache.states.contains_key("s1"));

        // Insert Pending.
        cache.states.insert("s1".to_string(), ThumbnailState::Pending);
        assert!(matches!(
            cache.states.get("s1"),
            Some(ThumbnailState::Pending)
        ));

        // Transition to Generating.
        cache
            .states
            .insert("s1".to_string(), ThumbnailState::Generating);
        assert!(matches!(
            cache.states.get("s1"),
            Some(ThumbnailState::Generating)
        ));

        // Transition to Failed.
        cache.states.insert(
            "s1".to_string(),
            ThumbnailState::Failed("test error".to_string()),
        );
        assert!(matches!(
            cache.states.get("s1"),
            Some(ThumbnailState::Failed(_))
        ));

        // poll_thumbnail_tasks with no in_flight tasks is a no-op.
        // (We can't easily test the async path without a Bevy App.)
    }
}
