//! Checkbox widget — toggle boolean parameter.

use bevy::prelude::*;

use super::{param_id::ParamId, slider::ParamValueDisplay, Disabled};
use crate::ui::colors;

// ── Components ────────────────────────────────────────────────────────────────

/// Marker on the checkbox square node.
#[derive(Component, Debug, Clone)]
pub struct CheckboxWidget {
    pub param_id: ParamId,
    pub checked: bool,
}

/// Marker for the checkmark text inside the checkbox.
#[derive(Component, Debug)]
pub struct CheckboxMark {
    pub param_id: ParamId,
}

// ── Spawn ─────────────────────────────────────────────────────────────────────

/// Spawns a checkbox control into the given parent.
#[tracing::instrument(skip_all)]
pub fn spawn_checkbox(commands: &mut ChildSpawnerCommands, param_id: ParamId, checked: bool) {
    let mark = if checked { "x" } else { " " };
    commands
        .spawn((
            CheckboxWidget { param_id, checked },
            Button,
            Node {
                width: Val::Px(20.0),
                height: Val::Px(20.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(1.0)),
                border_radius: BorderRadius::all(Val::Px(3.0)),
                ..default()
            },
            BackgroundColor(colors::BG2),
            BorderColor::all(colors::BG3),
        ))
        .with_children(|btn| {
            btn.spawn((
                CheckboxMark { param_id },
                Text::new(mark.to_string()),
                TextFont {
                    font_size: 13.0,
                    ..default()
                },
                TextColor(colors::ORANGE),
            ));
        });
}

// ── Systems ───────────────────────────────────────────────────────────────────

/// Handles checkbox click to toggle state.
#[tracing::instrument(skip_all)]
pub fn handle_checkbox_click(
    mut checkboxes: Query<
        (&mut CheckboxWidget, &Interaction),
        (With<Button>, Changed<Interaction>, Without<Disabled>),
    >,
    mut marks: Query<(&CheckboxMark, &mut Text)>,
    mut param_displays: Query<(&ParamValueDisplay, &mut Text), Without<CheckboxMark>>,
) {
    // Collect toggled (param_id, new_checked) first to avoid borrow conflicts.
    let mut toggled: Vec<(ParamId, bool)> = Vec::new();

    for (mut cb, interaction) in &mut checkboxes {
        if *interaction != Interaction::Pressed {
            continue;
        }
        cb.checked = !cb.checked;
        toggled.push((cb.param_id, cb.checked));
    }

    for (param_id, checked) in toggled {
        let mark = if checked { "x" } else { " " };
        for (m, mut text) in &mut marks {
            if m.param_id == param_id {
                text.0 = mark.to_string();
            }
        }
        // Update right-column value display
        let val_str = if checked { "on" } else { "off" };
        for (d, mut text) in &mut param_displays {
            if d.param_id == param_id {
                text.0 = val_str.to_string();
            }
        }
    }
}
