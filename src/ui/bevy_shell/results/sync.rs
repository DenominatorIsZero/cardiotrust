//! Card state sync systems: update card visuals to match cache states.

#![allow(clippy::type_complexity)]

use bevy::prelude::*;

use super::{
    card::{
        AnimCard, CardGenerateButton, CardImageNode, CardKind, CardRetryButton, CardSaveButton,
        CardStatusLabel, StaticImageCard,
    },
    AnimState, ResultAnimCache, ResultImageCache, ResultImageState,
};
use crate::ui::colors;

// ── Static card sync ──────────────────────────────────────────────────────────

/// Syncs static-card status to match `ResultImageCache`.
#[tracing::instrument(skip_all)]
pub fn sync_static_card_state(
    image_cache: Res<ResultImageCache>,
    mut status_labels: Query<(&CardStatusLabel, &mut Text, &mut Visibility)>,
    mut image_nodes: Query<
        (&CardImageNode, &mut Node, &mut ImageNode),
        (
            Without<CardGenerateButton>,
            Without<CardRetryButton>,
            Without<CardSaveButton>,
        ),
    >,
    mut generate_btns: Query<
        (&CardGenerateButton, &mut Node),
        (
            Without<CardRetryButton>,
            Without<CardImageNode>,
            Without<CardSaveButton>,
        ),
    >,
    mut save_btns: Query<
        (&CardSaveButton, &mut Node),
        (
            Without<CardGenerateButton>,
            Without<CardRetryButton>,
            Without<CardImageNode>,
        ),
    >,
    mut retry_btns: Query<
        (&CardRetryButton, &mut Node),
        (
            Without<CardGenerateButton>,
            Without<CardImageNode>,
            Without<CardSaveButton>,
        ),
    >,
) {
    if !image_cache.is_changed() {
        return;
    }

    for (status, mut text, mut vis) in &mut status_labels {
        let CardKind::Static(image_type) = status.kind else {
            continue;
        };
        let state = image_cache.0.get(&image_type);
        match state {
            None | Some(ResultImageState::Pending) => {
                text.0 = "Not generated".to_string();
                *vis = Visibility::Visible;
            }
            Some(ResultImageState::Generating { .. }) => {
                text.0 = "Generating...".to_string();
                *vis = Visibility::Visible;
            }
            Some(ResultImageState::Loading { .. }) => {
                text.0 = "Loading...".to_string();
                *vis = Visibility::Visible;
            }
            Some(ResultImageState::Ready(_)) => {
                text.0 = String::new();
                *vis = Visibility::Hidden;
            }
            Some(ResultImageState::Failed(msg)) => {
                text.0 = format!("Error: {msg}");
                *vis = Visibility::Visible;
            }
        }
    }

    // Show/hide the image node and set the handle when Ready.
    for (img_node, mut node, mut image_node) in &mut image_nodes {
        let CardKind::Static(image_type) = img_node.kind else {
            continue;
        };
        if let Some(ResultImageState::Ready(handle)) = image_cache.0.get(&image_type) {
            image_node.image = handle.clone();
            node.display = Display::Flex;
        } else {
            node.display = Display::None;
        }
    }

    for (btn, mut node) in &mut generate_btns {
        let CardKind::Static(image_type) = btn.kind else {
            continue;
        };
        // Show only when Pending/not-yet-started or Failed (hide when working or done).
        let show_generate = matches!(
            image_cache.0.get(&image_type),
            None | Some(ResultImageState::Pending | ResultImageState::Failed(_))
        );
        node.display = if show_generate {
            Display::Flex
        } else {
            Display::None
        };
    }

    for (btn, mut node) in &mut save_btns {
        let CardKind::Static(image_type) = btn.kind else {
            continue;
        };
        let show_save = matches!(
            image_cache.0.get(&image_type),
            Some(ResultImageState::Ready(_))
        );
        node.display = if show_save {
            Display::Flex
        } else {
            Display::None
        };
    }

    for (btn, mut node) in &mut retry_btns {
        let CardKind::Static(image_type) = btn.kind else {
            continue;
        };
        let show_retry = matches!(
            image_cache.0.get(&image_type),
            Some(ResultImageState::Failed(_))
        );
        node.display = if show_retry {
            Display::Flex
        } else {
            Display::None
        };
    }
}

