//! Spawn helpers for scenario cards and their child nodes.

use bevy::prelude::*;

use super::components::{
    CardIdLabel, CardMatchHighlight, CardNameLabel, CardQuickAction, CardQuickActionKind,
    CardScenarioId, CardThumbnailArea, LabelSpanHighlight, LabelSpanPrefix, LabelSpanSuffix,
    NewScenarioActionCard, ScenarioCard,
};
use crate::{
    core::scenario::Status,
    ui::{bevy_shell::explorer::thumbnail::ThumbnailState, colors},
};

// ── Public spawn helpers ──────────────────────────────────────────────────────

/// Spawns a scenario card entity tree.
///
/// The card contains (top to bottom):
/// 1. Status badge (pill)
/// 2. Thumbnail area
/// 3. Metrics row (Done only)
/// 4. Title
/// 5. Comment (when non-empty)
/// 6. Timestamp
#[tracing::instrument(skip_all)]
pub fn spawn_card(
    commands: &mut Commands,
    scenario_id: &str,
    index: usize,
    status: &Status,
    title: &str,
    comment: &str,
    timestamp: Option<&str>,
    metrics: Option<(f32, f32)>, // (dice, loss)
    progress: Option<f32>,
    etc: Option<&str>,
    thumbnail: Option<&ThumbnailState>,
) -> Entity {
    let badge_color = status_badge_color(status);
    let badge_label = status_label(status);

    let card = commands
        .spawn((
            ScenarioCard {
                scenario_id: scenario_id.to_string(),
                index,
            },
            CardScenarioId(scenario_id.to_string()),
            Button,
            Node {
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(12.0)),
                row_gap: Val::Px(8.0),
                border: UiRect::all(Val::Px(2.0)),
                min_width: Val::Px(200.0),
                min_height: Val::Px(280.0),
                border_radius: BorderRadius::all(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(colors::BG1),
            BorderColor::all(Color::NONE),
        ))
        .with_children(|card| {
            // ── Status badge ──────────────────────────────────────────────────
            card.spawn((
                Node {
                    padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
                    align_self: AlignSelf::FlexStart,
                    border_radius: BorderRadius::all(Val::Px(12.0)),
                    ..default()
                },
                BackgroundColor(badge_color),
            ))
            .with_children(|badge| {
                badge.spawn((
                    Text::new(badge_label),
                    TextFont {
                        font_size: 11.0,
                        ..default()
                    },
                    TextColor(colors::BG0),
                ));
            });

            // ── Thumbnail area ────────────────────────────────────────────────
            spawn_thumbnail_area(card, scenario_id, status, progress, etc, thumbnail);

            // ── Metrics row (Done only) ───────────────────────────────────────
            if let Some((dice, loss)) = metrics {
                card.spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(12.0),
                    ..default()
                })
                .with_children(|row| {
                    row.spawn((
                        Text::new(format!("Dice: {dice:.3}")),
                        TextFont {
                            font_size: 11.0,
                            ..default()
                        },
                        TextColor(colors::GREEN),
                    ));
                    row.spawn((
                        Text::new(format!("Loss: {loss:.4}")),
                        TextFont {
                            font_size: 11.0,
                            ..default()
                        },
                        TextColor(colors::YELLOW),
                    ));
                });
            }

            // ── Primary display name (comment if set, else ID) ────────────────
            let display_name = if comment.is_empty() { title } else { comment };
            card.spawn((
                CardNameLabel { index },
                CardMatchHighlight::default(),
                Text::default(),
                TextFont {
                    font_size: 13.0,
                    ..default()
                },
                TextColor(colors::FG0),
            ))
            .with_children(|label| {
                // prefix span
                label.spawn((
                    LabelSpanPrefix,
                    TextSpan::new(truncate_str(display_name, 36)),
                    TextFont {
                        font_size: 13.0,
                        ..default()
                    },
                    TextColor(colors::FG0),
                ));
                // highlight span (empty initially)
                label.spawn((
                    LabelSpanHighlight,
                    TextSpan::new(String::new()),
                    TextFont {
                        font_size: 13.0,
                        ..default()
                    },
                    TextColor(colors::YELLOW),
                ));
                // suffix span (empty initially)
                label.spawn((
                    LabelSpanSuffix,
                    TextSpan::new(String::new()),
                    TextFont {
                        font_size: 13.0,
                        ..default()
                    },
                    TextColor(colors::FG0),
                ));
            });

            // ── Secondary ID (smaller, dimmed) ────────────────────────────────
            card.spawn((
                CardIdLabel { index },
                CardMatchHighlight::default(),
                Text::default(),
                TextFont {
                    font_size: 10.0,
                    ..default()
                },
                TextColor(colors::GREY1),
            ))
            .with_children(|label| {
                label.spawn((
                    LabelSpanPrefix,
                    TextSpan::new(truncate_str(title, 32)),
                    TextFont {
                        font_size: 10.0,
                        ..default()
                    },
                    TextColor(colors::GREY1),
                ));
                label.spawn((
                    LabelSpanHighlight,
                    TextSpan::new(String::new()),
                    TextFont {
                        font_size: 10.0,
                        ..default()
                    },
                    TextColor(colors::YELLOW),
                ));
                label.spawn((
                    LabelSpanSuffix,
                    TextSpan::new(String::new()),
                    TextFont {
                        font_size: 10.0,
                        ..default()
                    },
                    TextColor(colors::GREY1),
                ));
            });

            // ── Timestamp ─────────────────────────────────────────────────────
            if let Some(ts) = timestamp {
                card.spawn((
                    Text::new(ts.to_string()),
                    TextFont {
                        font_size: 10.0,
                        ..default()
                    },
                    TextColor(colors::GREY1),
                ));
            }

            // Spacer pushes the quick-action row to the bottom of the card.
            card.spawn(Node {
                flex_grow: 1.0,
                ..default()
            });

            // ── Quick-action row ──────────────────────────────────────────────
            spawn_quick_actions(card, index, status);
        })
        .id();

    card
}

