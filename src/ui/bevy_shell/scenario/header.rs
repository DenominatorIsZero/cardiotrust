//! Scenario header bar — ID, status badge, model-type selector, action buttons.

#![allow(clippy::type_complexity, clippy::assigning_clones)]

pub mod actions;

pub use actions::{
    handle_comment_input, handle_copy_button, handle_delete_confirm, handle_delete_dismiss,
    handle_save_button, handle_schedule_button,
};
use bevy::prelude::*;

use crate::{
    core::scenario::{Scenario, Status},
    ui::colors,
    ScenarioList, SelectedSenario,
};

// ── Components ────────────────────────────────────────────────────────────────

/// Marker for the header bar root node.
#[derive(Component, Debug)]
pub struct ScenarioHeaderBar;

/// Marker for the status badge text.
#[derive(Component, Debug)]
pub struct HeaderStatusBadge;

/// Marker for the scenario ID text in the header.
#[derive(Component, Debug)]
pub struct HeaderScenarioId;

/// Marker for the Save button.
#[derive(Component, Debug)]
pub struct HeaderSaveButton;

/// Marker for the Schedule/Unschedule button.
#[derive(Component, Debug)]
pub struct HeaderScheduleButton;

/// Marker for the Copy button.
#[derive(Component, Debug)]
pub struct HeaderCopyButton;

/// Marker for the Delete button.
#[derive(Component, Debug)]
pub struct HeaderDeleteButton;

/// Marker for the comment text input display node.
#[derive(Component, Debug)]
pub struct HeaderCommentDisplay;

/// State for the editable comment field.
#[derive(Component, Debug, Clone)]
pub struct CommentInputState {
    pub content: String,
    pub focused: bool,
}

/// Marker for the delete confirmation modal overlay.
#[derive(Component, Debug)]
pub struct DeleteConfirmModal;

/// Marker for the confirm button inside the delete modal.
#[derive(Component, Debug)]
pub struct DeleteConfirmOkButton;

/// Marker for the dismiss button inside the delete modal.
#[derive(Component, Debug)]
pub struct DeleteConfirmCancelButton;

// ── Spawn ─────────────────────────────────────────────────────────────────────

