//! Modal overlay for enlarged image / animation display.

#![allow(clippy::type_complexity)]

use bevy::prelude::*;
use num_traits::ToPrimitive;

use super::{
    card::{static_cards_for_tab, AnimCard, CardImageNode, CardKind},
    AnimState, GalleryTab, ImageType, ModalTarget, ResultAnimCache, ResultImageCache,
    ResultImageState, ResultsViewState,
};
use crate::ui::{bevy_shell::content_area::ShellRoot, colors};

// ── Components ─────────────────────────────────────────────────────────────────

/// Marker for the modal overlay root entity (the full-screen dim backdrop).
/// Also carries `Button` so it absorbs pointer events and prevents click-through.
#[derive(Component, Debug)]
pub struct ModalOverlay;

/// Marker for the close button inside the modal.
#[derive(Component, Debug)]
pub struct ModalCloseButton;

/// Marker for Prev / Next navigation buttons inside the modal.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModalNavButton {
    Prev,
    Next,
}

// ── Click handlers ─────────────────────────────────────────────────────────────

/// Opens a modal when the user clicks on a generated image thumbnail (`CardImageNode`).
/// Skips the event when a modal is already open (the overlay absorbs the click, but
/// guard here for safety).
#[tracing::instrument(skip_all)]
pub fn handle_static_card_click(
    img_nodes: Query<(&CardImageNode, &Interaction), (Changed<Interaction>, With<Button>)>,
    mut view_state: ResMut<ResultsViewState>,
    image_cache: Res<ResultImageCache>,
    anim_cache: Res<ResultAnimCache>,
) {
    // Don't open a new modal if one is already open.
    if view_state.modal.is_some() {
        return;
    }
    for (img_node, interaction) in &img_nodes {
        if *interaction != Interaction::Pressed {
            continue;
        }
        match img_node.kind {
            CardKind::Static(image_type) => {
                if matches!(
                    image_cache.0.get(&image_type),
                    Some(ResultImageState::Ready(_))
                ) {
                    view_state.modal = Some(ModalTarget::StaticImage(image_type));
                }
            }
            CardKind::Anim(anim_type) => {
                if matches!(anim_cache.0.get(&anim_type), Some(AnimState::Ready(_))) {
                    view_state.modal = Some(ModalTarget::Animation(anim_type));
                }
            }
        }
    }
}

/// Stub — anim card clicks are now handled in `handle_static_card_click`.
#[tracing::instrument(skip_all)]
pub fn handle_anim_card_click(
    _cards: Query<(&AnimCard, &Interaction), Changed<Interaction>>,
    _view_state: ResMut<ResultsViewState>,
    _anim_cache: Res<ResultAnimCache>,
) {
    // Handled via CardImageNode click in handle_static_card_click above.
}

// ── Modal spawn / update ───────────────────────────────────────────────────────

/// Spawns or despawns the modal overlay based on `ResultsViewState.modal`.
///
/// The modal is parented to `ShellRoot` (the full-window root) so the
/// absolute-positioned overlay covers the sidebar as well as the content area.
#[tracing::instrument(skip_all)]
pub fn update_static_modal(
    mut commands: Commands,
    view_state: Res<ResultsViewState>,
    existing_modals: Query<Entity, With<ModalOverlay>>,
    image_cache: Res<ResultImageCache>,
    anim_cache: Res<ResultAnimCache>,
    images: Res<Assets<Image>>,
    shell_roots: Query<Entity, With<ShellRoot>>,
) {
    if !view_state.is_changed() && !anim_cache.is_changed() {
        return;
    }

    for entity in &existing_modals {
        commands.entity(entity).despawn();
    }

    let Some(target) = view_state.modal else {
        return;
    };

    let Ok(shell_root) = shell_roots.single() else {
        return;
    };

    let modal = match target {
        ModalTarget::StaticImage(image_type) => {
            let Some(ResultImageState::Ready(handle)) = image_cache.0.get(&image_type) else {
                return;
            };
            let nav_list = nav_list_for_image(image_type);
            let aspect_ratio = image_aspect_ratio(&images, handle);
            spawn_image_modal(
                &mut commands,
                handle.clone(),
                Some((image_type, &nav_list)),
                aspect_ratio,
            )
        }
        ModalTarget::Animation(anim_type) => {
            let Some(AnimState::Ready(playback)) = anim_cache.0.get(&anim_type) else {
                return;
            };
            let Some(handle) = playback.frames.get(playback.current_frame) else {
                return;
            };
            let aspect_ratio = image_aspect_ratio(&images, handle);
            spawn_image_modal(&mut commands, handle.clone(), None, aspect_ratio)
        }
    };

    commands.entity(shell_root).add_child(modal);
}

