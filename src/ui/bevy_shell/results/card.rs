//! Result card component definitions and spawning.
//!
//! Each card represents one static image or one animation. Cards are spawned
//! into their corresponding `GalleryTabBody` node.

use bevy::prelude::*;

use super::{AnimType, GalleryTab, ImageType};
use crate::ui::colors;

// ── Card kind ─────────────────────────────────────────────────────────────────

/// Describes whether a card holds a static image or an animation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardKind {
    Static(ImageType),
    Anim(AnimType),
}

// ── Components ─────────────────────────────────────────────────────────────────

/// Marker on the root node of a static-image card.
#[derive(Component, Debug, Clone, Copy)]
pub struct StaticImageCard {
    pub image_type: ImageType,
}

/// Marker on the root node of an animation card.
#[derive(Component, Debug, Clone, Copy)]
pub struct AnimCard {
    pub anim_type: AnimType,
}

/// Marker on the thumbnail image node inside a card.
#[derive(Component, Debug, Clone, Copy)]
pub struct CardThumbnail {
    pub kind: CardKind,
}

/// Marker on the "Generate" button inside a card.
#[derive(Component, Debug, Clone, Copy)]
pub struct CardGenerateButton {
    pub kind: CardKind,
}

/// Marker on the "Retry" button inside a card.
#[derive(Component, Debug, Clone, Copy)]
pub struct CardRetryButton {
    pub kind: CardKind,
}

/// Marker on the "Save" button inside a card (shown when Ready).
#[derive(Component, Debug, Clone, Copy)]
pub struct CardSaveButton {
    pub kind: CardKind,
}

/// Marker on the spinner/status text label inside a card.
#[derive(Component, Debug, Clone, Copy)]
pub struct CardStatusLabel {
    pub kind: CardKind,
}

/// Marker on the `ImageNode` inside a card thumbnail area.
#[derive(Component, Debug, Clone, Copy)]
pub struct CardImageNode {
    pub kind: CardKind,
}

/// Marker on the play/pause button inside an animation card.
#[derive(Component, Debug, Clone, Copy)]
pub struct PlayPauseButton {
    pub anim_type: AnimType,
}

/// Marker on the frame-decrement button inside an animation card.
#[derive(Component, Debug, Clone, Copy)]
pub struct FrameDecrementButton {
    pub anim_type: AnimType,
}

/// Marker on the frame-increment button inside an animation card.
#[derive(Component, Debug, Clone, Copy)]
pub struct FrameIncrementButton {
    pub anim_type: AnimType,
}

/// Marker on the frame-number label inside an animation card.
#[derive(Component, Debug, Clone, Copy)]
pub struct FrameNumberLabel {
    pub anim_type: AnimType,
}

// ── Descriptors ───────────────────────────────────────────────────────────────

/// Returns the list of static `ImageType` cards for the given tab.
#[tracing::instrument(level = "trace")]
pub fn static_cards_for_tab(tab: GalleryTab) -> Vec<ImageType> {
    match tab {
        GalleryTab::SpatialMaps => vec![
            ImageType::StatesMaxAlgorithm,
            ImageType::StatesMaxSimulation,
            ImageType::StatesMaxDelta,
            ImageType::ActivationTimeAlgorithm,
            ImageType::ActivationTimeSimulation,
            ImageType::ActivationTimeDelta,
            ImageType::VoxelTypesAlgorithm,
            ImageType::VoxelTypesSimulation,
            ImageType::VoxelTypesPrediction,
            ImageType::AverageDelaySimulation,
            ImageType::AveragePropagationSpeedSimulation,
            ImageType::AverageDelayAlgorithm,
            ImageType::AveragePropagationSpeedAlgorithm,
            ImageType::AverageDelayDelta,
        ],
        GalleryTab::Metrics => vec![
            ImageType::Dice,
            ImageType::IoU,
            ImageType::Recall,
            ImageType::Precision,
        ],
        GalleryTab::Losses => vec![
            ImageType::LossEpoch,
            ImageType::Loss,
            ImageType::LossMseEpoch,
            ImageType::LossMse,
            ImageType::LossMaximumRegularizationEpoch,
            ImageType::LossMaximumRegularization,
        ],
        GalleryTab::TimeFunctions => vec![
            ImageType::ControlFunctionAlgorithm,
            ImageType::ControlFunctionSimulation,
            ImageType::ControlFunctionDelta,
            ImageType::StateAlgorithm,
            ImageType::StateSimulation,
            ImageType::StateDelta,
            ImageType::MeasurementAlgorithm,
            ImageType::MeasurementSimulation,
            ImageType::MeasurementDelta,
        ],
    }
}