/// Spawns the "New Scenario" dashed-border action card.
#[tracing::instrument(skip_all)]
pub fn spawn_new_scenario_action_card(commands: &mut Commands) -> Entity {
    commands
        .spawn((
            NewScenarioActionCard,
            Button,
            Node {
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(24.0)),
                border: UiRect::all(Val::Px(2.0)),
                row_gap: Val::Px(8.0),
                min_width: Val::Px(200.0),
                min_height: Val::Px(120.0),
                border_radius: BorderRadius::all(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(Color::NONE),
            BorderColor::all(colors::GREY1),
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new("+"),
                TextFont {
                    font_size: 28.0,
                    ..default()
                },
                TextColor(colors::GREY1),
            ));
            btn.spawn((
                Text::new("New Scenario"),
                TextFont {
                    font_size: 13.0,
                    ..default()
                },
                TextColor(colors::GREY1),
            ));
        })
        .id()
}

// ── Private helpers ───────────────────────────────────────────────────────────

/// Spawns the thumbnail area child node.
#[tracing::instrument(skip_all)]
fn spawn_thumbnail_area(
    parent: &mut ChildSpawnerCommands,
    scenario_id: &str,
    status: &Status,
    progress: Option<f32>,
    etc: Option<&str>,
    thumbnail: Option<&ThumbnailState>,
) {
    parent
        .spawn((
            CardThumbnailArea {
                scenario_id: scenario_id.to_string(),
            },
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(120.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                overflow: Overflow::clip(),
                border_radius: BorderRadius::all(Val::Px(4.0)),
                ..default()
            },
            BackgroundColor(colors::BG3),
        ))
        .with_children(|area| match (status, thumbnail) {
            (Status::Done, Some(ThumbnailState::Ready(handle))) => {
                area.spawn((
                    ImageNode::new(handle.clone()),
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                ));
            }
            (Status::Done, None | Some(ThumbnailState::Generating | ThumbnailState::Pending)) => {
                area.spawn((
                    Text::new("Loading..."),
                    TextFont {
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor(colors::GREY1),
                ));
            }
            (Status::Done, Some(ThumbnailState::Failed(msg))) => {
                area.spawn((
                    Text::new(format!("! {}", truncate_str(msg, 24))),
                    TextFont {
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor(colors::RED),
                ));
            }
            (Status::Running(_), _) => {
                let pct = progress.unwrap_or(0.0) * 100.0;
                let etc_text = etc.unwrap_or("ETC: ???");
                area.spawn(Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(4.0),
                    ..default()
                })
                .with_children(|col| {
                    col.spawn((
                        Text::new(format!("{pct:.0}%")),
                        TextFont {
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(colors::YELLOW),
                    ));
                    col.spawn((
                        Text::new(etc_text.to_string()),
                        TextFont {
                            font_size: 11.0,
                            ..default()
                        },
                        TextColor(colors::GREY1),
                    ));
                });
            }
            (Status::Planning | Status::Scheduled, _) => {
                area.spawn((
                    Text::new("[?]"),
                    TextFont {
                        font_size: 24.0,
                        ..default()
                    },
                    TextColor(colors::GREY1),
                ));
            }
            (Status::Aborted, _) => {
                area.spawn((
                    Text::new("[X]"),
                    TextFont {
                        font_size: 24.0,
                        ..default()
                    },
                    TextColor(colors::RED),
                ));
            }
            (Status::Simulating, _) => {
                area.spawn((
                    Text::new("Simulating..."),
                    TextFont {
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor(colors::BLUE),
                ));
            }
        });
}

/// Spawns the `[D] [C] [S]` quick-action button row at the bottom of a card.
#[tracing::instrument(skip_all)]
fn spawn_quick_actions(parent: &mut ChildSpawnerCommands, index: usize, status: &Status) {
    let show_schedule = matches!(status, Status::Planning | Status::Aborted);

    parent
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::FlexEnd,
            column_gap: Val::Px(4.0),
            width: Val::Percent(100.0),
            ..default()
        })
        .with_children(|row| {
            spawn_quick_button(row, "[D]", CardQuickActionKind::Delete, colors::RED, index);
            spawn_quick_button(row, "[C]", CardQuickActionKind::Copy, colors::GREY1, index);
            if show_schedule {
                spawn_quick_button(
                    row,
                    "[S]",
                    CardQuickActionKind::Schedule,
                    colors::BLUE,
                    index,
                );
            }
        });
}

/// Spawns a single quick-action button inside the row.
#[tracing::instrument(skip_all)]
fn spawn_quick_button(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    kind: CardQuickActionKind,
    text_color: Color,
    index: usize,
) {
    parent
        .spawn((
            CardQuickAction { index, kind },
            Button,
            Node {
                padding: UiRect::axes(Val::Px(6.0), Val::Px(2.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border_radius: BorderRadius::all(Val::Px(3.0)),
                ..default()
            },
            BackgroundColor(Color::NONE),
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new(label.to_string()),
                TextFont {
                    font_size: 11.0,
                    ..default()
                },
                TextColor(text_color),
            ));
        });
}

// ── Helpers ───────────────────────────────────────────────────────────────────

pub(super) const fn status_badge_color(status: &Status) -> Color {
    match status {
        Status::Done => colors::GREEN,
        Status::Running(_) => colors::YELLOW,
        Status::Scheduled | Status::Simulating => colors::BLUE,
        Status::Planning => colors::GREY1,
        Status::Aborted => colors::RED,
    }
}

#[tracing::instrument(level = "trace", skip_all)]
pub(super) fn status_label(status: &Status) -> String {
    match status {
        Status::Done => "Done".to_string(),
        Status::Running(_) => "Running".to_string(),
        Status::Scheduled => "Queued".to_string(),
        Status::Planning => "Planning".to_string(),
        Status::Aborted => "Failed".to_string(),
        Status::Simulating => "Simulating".to_string(),
    }
}

#[tracing::instrument(level = "trace", skip_all)]
pub(super) fn truncate_str(s: &str, max_chars: usize) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() > max_chars {
        let truncated: String = chars[..max_chars].iter().collect();
        format!("{truncated}…")
    } else {
        s.to_string()
    }
}
