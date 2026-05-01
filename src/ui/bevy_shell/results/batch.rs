//! Batch generation systems: "Generate All in Tab" and "Generate All".

#![allow(clippy::type_complexity)]

use bevy::prelude::*;

use super::{
    card::{anim_cards_for_tab, static_cards_for_tab},
    gallery::{BatchProgressLabel, GenerateAllButton, GenerateAllInTabButton},
    generate, AnimState, ResultAnimCache, ResultImageCache, ResultImageState,
};
use crate::ActiveLoadedScenario;

// ── Generate All in Tab ────────────────────────────────────────────────────────

/// Handles the "Generate All in Tab" button: queues all Pending cards in the active tab.
#[tracing::instrument(skip_all)]
pub fn handle_generate_all_in_tab(
    buttons: Query<(&GenerateAllInTabButton, &Interaction), (Changed<Interaction>, With<Button>)>,
    mut image_cache: ResMut<ResultImageCache>,
    mut anim_cache: ResMut<ResultAnimCache>,
    mut view_state: ResMut<super::ResultsViewState>,
    active_loaded_scenario: Res<ActiveLoadedScenario>,
) {
    let mut pressed = false;
    for (_, interaction) in &buttons {
        if *interaction == Interaction::Pressed {
            pressed = true;
        }
    }
    if !pressed {
        return;
    }

    let Some(active) = active_loaded_scenario.0.as_ref() else {
        return;
    };

    let tab = view_state.active_tab;
    let static_types = static_cards_for_tab(tab);
    let anim_types = anim_cards_for_tab(tab);

    view_state.batch_total += static_types.len() + anim_types.len();

    for image_type in static_types {
        if matches!(
            image_cache.0.get(&image_type),
            None | Some(ResultImageState::Pending | ResultImageState::Ready(_))
        ) {
            let scenario = active.scenario.clone();
            let payload = active.payload.clone();
            let output_path = active
                .storage
                .image_path(scenario.get_id(), &image_type.to_string());
            let channel = super::new_channel::<std::path::PathBuf>();
            let writer = channel.clone();
            std::thread::spawn(move || {
                let result = generate::generate_image(scenario, payload, output_path, image_type);
                if let Ok(mut guard) = writer.lock() {
                    *guard = Some(result);
                }
            });
            image_cache
                .0
                .insert(image_type, ResultImageState::Generating { channel });
        }
    }

    for anim_type in anim_types {
        if matches!(
            anim_cache.0.get(&anim_type),
            None | Some(AnimState::Pending | AnimState::Ready(_))
        ) {
            let scenario = active.scenario.clone();
            let payload = active.payload.clone();
            let anim_dir = active
                .storage
                .animation_dir(scenario.get_id(), anim_type.dir_name());
            let channel = super::new_channel::<std::path::PathBuf>();
            let writer = channel.clone();
            std::thread::spawn(move || {
                let result = generate::generate_animation(scenario, payload, anim_dir, anim_type);
                if let Ok(mut guard) = writer.lock() {
                    *guard = Some(result);
                }
            });
            anim_cache
                .0
                .insert(anim_type, AnimState::Generating { channel });
        }
    }
}

// ── Generate All ──────────────────────────────────────────────────────────────

/// Handles the "Generate All" button: queues all Pending cards across all tabs.
#[tracing::instrument(skip_all)]
pub fn handle_generate_all(
    buttons: Query<(&GenerateAllButton, &Interaction), (Changed<Interaction>, With<Button>)>,
    mut image_cache: ResMut<ResultImageCache>,
    mut anim_cache: ResMut<ResultAnimCache>,
    mut view_state: ResMut<super::ResultsViewState>,
    active_loaded_scenario: Res<ActiveLoadedScenario>,
) {
    let mut pressed = false;
    for (_, interaction) in &buttons {
        if *interaction == Interaction::Pressed {
            pressed = true;
        }
    }
    if !pressed {
        return;
    }

    let Some(active) = active_loaded_scenario.0.as_ref() else {
        return;
    };

    use super::GalleryTab;
    let all_tabs = [
        GalleryTab::SpatialMaps,
        GalleryTab::Metrics,
        GalleryTab::Losses,
        GalleryTab::TimeFunctions,
    ];

    for tab in all_tabs {
        view_state.batch_total += static_cards_for_tab(tab).len() + anim_cards_for_tab(tab).len();
    }

    for tab in all_tabs {
        for image_type in static_cards_for_tab(tab) {
            if matches!(
                image_cache.0.get(&image_type),
                None | Some(ResultImageState::Pending | ResultImageState::Ready(_))
            ) {
                let scenario = active.scenario.clone();
                let payload = active.payload.clone();
                let output_path = active
                    .storage
                    .image_path(scenario.get_id(), &image_type.to_string());
                let channel = super::new_channel::<std::path::PathBuf>();
                let writer = channel.clone();
                std::thread::spawn(move || {
                    let result =
                        generate::generate_image(scenario, payload, output_path, image_type);
                    if let Ok(mut guard) = writer.lock() {
                        *guard = Some(result);
                    }
                });
                image_cache
                    .0
                    .insert(image_type, ResultImageState::Generating { channel });
            }
        }

        for anim_type in anim_cards_for_tab(tab) {
            if matches!(
                anim_cache.0.get(&anim_type),
                None | Some(AnimState::Pending | AnimState::Ready(_))
            ) {
                let scenario = active.scenario.clone();
                let payload = active.payload.clone();
                let anim_dir = active
                    .storage
                    .animation_dir(scenario.get_id(), anim_type.dir_name());
                let channel = super::new_channel::<std::path::PathBuf>();
                let writer = channel.clone();
                std::thread::spawn(move || {
                    let result =
                        generate::generate_animation(scenario, payload, anim_dir, anim_type);
                    if let Ok(mut guard) = writer.lock() {
                        *guard = Some(result);
                    }
                });
                anim_cache
                    .0
                    .insert(anim_type, AnimState::Generating { channel });
            }
        }
    }
}

// ── Batch progress update ──────────────────────────────────────────────────────

/// Updates the batch progress label based on current cache states.
#[tracing::instrument(skip_all)]
pub fn update_batch_progress(
    image_cache: Res<ResultImageCache>,
    anim_cache: Res<ResultAnimCache>,
    view_state: Res<super::ResultsViewState>,
    mut labels: Query<&mut Text, With<BatchProgressLabel>>,
) {
    if !image_cache.is_changed() && !anim_cache.is_changed() && !view_state.is_changed() {
        return;
    }

    let done = image_cache
        .0
        .values()
        .filter(|s| matches!(s, ResultImageState::Ready(_) | ResultImageState::Failed(_)))
        .count()
        + anim_cache
            .0
            .values()
            .filter(|s| matches!(s, AnimState::Ready(_) | AnimState::Failed(_)))
            .count();

    let in_progress = image_cache
        .0
        .values()
        .filter(|s| {
            matches!(
                s,
                ResultImageState::Generating { .. } | ResultImageState::Loading { .. }
            )
        })
        .count()
        + anim_cache
            .0
            .values()
            .filter(|s| matches!(s, AnimState::Generating { .. } | AnimState::Loading { .. }))
            .count();

    let label_text = if in_progress > 0 {
        format!("{done} done, {in_progress} in progress")
    } else if done > 0 {
        format!("{done} generated")
    } else {
        String::new()
    };

    for mut text in &mut labels {
        text.0.clone_from(&label_text);
    }
}