// ── Anim card sync ─────────────────────────────────────────────────────────────

/// Syncs animation-card status to match `ResultAnimCache`.
#[tracing::instrument(skip_all)]
pub fn sync_anim_card_state(
    anim_cache: Res<ResultAnimCache>,
    mut status_labels: Query<(&CardStatusLabel, &mut Text, &mut Visibility)>,
    mut image_nodes: Query<
        (&CardImageNode, &mut Node, &mut ImageNode),
        (
            Without<CardGenerateButton>,
            Without<CardRetryButton>,
            Without<CardSaveButton>,
        ),
    >,
    mut generate_btns: Query<
        (&CardGenerateButton, &mut Node),
        (
            Without<CardRetryButton>,
            Without<CardImageNode>,
            Without<CardSaveButton>,
        ),
    >,
    mut save_btns: Query<
        (&CardSaveButton, &mut Node),
        (
            Without<CardGenerateButton>,
            Without<CardRetryButton>,
            Without<CardImageNode>,
        ),
    >,
    mut retry_btns: Query<
        (&CardRetryButton, &mut Node),
        (
            Without<CardGenerateButton>,
            Without<CardImageNode>,
            Without<CardSaveButton>,
        ),
    >,
) {
    if !anim_cache.is_changed() {
        return;
    }

    for (status, mut text, mut vis) in &mut status_labels {
        let CardKind::Anim(anim_type) = status.kind else {
            continue;
        };
        let state = anim_cache.0.get(&anim_type);
        match state {
            None | Some(AnimState::Pending) => {
                text.0 = "Not generated".to_string();
                *vis = Visibility::Visible;
            }
            Some(AnimState::Generating { .. }) => {
                text.0 = "Generating...".to_string();
                *vis = Visibility::Visible;
            }
            Some(AnimState::Loading { channels, loaded }) => {
                let done = loaded.iter().filter(|s| s.is_some()).count();
                text.0 = format!("Loading {done}/{} frames...", channels.len());
                *vis = Visibility::Visible;
            }
            Some(AnimState::Ready(_)) => {
                text.0 = String::new();
                *vis = Visibility::Hidden;
            }
            Some(AnimState::Failed(msg)) => {
                text.0 = format!("Error: {msg}");
                *vis = Visibility::Visible;
            }
        }
    }

    // Show the current frame when Ready.
    for (img_node, mut node, mut image_node) in &mut image_nodes {
        let CardKind::Anim(anim_type) = img_node.kind else {
            continue;
        };
        if let Some(AnimState::Ready(playback)) = anim_cache.0.get(&anim_type) {
            if let Some(handle) = playback.frames.get(playback.current_frame) {
                image_node.image = handle.clone();
                node.display = Display::Flex;
            }
        } else {
            node.display = Display::None;
        }
    }

    for (btn, mut node) in &mut generate_btns {
        let CardKind::Anim(anim_type) = btn.kind else {
            continue;
        };
        // Show only when not-yet-started or Failed.
        let show_generate = matches!(
            anim_cache.0.get(&anim_type),
            None | Some(AnimState::Pending | AnimState::Failed(_))
        );
        node.display = if show_generate {
            Display::Flex
        } else {
            Display::None
        };
    }

    for (btn, mut node) in &mut save_btns {
        let CardKind::Anim(anim_type) = btn.kind else {
            continue;
        };
        let show_save = matches!(anim_cache.0.get(&anim_type), Some(AnimState::Ready(_)));
        node.display = if show_save {
            Display::Flex
        } else {
            Display::None
        };
    }

    for (btn, mut node) in &mut retry_btns {
        let CardKind::Anim(anim_type) = btn.kind else {
            continue;
        };
        let show_retry = matches!(anim_cache.0.get(&anim_type), Some(AnimState::Failed(_)));
        node.display = if show_retry {
            Display::Flex
        } else {
            Display::None
        };
    }
}

// ── Hover highlight ────────────────────────────────────────────────────────────