/// Returns the list of `AnimType` cards for the given tab.
#[tracing::instrument(level = "trace")]
pub fn anim_cards_for_tab(tab: GalleryTab) -> Vec<AnimType> {
    match tab {
        GalleryTab::SpatialMaps => vec![
            AnimType::StatesAlgorithm,
            AnimType::StatesSimulation,
            AnimType::MatrixOverSlices,
            AnimType::VoxelTypesOverSlices,
        ],
        GalleryTab::Metrics | GalleryTab::Losses | GalleryTab::TimeFunctions => vec![],
    }
}

// ── Spawning ───────────────────────────────────────────────────────────────────

/// Spawns all cards for the given tab into `body_entity`.
#[tracing::instrument(skip(commands))]
pub fn spawn_gallery_cards(commands: &mut Commands, body_entity: Entity, tab: GalleryTab) {
    for image_type in static_cards_for_tab(tab) {
        let card = spawn_static_card(commands, image_type);
        commands.entity(body_entity).add_child(card);
    }
    for anim_type in anim_cards_for_tab(tab) {
        let card = spawn_anim_card(commands, anim_type);
        commands.entity(body_entity).add_child(card);
    }
}

/// Spawns a single static-image card and returns its entity.
#[tracing::instrument(skip(commands))]
pub fn spawn_static_card(commands: &mut Commands, image_type: ImageType) -> Entity {
    let kind = CardKind::Static(image_type);
    let label = image_type.to_string();

    let card = commands
        .spawn((
            StaticImageCard { image_type },
            Node {
                // Fill the grid cell; height is driven by the image aspect ratio.
                flex_direction: FlexDirection::Column,
                border: UiRect::all(Val::Px(1.0)),
                border_radius: BorderRadius::all(Val::Px(6.0)),
                overflow: Overflow::clip(),
                ..default()
            },
            BackgroundColor(colors::BG1),
            BorderColor::all(colors::BG3),
        ))
        .id();

    commands.entity(card).with_children(|c| {
        // Title bar
        c.spawn((
            Node {
                padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
                ..default()
            },
            BackgroundColor(colors::BG2),
        ))
        .with_children(|title| {
            title.spawn((
                Text::new(&label),
                TextFont {
                    font_size: 10.0,
                    ..default()
                },
                TextColor(colors::GREY2),
            ));
        });

        // Thumbnail area — fixed height keeps cards uniform; image scales to fit.
        c.spawn((
            CardThumbnail { kind },
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(160.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                overflow: Overflow::clip(),
                ..default()
            },
            BackgroundColor(colors::BG0),
        ))
        .with_children(|thumb| {
            // Image node: constrained to container height, width auto → correct
            // aspect ratio without overflowing.
            thumb.spawn((
                CardImageNode { kind },
                Button,
                Node {
                    height: Val::Percent(100.0),
                    width: Val::Auto,
                    display: Display::None,
                    ..default()
                },
                ImageNode::default(),
            ));
            // Status label (shown when not Ready) — centred by parent flex.
            thumb.spawn((
                CardStatusLabel { kind },
                Node::default(),
                Text::new("Not generated"),
                TextFont {
                    font_size: 10.0,
                    ..default()
                },
                TextColor(colors::GREY1),
            ));
        });

        // Button row — shown below the image.
        c.spawn(Node {
            flex_direction: FlexDirection::Row,
            padding: UiRect::axes(Val::Px(6.0), Val::Px(4.0)),
            column_gap: Val::Px(4.0),
            justify_content: JustifyContent::FlexEnd,
            ..default()
        })
        .with_children(|row| {
            // Generate button (hidden when Ready)
            row.spawn((
                CardGenerateButton { kind },
                Button,
                Node {
                    padding: UiRect::axes(Val::Px(8.0), Val::Px(3.0)),
                    border_radius: BorderRadius::all(Val::Px(3.0)),
                    ..default()
                },
                BackgroundColor(colors::BG3),
            ))
            .with_children(|btn| {
                btn.spawn((
                    Text::new("Generate"),
                    TextFont {
                        font_size: 10.0,
                        ..default()
                    },
                    TextColor(colors::FG0),
                ));
            });

            // Save button (hidden until Ready)
            row.spawn((
                CardSaveButton { kind },
                Button,
                Node {
                    padding: UiRect::axes(Val::Px(8.0), Val::Px(3.0)),
                    border_radius: BorderRadius::all(Val::Px(3.0)),
                    display: Display::None,
                    ..default()
                },
                BackgroundColor(colors::BG3),
            ))
            .with_children(|btn| {
                btn.spawn((
                    Text::new("Save"),
                    TextFont {
                        font_size: 10.0,
                        ..default()
                    },
                    TextColor(colors::FG0),
                ));
            });

            // Retry button (hidden by default)
            row.spawn((
                CardRetryButton { kind },
                Button,
                Node {
                    padding: UiRect::axes(Val::Px(8.0), Val::Px(3.0)),
                    border_radius: BorderRadius::all(Val::Px(3.0)),
                    display: Display::None,
                    ..default()
                },
                BackgroundColor(colors::RED),
            ))
            .with_children(|btn| {
                btn.spawn((
                    Text::new("Retry"),
                    TextFont {
                        font_size: 10.0,
                        ..default()
                    },
                    TextColor(colors::FG0),
                ));
            });
        });
    });

    card
}