fn image_aspect_ratio(images: &Assets<Image>, handle: &Handle<Image>) -> f32 {
    images.get(handle).map_or(1.0, |image| {
        let size = image.size();
        size.x
            .to_f32()
            .zip(size.y.to_f32())
            .filter(|(_, height)| *height > 0.0)
            .map_or(1.0, |(width, height)| width / height)
    })
}

/// Returns the ordered list of *ready* `ImageType` values for the same tab as
/// `current`, used for Prev/Next navigation. Only includes the full tab list —
/// availability is checked at navigation time.
fn nav_list_for_image(current: ImageType) -> Vec<ImageType> {
    for tab in [
        GalleryTab::SpatialMaps,
        GalleryTab::Metrics,
        GalleryTab::Losses,
        GalleryTab::TimeFunctions,
    ] {
        let list = static_cards_for_tab(tab);
        if list.contains(&current) {
            return list;
        }
    }
    vec![current]
}

fn next_ready_image(
    list: &[ImageType],
    current: ImageType,
    direction: ModalNavButton,
    image_cache: &ResultImageCache,
) -> Option<ImageType> {
    let idx = list.iter().position(|t| *t == current)?;
    let len = list.len();

    for offset in 1..=len {
        let candidate = match direction {
            ModalNavButton::Prev => list[(idx + len - offset) % len],
            ModalNavButton::Next => list[(idx + offset) % len],
        };
        if matches!(
            image_cache.0.get(&candidate),
            Some(ResultImageState::Ready(_))
        ) {
            return Some(candidate);
        }
    }

    None
}

#[tracing::instrument(skip_all)]
fn spawn_image_modal(
    commands: &mut Commands,
    image_handle: Handle<Image>,
    nav: Option<(ImageType, &[ImageType])>,
    aspect_ratio: f32,
) -> Entity {
    // The overlay itself carries `Button` so it absorbs all pointer events and
    // prevents click-through to the cards beneath.
    commands
        .spawn((
            ModalOverlay,
            Button,
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.75)),
            ZIndex(200),
        ))
        .with_children(|overlay| {
            // ── Modal card ──────────────────────────────────────────────
            overlay
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        max_width: Val::Percent(85.0),
                        max_height: Val::Percent(85.0),
                        padding: UiRect::all(Val::Px(8.0)),
                        row_gap: Val::Px(8.0),
                        border_radius: BorderRadius::all(Val::Px(8.0)),
                        ..default()
                    },
                    BackgroundColor(colors::BG1),
                ))
                .with_children(|container| {
                    // ── Header row: Prev / label / Next / Close ─────────
                    container
                        .spawn(Node {
                            flex_direction: FlexDirection::Row,
                            justify_content: JustifyContent::FlexEnd,
                            align_items: AlignItems::Center,
                            column_gap: Val::Px(8.0),
                            ..default()
                        })
                        .with_children(|row| {
                            if nav.is_some() {
                                row.spawn((
                                    ModalNavButton::Prev,
                                    Button,
                                    Node {
                                        padding: UiRect::axes(Val::Px(10.0), Val::Px(3.0)),
                                        border_radius: BorderRadius::all(Val::Px(4.0)),
                                        ..default()
                                    },
                                    BackgroundColor(colors::BG3),
                                ))
                                .with_children(|btn| {
                                    btn.spawn((
                                        Text::new("<"),
                                        TextFont {
                                            font_size: 12.0,
                                            ..default()
                                        },
                                        TextColor(colors::FG0),
                                    ));
                                });

                                row.spawn(Node {
                                    flex_grow: 1.0,
                                    ..default()
                                });

                                row.spawn((
                                    ModalNavButton::Next,
                                    Button,
                                    Node {
                                        padding: UiRect::axes(Val::Px(10.0), Val::Px(3.0)),
                                        border_radius: BorderRadius::all(Val::Px(4.0)),
                                        ..default()
                                    },
                                    BackgroundColor(colors::BG3),
                                ))
                                .with_children(|btn| {
                                    btn.spawn((
                                        Text::new(">"),
                                        TextFont {
                                            font_size: 12.0,
                                            ..default()
                                        },
                                        TextColor(colors::FG0),
                                    ));
                                });
                            }

                            // Close button
                            row.spawn((
                                ModalCloseButton,
                                Button,
                                Node {
                                    padding: UiRect::axes(Val::Px(8.0), Val::Px(3.0)),
                                    border_radius: BorderRadius::all(Val::Px(4.0)),
                                    ..default()
                                },
                                BackgroundColor(colors::BG3),
                            ))
                            .with_children(|btn| {
                                btn.spawn((
                                    Text::new("X"),
                                    TextFont {
                                        font_size: 12.0,
                                        ..default()
                                    },
                                    TextColor(colors::FG0),
                                ));
                            });
                        });

                    // ── Image ────────────────────────────────────────────
                    container
                        .spawn(Node {
                            width: Val::Percent(100.0),
                            flex_grow: 1.0,
                            min_height: Val::Px(0.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            overflow: Overflow::clip(),
                            ..default()
                        })
                        .with_children(|image_container| {
                            image_container.spawn((
                                ImageNode::new(image_handle),
                                Node {
                                    max_width: Val::Percent(100.0),
                                    max_height: Val::Percent(100.0),
                                    aspect_ratio: Some(aspect_ratio),
                                    ..default()
                                },
                            ));
                        });
                });
        })
        .id()
}