/// Updates card border color on hover.
#[tracing::instrument(skip_all)]
pub fn update_card_hover(
    mut static_cards: Query<
        (&Interaction, &mut BorderColor),
        (Changed<Interaction>, With<StaticImageCard>),
    >,
    mut anim_cards: Query<
        (&Interaction, &mut BorderColor),
        (
            Changed<Interaction>,
            With<AnimCard>,
            Without<StaticImageCard>,
        ),
    >,
) {
    for (interaction, mut border) in &mut static_cards {
        border.set_if_neq(match interaction {
            Interaction::Hovered => BorderColor::all(colors::GREY1),
            _ => BorderColor::all(colors::BG3),
        });
    }
    for (interaction, mut border) in &mut anim_cards {
        border.set_if_neq(match interaction {
            Interaction::Hovered => BorderColor::all(colors::GREY1),
            _ => BorderColor::all(colors::BG3),
        });
    }
}

// ── Generate button handlers ───────────────────────────────────────────────────

/// Handles clicks on "Generate" buttons for static images.
#[tracing::instrument(skip_all)]
pub fn handle_generate_button_static(
    buttons: Query<(&CardGenerateButton, &Interaction), (Changed<Interaction>, With<Button>)>,
    mut image_cache: ResMut<ResultImageCache>,
    scenario_list: Res<crate::ScenarioList>,
    selected: Res<crate::SelectedSenario>,
) {
    for (btn, interaction) in &buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let CardKind::Static(image_type) = btn.kind else {
            continue;
        };
        if !matches!(
            image_cache.0.get(&image_type),
            None | Some(ResultImageState::Pending)
        ) {
            continue;
        }

        let Some(index) = selected.index else {
            continue;
        };
        let Some(entry) = scenario_list.entries.get(index) else {
            continue;
        };
        let scenario = entry.scenario.clone();

        let channel = super::new_channel::<std::path::PathBuf>();
        let channel_writer = channel.clone();

        std::thread::spawn(move || {
            let result = super::generate::generate_image(scenario.clone(), image_type).map(|()| {
                std::path::Path::new("results")
                    .join(scenario.get_id())
                    .join("img")
                    .join(image_type.to_string())
                    .with_extension("png")
            });
            if let Ok(mut guard) = channel_writer.lock() {
                *guard = Some(result);
            }
        });
        image_cache
            .0
            .insert(image_type, ResultImageState::Generating { channel });
    }
}

/// Handles clicks on "Generate" buttons for animations.
#[tracing::instrument(skip_all)]
pub fn handle_generate_button_anim(
    buttons: Query<(&CardGenerateButton, &Interaction), (Changed<Interaction>, With<Button>)>,
    mut anim_cache: ResMut<ResultAnimCache>,
    scenario_list: Res<crate::ScenarioList>,
    selected: Res<crate::SelectedSenario>,
) {
    for (btn, interaction) in &buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let CardKind::Anim(anim_type) = btn.kind else {
            continue;
        };
        if !matches!(
            anim_cache.0.get(&anim_type),
            None | Some(AnimState::Pending)
        ) {
            continue;
        }

        let Some(index) = selected.index else {
            continue;
        };
        let Some(entry) = scenario_list.entries.get(index) else {
            continue;
        };
        let scenario = entry.scenario.clone();

        let channel = super::new_channel::<std::path::PathBuf>();
        let channel_writer = channel.clone();

        std::thread::spawn(move || {
            let result = super::generate::generate_animation(scenario, anim_type);
            if let Ok(mut guard) = channel_writer.lock() {
                *guard = Some(result);
            }
        });

        anim_cache
            .0
            .insert(anim_type, AnimState::Generating { channel });
    }
}

/// Handles clicks on "Retry" buttons — resets state to `Pending`.
#[tracing::instrument(skip_all)]
pub fn handle_retry_button(
    buttons: Query<(&CardRetryButton, &Interaction), (Changed<Interaction>, With<Button>)>,
    mut image_cache: ResMut<ResultImageCache>,
    mut anim_cache: ResMut<ResultAnimCache>,
) {
    for (btn, interaction) in &buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }
        match btn.kind {
            CardKind::Static(image_type) => {
                image_cache.0.insert(image_type, ResultImageState::Pending);
            }
            CardKind::Anim(anim_type) => {
                anim_cache.0.insert(anim_type, AnimState::Pending);
            }
        }
    }
}
