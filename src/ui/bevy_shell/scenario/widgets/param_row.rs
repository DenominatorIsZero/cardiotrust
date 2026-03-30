//! Parameter row spawn helper and legacy `TooltipTarget`.

use bevy::prelude::*;

use super::{param_id::ParamId, slider::ParamValueDisplay, tooltip::TooltipButton};
use crate::ui::colors;

// ── Legacy component ──────────────────────────────────────────────────────────

/// Legacy component — kept so existing call sites compile.
#[derive(Component, Debug, Clone)]
pub struct TooltipTarget {
    pub text: String,
}

// ── Spawn ─────────────────────────────────────────────────────────────────────

/// Spawns a parameter row:
/// - label node with [?] tooltip button
/// - control slot node (flex grow)
/// - value+unit node (right-aligned, single line, clickable Button)
///
/// Returns `(control_slot, value_container)`.  
/// `control_slot` — attach the actual control widget here.  
/// `value_container` — a `Button` entity; attach [`SliderValueInput`] to enable
/// click-to-type exact number entry for slider rows.
#[tracing::instrument(skip_all)]
pub fn spawn_param_row(
    commands: &mut ChildSpawnerCommands,
    label: &str,
    unit: &str,
    tooltip_text: &str,
    initial_value_text: &str,
    param_id: ParamId,
) -> (Entity, Entity) {
    let mut control_slot = Entity::PLACEHOLDER;
    let mut value_container = Entity::PLACEHOLDER;

    commands
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            width: Val::Percent(100.0),
            min_height: Val::Px(30.0),
            padding: UiRect::axes(Val::Px(0.0), Val::Px(2.0)),
            column_gap: Val::Px(6.0),
            ..default()
        })
        .with_children(|row| {
            // Label (fixed width)
            row.spawn(Node {
                width: Val::Px(160.0),
                flex_shrink: 0.0,
                ..default()
            })
            .with_children(|label_node| {
                label_node.spawn((
                    Text::new(label.to_string()),
                    TextFont {
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor(colors::FG0),
                ));
            });

            // [?] tooltip button
            if !tooltip_text.is_empty() {
                row.spawn((
                    TooltipButton {
                        text: tooltip_text.to_string(),
                    },
                    Button,
                    Node {
                        width: Val::Px(16.0),
                        height: Val::Px(16.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(1.0)),
                        border_radius: BorderRadius::all(Val::Px(8.0)),
                        flex_shrink: 0.0,
                        margin: UiRect::right(Val::Px(4.0)),
                        ..default()
                    },
                    BackgroundColor(colors::BG3),
                    BorderColor::all(colors::GREY1),
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new("?"),
                        TextFont {
                            font_size: 9.0,
                            ..default()
                        },
                        TextColor(colors::GREY1),
                    ));
                });
            }

            // Control slot (flex-grow)
            let slot = row
                .spawn(Node {
                    flex_grow: 1.0,
                    align_items: AlignItems::Center,
                    ..default()
                })
                .id();
            control_slot = slot;

            // Value + unit (right-aligned, single line, no wrap).
            // Rendered as a Button so callers can attach SliderValueInput.
            let vc = row
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(90.0),
                        flex_shrink: 0.0,
                        justify_content: JustifyContent::FlexEnd,
                        align_items: AlignItems::Center,
                        overflow: Overflow::clip(),
                        border: UiRect::all(Val::Px(1.0)),
                        border_radius: BorderRadius::all(Val::Px(3.0)),
                        ..default()
                    },
                    BackgroundColor(Color::NONE),
                    BorderColor::all(Color::NONE),
                ))
                .with_children(|val_node| {
                    // Value text
                    val_node.spawn((
                        ParamValueDisplay { param_id },
                        Text::new(initial_value_text.to_string()),
                        TextFont {
                            font_size: 11.0,
                            ..default()
                        },
                        TextColor(colors::GREY1),
                    ));
                    // Unit text (if any) — on same line
                    if !unit.is_empty() {
                        val_node.spawn((
                            Text::new(format!(" {unit}")),
                            TextFont {
                                font_size: 10.0,
                                ..default()
                            },
                            TextColor(colors::GREY0),
                        ));
                    }
                })
                .id();
            value_container = vc;
        });

    (control_slot, value_container)
}