/// Spawns the scenario header bar as a child of `parent`.
#[tracing::instrument(skip_all)]
pub fn spawn_header_bar(commands: &mut Commands, parent: Entity, scenario: &Scenario) {
    let status = scenario.get_status();
    let is_planning = matches!(status, Status::Planning);
    let badge_color = status_badge_color(status);
    let badge_label = status_label(status);
    let schedule_label = schedule_button_label(status);
    let comment = scenario.comment.clone();
    let scenario_id = scenario.get_id().clone();

    let header = commands
        .spawn((
            ScenarioHeaderBar,
            Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::FlexStart,
                width: Val::Percent(100.0),
                min_height: Val::Px(48.0),
                padding: UiRect::axes(Val::Px(12.0), Val::Px(8.0)),
                column_gap: Val::Px(8.0),
                flex_shrink: 0.0,
                flex_wrap: FlexWrap::NoWrap,
                ..default()
            },
            BackgroundColor(colors::BG1),
        ))
        .id();
    commands.entity(parent).add_child(header);

    commands.entity(header).with_children(|bar| {
        // ── Left section: ID + status badge ──────────────────────────────────

        // Scenario ID — no truncation, use overflow clip on the container
        bar.spawn(Node {
            max_width: Val::Px(320.0),
            overflow: Overflow::clip(),
            flex_shrink: 0.0,
            align_self: AlignSelf::Center,
            ..default()
        })
        .with_children(|id_wrap| {
            id_wrap.spawn((
                HeaderScenarioId,
                Text::new(scenario_id.clone()),
                TextFont {
                    font_size: 13.0,
                    ..default()
                },
                TextColor(colors::FG0),
            ));
        });

        // Status badge pill
        bar.spawn((
            Node {
                padding: UiRect::axes(Val::Px(8.0), Val::Px(3.0)),
                border_radius: BorderRadius::all(Val::Px(12.0)),
                align_items: AlignItems::Center,
                align_self: AlignSelf::Center,
                flex_shrink: 0.0,
                ..default()
            },
            BackgroundColor(badge_color),
        ))
        .with_children(|badge| {
            badge.spawn((
                HeaderStatusBadge,
                Text::new(badge_label),
                TextFont {
                    font_size: 11.0,
                    ..default()
                },
                TextColor(colors::BG0),
            ));
        });

        // Spacer
        bar.spawn(Node {
            flex_grow: 1.0,
            ..default()
        });

        // ── Right section: comment field + action buttons ─────────────────────

        // Comment field — auto-height so it grows with the text (up to 100 chars).
        // overflow: clip() prevents a single long word from stretching past max_width.
        // The header bar has no fixed height so it expands vertically with this field.
        bar.spawn((
            CommentInputState {
                content: comment.clone(),
                focused: false,
            },
            HeaderCommentDisplay,
            Button,
            Node {
                min_width: Val::Px(140.0),
                max_width: Val::Px(280.0),
                min_height: Val::Px(26.0),
                flex_shrink: 1.0,
                padding: UiRect::all(Val::Px(6.0)),
                align_items: AlignItems::FlexStart,
                align_self: AlignSelf::Center,
                border: UiRect::all(Val::Px(1.0)),
                border_radius: BorderRadius::all(Val::Px(3.0)),
                overflow: Overflow::clip(),
                ..default()
            },
            BackgroundColor(colors::BG0),
            BorderColor::all(colors::BG3),
        ))
        .with_children(|field| {
            let display_comment = if comment.is_empty() {
                "add comment…".to_string()
            } else {
                comment.clone()
            };
            field.spawn((
                Text::new(display_comment),
                TextFont {
                    font_size: 11.0,
                    ..default()
                },
                TextColor(if comment.is_empty() {
                    colors::GREY0
                } else {
                    colors::FG0
                }),
            ));
        });

        // Save button (AQUA, hidden when not Planning)
        bar.spawn((
            HeaderSaveButton,
            Button,
            Node {
                padding: UiRect::axes(Val::Px(10.0), Val::Px(4.0)),
                border_radius: BorderRadius::all(Val::Px(4.0)),
                align_items: AlignItems::Center,
                align_self: AlignSelf::Center,
                flex_shrink: 0.0,
                display: if is_planning {
                    Display::Flex
                } else {
                    Display::None
                },
                ..default()
            },
            BackgroundColor(colors::AQUA),
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new("Save"),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(colors::BG0),
            ));
        });

        // Schedule / Unschedule button
        let schedule_visible = !matches!(
            status,
            Status::Running(_) | Status::Done | Status::Simulating
        );
        let schedule_color = match status {
            Status::Scheduled => colors::YELLOW,
            _ => colors::ORANGE,
        };
        bar.spawn((
            HeaderScheduleButton,
            Button,
            Node {
                padding: UiRect::axes(Val::Px(10.0), Val::Px(4.0)),
                border_radius: BorderRadius::all(Val::Px(4.0)),
                align_items: AlignItems::Center,
                align_self: AlignSelf::Center,
                flex_shrink: 0.0,
                display: if schedule_visible {
                    Display::Flex
                } else {
                    Display::None
                },
                ..default()
            },
            BackgroundColor(schedule_color),
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new(schedule_label),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(colors::BG0),
            ));
        });

        // Copy button
        bar.spawn((
            HeaderCopyButton,
            Button,
            Node {
                padding: UiRect::axes(Val::Px(10.0), Val::Px(4.0)),
                border_radius: BorderRadius::all(Val::Px(4.0)),
                align_items: AlignItems::Center,
                align_self: AlignSelf::Center,
                border: UiRect::all(Val::Px(1.0)),
                flex_shrink: 0.0,
                ..default()
            },
            BackgroundColor(colors::BG0),
            BorderColor::all(colors::GREY1),
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new("Copy"),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(colors::FG0),
            ));
        });

        // Delete button (RED)
        bar.spawn((
            HeaderDeleteButton,
            Button,
            Node {
                padding: UiRect::axes(Val::Px(10.0), Val::Px(4.0)),
                border_radius: BorderRadius::all(Val::Px(4.0)),
                align_items: AlignItems::Center,
                align_self: AlignSelf::Center,
                flex_shrink: 0.0,
                ..default()
            },
            BackgroundColor(colors::RED),
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new("Delete"),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(colors::BG0),
            ));
        });
    });
}

// ── Update ────────────────────────────────────────────────────────────────────

/// Reactively updates header bar contents when scenario status changes.
#[tracing::instrument(skip_all)]
pub fn update_header_bar(
    scenario_list: Res<ScenarioList>,
    selected: Res<SelectedSenario>,
    mut badges: Query<&mut Text, With<HeaderStatusBadge>>,
    mut id_texts: Query<&mut Text, (With<HeaderScenarioId>, Without<HeaderStatusBadge>)>,
    mut save_buttons: Query<&mut Node, With<HeaderSaveButton>>,
) {
    if !scenario_list.is_changed() && !selected.is_changed() {
        return;
    }
    let Some(index) = selected.index else {
        return;
    };
    let Some(entry) = scenario_list.entries.get(index) else {
        return;
    };
    let scenario = &entry.scenario;
    let is_planning = matches!(scenario.get_status(), Status::Planning);

    for mut text in &mut badges {
        text.0 = status_label(scenario.get_status());
    }
    for mut text in &mut id_texts {
        text.0 = scenario.get_id().clone();
    }
    for mut node in &mut save_buttons {
        node.display = if is_planning {
            Display::Flex
        } else {
            Display::None
        };
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

const fn status_badge_color(status: &Status) -> Color {
    match status {
        Status::Done => colors::GREEN,
        Status::Running(_) => colors::YELLOW,
        Status::Scheduled | Status::Simulating => colors::BLUE,
        Status::Planning => colors::GREY1,
        Status::Aborted => colors::RED,
    }
}

#[tracing::instrument(level = "trace", skip_all)]
fn status_label(status: &Status) -> String {
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
fn schedule_button_label(status: &Status) -> String {
    match status {
        Status::Planning | Status::Aborted => "Schedule".to_string(),
        Status::Scheduled => "Unschedule".to_string(),
        _ => String::new(),
    }
}
