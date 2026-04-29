//! Animation playback systems for animation cards.

#![allow(clippy::type_complexity)]

use bevy::prelude::*;

use super::{
    card::{
        CardThumbnail, FrameDecrementButton, FrameIncrementButton, FrameNumberLabel,
        PlayPauseButton,
    },
    AnimState, ResultAnimCache, ResultsViewState,
};

// ── Tick ──────────────────────────────────────────────────────────────────────

/// Advances animation frames for all playing animations.
#[tracing::instrument(skip_all)]
pub fn tick_animation_playback(
    time: Res<Time>,
    mut anim_cache: ResMut<ResultAnimCache>,
    mut thumbnails: Query<(&CardThumbnail, &mut ImageNode)>,
    mut frame_labels: Query<(&FrameNumberLabel, &mut Text)>,
) {
    let mut any_changed = false;

    for state in anim_cache.0.values_mut() {
        let AnimState::Ready(playback) = state else {
            continue;
        };
        if !playback.playing || playback.frames.is_empty() {
            continue;
        }
        playback.timer.tick(time.delta());
        if playback.timer.just_finished() {
            playback.current_frame = (playback.current_frame + 1) % playback.frames.len();
            any_changed = true;
        }
    }

    if !any_changed {
        return;
    }

    // Update thumbnails
    for (thumb, mut img) in &mut thumbnails {
        let super::card::CardKind::Anim(anim_type) = thumb.kind else {
            continue;
        };
        if let Some(AnimState::Ready(playback)) = anim_cache.0.get(&anim_type) {
            if let Some(handle) = playback.frames.get(playback.current_frame) {
                img.image = handle.clone();
            }
        }
    }

    // Update frame labels
    for (label, mut text) in &mut frame_labels {
        if let Some(AnimState::Ready(playback)) = anim_cache.0.get(&label.anim_type) {
            text.0 = format!("{}/{}", playback.current_frame + 1, playback.frames.len());
        }
    }
}

// ── Play/Pause ─────────────────────────────────────────────────────────────────

/// Handles play/pause button clicks.
#[tracing::instrument(skip_all)]
pub fn handle_play_pause_button(
    buttons: Query<(&PlayPauseButton, &Interaction, Entity), (Changed<Interaction>, With<Button>)>,
    mut anim_cache: ResMut<ResultAnimCache>,
    children_query: Query<&Children>,
    mut texts: Query<&mut Text>,
) {
    for (btn, interaction, entity) in &buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let Some(AnimState::Ready(playback)) = anim_cache.0.get_mut(&btn.anim_type) else {
            continue;
        };
        playback.playing = !playback.playing;
        let playing_now = playback.playing;

        // Update child text
        if let Ok(children) = children_query.get(entity) {
            for child in children.iter() {
                if let Ok(mut text) = texts.get_mut(child) {
                    text.0 = if playing_now {
                        "Pause".to_string()
                    } else {
                        "Play".to_string()
                    };
                }
            }
        }
    }
}

// ── Frame increment / decrement ────────────────────────────────────────────────

/// Handles frame-decrement button clicks.
#[tracing::instrument(skip_all)]
pub fn handle_frame_decrement(
    buttons: Query<(&FrameDecrementButton, &Interaction), (Changed<Interaction>, With<Button>)>,
    mut anim_cache: ResMut<ResultAnimCache>,
    mut frame_labels: Query<(&FrameNumberLabel, &mut Text)>,
    mut thumbnails: Query<(&CardThumbnail, &mut ImageNode)>,
) {
    for (btn, interaction) in &buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let Some(AnimState::Ready(playback)) = anim_cache.0.get_mut(&btn.anim_type) else {
            continue;
        };
        if playback.frames.is_empty() {
            continue;
        }
        if playback.current_frame == 0 {
            playback.current_frame = playback.frames.len() - 1;
        } else {
            playback.current_frame -= 1;
        }
        let frame_idx = playback.current_frame;
        let total = playback.frames.len();
        let handle = playback.frames[frame_idx].clone();

        for (label, mut text) in &mut frame_labels {
            if label.anim_type == btn.anim_type {
                text.0 = format!("{}/{}", frame_idx + 1, total);
            }
        }
        for (thumb, mut img) in &mut thumbnails {
            if thumb.kind == super::card::CardKind::Anim(btn.anim_type) {
                img.image = handle.clone();
            }
        }
    }
}

/// Handles frame-increment button clicks.
#[tracing::instrument(skip_all)]
pub fn handle_frame_increment(
    buttons: Query<(&FrameIncrementButton, &Interaction), (Changed<Interaction>, With<Button>)>,
    mut anim_cache: ResMut<ResultAnimCache>,
    mut frame_labels: Query<(&FrameNumberLabel, &mut Text)>,
    mut thumbnails: Query<(&CardThumbnail, &mut ImageNode)>,
) {
    for (btn, interaction) in &buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let Some(AnimState::Ready(playback)) = anim_cache.0.get_mut(&btn.anim_type) else {
            continue;
        };
        if playback.frames.is_empty() {
            continue;
        }
        playback.current_frame = (playback.current_frame + 1) % playback.frames.len();
        let frame_idx = playback.current_frame;
        let total = playback.frames.len();
        let handle = playback.frames[frame_idx].clone();

        for (label, mut text) in &mut frame_labels {
            if label.anim_type == btn.anim_type {
                text.0 = format!("{}/{}", frame_idx + 1, total);
            }
        }
        for (thumb, mut img) in &mut thumbnails {
            if thumb.kind == super::card::CardKind::Anim(btn.anim_type) {
                img.image = handle.clone();
            }
        }
    }
}

/// Updates the playback speed timer from `ResultsViewState.playback_speed`.
#[tracing::instrument(skip_all)]
pub fn update_playback_speed(
    view_state: Res<ResultsViewState>,
    mut anim_cache: ResMut<ResultAnimCache>,
) {
    if !view_state.is_changed() {
        return;
    }
    let fps = view_state.playback_speed.max(1.0);
    let duration = std::time::Duration::from_secs_f32(1.0 / fps);

    for state in anim_cache.0.values_mut() {
        if let AnimState::Ready(playback) = state {
            playback.timer = Timer::new(duration, TimerMode::Repeating);
        }
    }
}