// ── Close / nav handlers ───────────────────────────────────────────────────────

/// Handles clicks on the modal close button.
#[tracing::instrument(skip_all)]
pub fn handle_modal_close(
    buttons: Query<(&ModalCloseButton, &Interaction), (Changed<Interaction>, With<Button>)>,
    mut view_state: ResMut<ResultsViewState>,
) {
    for (_, interaction) in &buttons {
        if *interaction == Interaction::Pressed {
            view_state.modal = None;
        }
    }
}

/// Handles clicks on Prev / Next navigation buttons inside the modal.
#[tracing::instrument(skip_all)]
pub fn handle_modal_nav(
    buttons: Query<(&ModalNavButton, &Interaction), (Changed<Interaction>, With<Button>)>,
    mut view_state: ResMut<ResultsViewState>,
    image_cache: Res<ResultImageCache>,
) {
    for (nav, interaction) in &buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let Some(ModalTarget::StaticImage(current)) = view_state.modal else {
            continue;
        };
        let list = nav_list_for_image(current);
        if let Some(candidate) = next_ready_image(&list, current, *nav, &image_cache) {
            view_state.modal = Some(ModalTarget::StaticImage(candidate));
        }
    }
}

/// Closes the modal on Escape; navigates with Left / Right arrow keys.
#[tracing::instrument(skip_all)]
pub fn handle_modal_keyboard(
    keys: Res<ButtonInput<KeyCode>>,
    mut view_state: ResMut<ResultsViewState>,
    image_cache: Res<ResultImageCache>,
) {
    if view_state.modal.is_none() {
        return;
    }

    if keys.just_pressed(KeyCode::Escape) {
        view_state.modal = None;
        return;
    }

    let direction = if keys.just_pressed(KeyCode::ArrowLeft) {
        ModalNavButton::Prev
    } else if keys.just_pressed(KeyCode::ArrowRight) {
        ModalNavButton::Next
    } else {
        return;
    };

    let Some(ModalTarget::StaticImage(current)) = view_state.modal else {
        return;
    };
    let list = nav_list_for_image(current);
    if let Some(candidate) = next_ready_image(&list, current, direction, &image_cache) {
        view_state.modal = Some(ModalTarget::StaticImage(candidate));
    }
}
