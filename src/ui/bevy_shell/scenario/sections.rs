//! Collapsible section widget for the scenario editor.
//!
//! Each section has a clickable header (icon placeholder + title + chevron) and
//! a body container. The body `Display::Flex / None` is toggled via
//! `ScenarioViewState.section_collapsed`.

#![allow(clippy::type_complexity)]

use bevy::prelude::*;

use super::{ScenarioViewState, SectionId};
use crate::ui::colors;

// ── Components ────────────────────────────────────────────────────────────────

/// Marker for a section header node.
#[derive(Component, Debug, Clone)]
pub struct SectionHeader {
    pub section_id: SectionId,
}

/// Marker for the body container node of a section.
#[derive(Component, Debug, Clone)]
pub struct SectionBody {
    pub section_id: SectionId,
}

/// Marker for the chevron text node of a section header.
#[derive(Component, Debug, Clone)]
pub struct SectionChevron {
    pub section_id: SectionId,
}

// ── Spawn ─────────────────────────────────────────────────────────────────────

/// Spawns a collapsible section as a child of `parent`.
///
/// Returns the body entity so callers can add parameter rows into it.
#[tracing::instrument(skip_all)]
pub fn spawn_section(
    commands: &mut Commands,
    parent: Entity,
    section_id: SectionId,
    title: &str,
    view_state: &ScenarioViewState,
) -> Entity {
    let collapsed = view_state
        .section_collapsed
        .get(&section_id)
        .copied()
        .unwrap_or(true);

    let chevron = if collapsed { ">" } else { "v" };
    let body_display = if collapsed {
        Display::None
    } else {
        Display::Flex
    };

    // Section container (header + body)
    let section_container = commands
        .spawn((Node {
            flex_direction: FlexDirection::Column,
            width: Val::Percent(100.0),
            ..default()
        },))
        .id();
    commands.entity(parent).add_child(section_container);

    // Header row
    commands
        .entity(section_container)
        .with_children(|container| {
            container
                .spawn((
                    SectionHeader { section_id },
                    Button,
                    Node {
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        width: Val::Percent(100.0),
                        height: Val::Px(36.0),
                        padding: UiRect::horizontal(Val::Px(12.0)),
                        column_gap: Val::Px(8.0),
                        ..default()
                    },
                    BackgroundColor(colors::BG1),
                ))
                .with_children(|header| {
                    // Icon placeholder (colored rectangle)
                    header.spawn((
                        Node {
                            width: Val::Px(12.0),
                            height: Val::Px(12.0),
                            border_radius: BorderRadius::all(Val::Px(2.0)),
                            ..default()
                        },
                        BackgroundColor(colors::ORANGE),
                    ));

                    // Title text
                    header.spawn((
                        Text::new(title.to_string()),
                        TextFont {
                            font_size: 13.0,
                            ..default()
                        },
                        TextColor(colors::FG0),
                        Node {
                            flex_grow: 1.0,
                            ..default()
                        },
                    ));

                    // Chevron
                    header.spawn((
                        SectionChevron { section_id },
                        Text::new(chevron.to_string()),
                        TextFont {
                            font_size: 12.0,
                            ..default()
                        },
                        TextColor(colors::GREY1),
                    ));
                });
        });

    // Body container
    let body = commands
        .spawn((
            SectionBody { section_id },
            Node {
                flex_direction: FlexDirection::Column,
                width: Val::Percent(100.0),
                padding: UiRect::new(Val::Px(16.0), Val::Px(16.0), Val::Px(12.0), Val::Px(12.0)),
                row_gap: Val::Px(2.0),
                display: body_display,
                ..default()
            },
            BackgroundColor(colors::BG0),
        ))
        .id();
    commands.entity(section_container).add_child(body);

    body
}

// ── Click handler ─────────────────────────────────────────────────────────────

/// Toggles section collapse state when the header is clicked.
#[tracing::instrument(skip_all)]
pub fn handle_section_header_click(
    headers: Query<(&SectionHeader, &Interaction), (With<Button>, Changed<Interaction>)>,
    mut view_state: ResMut<ScenarioViewState>,
    mut bodies: Query<(&SectionBody, &mut Node)>,
    mut chevrons: Query<(&SectionChevron, &mut Text)>,
) {
    for (header, interaction) in &headers {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let section_id = header.section_id;
        let entry = view_state
            .section_collapsed
            .entry(section_id)
            .or_insert(false);
        *entry = !*entry;
        let collapsed = *entry;

        // Update body display
        for (body, mut node) in &mut bodies {
            if body.section_id == section_id {
                node.display = if collapsed {
                    Display::None
                } else {
                    Display::Flex
                };
            }
        }

        // Update chevron
        let chevron = if collapsed { ">" } else { "v" };
        for (ch, mut text) in &mut chevrons {
            if ch.section_id == section_id {
                text.0 = chevron.to_string();
            }
        }
    }
}