/// Spawns a single animation card and returns its entity.
#[tracing::instrument(skip(commands))]
pub fn spawn_anim_card(commands: &mut Commands, anim_type: AnimType) -> Entity {
    let kind = CardKind::Anim(anim_type);
    let label = anim_type.to_string();

    let card = commands
        .spawn((
            AnimCard { anim_type },
            Node {
                // Fill the grid cell; height driven by image aspect ratio.
                flex_direction: FlexDirection::Column,
                border: UiRect::all(Val::Px(1.0)),
                overflow: Overflow::clip(),
                border_radius: BorderRadius::all(Val::Px(6.0)),
                ..default()
            },
            BackgroundColor(colors::BG1),
            BorderColor::all(colors::BG3),
        ))
        .id();

    commands.entity(card).with_children(|c| {
        // Title bar
        c.spawn((
            Node {
                padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
                ..default()
            },
            BackgroundColor(colors::BG2),
        ))
        .with_children(|title| {
            title.spawn((
                Text::new(&label),
                TextFont {
                    font_size: 10.0,
                    ..default()
                },
                TextColor(colors::GREY2),
            ));
        });

        // Thumbnail area — fixed height keeps cards uniform; image scales to fit.
        c.spawn((
            CardThumbnail { kind },
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(160.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                overflow: Overflow::clip(),
                ..default()
            },
            BackgroundColor(colors::BG0),
        ))
        .with_children(|thumb| {
            // Image node: constrained to container height, width auto → correct
            // aspect ratio without overflowing.
            thumb.spawn((
                CardImageNode { kind },
                Button,
                Node {
                    height: Val::Percent(100.0),
                    width: Val::Auto,
                    display: Display::None,
                    ..default()
                },
                ImageNode::default(),
            ));
            thumb.spawn((
                CardStatusLabel { kind },
                Node::default(),
                Text::new("Not generated"),
                TextFont {
                    font_size: 10.0,
                    ..default()
                },
                TextColor(colors::GREY1),
            ));
        });

        // Playback controls row
        c.spawn(Node {
            flex_direction: FlexDirection::Row,
            padding: UiRect::axes(Val::Px(6.0), Val::Px(4.0)),
            column_gap: Val::Px(4.0),
            align_items: AlignItems::Center,
            ..default()
        })
        .with_children(|row| {
            // Play/Pause button
            row.spawn((
                PlayPauseButton { anim_type },
                Button,
                Node {
                    padding: UiRect::axes(Val::Px(8.0), Val::Px(3.0)),
                    border_radius: BorderRadius::all(Val::Px(3.0)),
                    ..default()
                },
                BackgroundColor(colors::BG3),
            ))
            .with_children(|btn| {
                btn.spawn((
                    Text::new("Play"),
                    TextFont {
                        font_size: 10.0,
                        ..default()
                    },
                    TextColor(colors::FG0),
                ));
            });

            // Frame decrement
            row.spawn((
                FrameDecrementButton { anim_type },
                Button,
                Node {
                    padding: UiRect::axes(Val::Px(6.0), Val::Px(3.0)),
                    border_radius: BorderRadius::all(Val::Px(3.0)),
                    ..default()
                },
                BackgroundColor(colors::BG3),
            ))
            .with_children(|btn| {
                btn.spawn((
                    Text::new("<"),
                    TextFont {
                        font_size: 10.0,
                        ..default()
                    },
                    TextColor(colors::FG0),
                ));
            });

            // Frame number label
            row.spawn((
                FrameNumberLabel { anim_type },
                Text::new("0/0"),
                TextFont {
                    font_size: 10.0,
                    ..default()
                },
                TextColor(colors::GREY1),
            ));

            // Frame increment
            row.spawn((
                FrameIncrementButton { anim_type },
                Button,
                Node {
                    padding: UiRect::axes(Val::Px(6.0), Val::Px(3.0)),
                    border_radius: BorderRadius::all(Val::Px(3.0)),
                    ..default()
                },
                BackgroundColor(colors::BG3),
            ))
            .with_children(|btn| {
                btn.spawn((
                    Text::new(">"),
                    TextFont {
                        font_size: 10.0,
                        ..default()
                    },
                    TextColor(colors::FG0),
                ));
            });

            // Spacer
            row.spawn(Node {
                flex_grow: 1.0,
                ..default()
            });

            // Save button (hidden until Ready)
            row.spawn((
                CardSaveButton { kind },
                Button,
                Node {
                    padding: UiRect::axes(Val::Px(8.0), Val::Px(3.0)),
                    border_radius: BorderRadius::all(Val::Px(3.0)),
                    display: Display::None,
                    ..default()
                },
                BackgroundColor(colors::BG3),
            ))
            .with_children(|btn| {
                btn.spawn((
                    Text::new("Save"),
                    TextFont {
                        font_size: 10.0,
                        ..default()
                    },
                    TextColor(colors::FG0),
                ));
            });
        });

        // Button row
        c.spawn(Node {
            flex_direction: FlexDirection::Row,
            padding: UiRect::axes(Val::Px(6.0), Val::Px(4.0)),
            column_gap: Val::Px(4.0),
            justify_content: JustifyContent::FlexEnd,
            ..default()
        })
        .with_children(|row| {
            // Generate button (hidden when Ready)
            row.spawn((
                CardGenerateButton { kind },
                Button,
                Node {
                    padding: UiRect::axes(Val::Px(8.0), Val::Px(3.0)),
                    border_radius: BorderRadius::all(Val::Px(3.0)),
                    ..default()
                },
                BackgroundColor(colors::BG3),
            ))
            .with_children(|btn| {
                btn.spawn((
                    Text::new("Generate"),
                    TextFont {
                        font_size: 10.0,
                        ..default()
                    },
                    TextColor(colors::FG0),
                ));
            });

            // Retry button (hidden by default)
            row.spawn((
                CardRetryButton { kind },
                Button,
                Node {
                    padding: UiRect::axes(Val::Px(8.0), Val::Px(3.0)),
                    display: Display::None,
                    border_radius: BorderRadius::all(Val::Px(3.0)),
                    ..default()
                },
                BackgroundColor(colors::RED),
            ))
            .with_children(|btn| {
                btn.spawn((
                    Text::new("Retry"),
                    TextFont {
                        font_size: 10.0,
                        ..default()
                    },
                    TextColor(colors::FG0),
                ));
            });
        });
    });

    card
}
