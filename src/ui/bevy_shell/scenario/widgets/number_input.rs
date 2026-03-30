//! NumberInput widget — click-to-type numeric field.

use bevy::{input::keyboard::KeyboardInput, prelude::*};

use super::{param_id::ParamId, slider::ParamValueDisplay, Disabled};
use crate::ui::colors;

// ── Components ────────────────────────────────────────────────────────────────

/// Marker on a number input node. Supports click-to-focus keyboard entry.
#[derive(Component, Debug, Clone)]
pub struct NumberInputWidget {
    pub param_id: ParamId,
    pub axis_index: usize, // 0=X, 1=Y, 2=Z for XYZ groups
    pub value: f32,
    /// When true, keyboard events are accepted for direct text entry.
    pub focused: bool,
    /// Accumulated text while in keyboard-entry mode.
    pub input_buffer: String,
}

/// Marker for the text display inside a number input.
#[derive(Component, Debug)]
pub struct NumberInputDisplay {
    pub param_id: ParamId,
    pub axis_index: usize,
}

// ── Spawn ─────────────────────────────────────────────────────────────────────

/// Spawns a number input (click-to-type) into the given parent.
#[tracing::instrument(skip_all)]
pub fn spawn_number_input(
    commands: &mut ChildSpawnerCommands,
    param_id: ParamId,
    axis_index: usize,
    value: f32,
    label: &str,
) {
    let initial_text = format_number_input_value(param_id, value);
    commands
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(2.0),
            ..default()
        })
        .with_children(|row| {
            // Axis label
            if !label.is_empty() {
                row.spawn((
                    Text::new(format!("{label}:")),
                    TextFont {
                        font_size: 10.0,
                        ..default()
                    },
                    TextColor(colors::GREY1),
                ));
            }
            // Input field
            row.spawn((
                NumberInputWidget {
                    param_id,
                    axis_index,
                    value,
                    focused: false,
                    input_buffer: String::new(),
                },
                Button,
                Node {
                    min_width: Val::Px(60.0),
                    height: Val::Px(20.0),
                    padding: UiRect::horizontal(Val::Px(4.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: UiRect::all(Val::Px(1.0)),
                    border_radius: BorderRadius::all(Val::Px(2.0)),
                    ..default()
                },
                BackgroundColor(colors::BG2),
                BorderColor::all(colors::BG3),
            ))
            .with_children(|btn| {
                btn.spawn((
                    NumberInputDisplay {
                        param_id,
                        axis_index,
                    },
                    Text::new(initial_text),
                    TextFont {
                        font_size: 11.0,
                        ..default()
                    },
                    TextColor(colors::FG0),
                ));
            });
        });
}

// ── Systems ───────────────────────────────────────────────────────────────────

/// Handles click-to-focus and keyboard entry on number inputs.
///
/// On commit (Enter / click-away), the per-axis display is updated and the
/// right-column [`ParamValueDisplay`] is rebuilt as a composite string so that
/// XYZ group rows always show all three values.
#[tracing::instrument(skip_all)]
pub fn handle_number_input(
    mouse: Res<ButtonInput<MouseButton>>,
    mut inputs: Query<
        (&mut NumberInputWidget, &Interaction, &mut BorderColor),
        (With<Button>, Without<Disabled>),
    >,
    mut displays: Query<(&NumberInputDisplay, &mut Text)>,
    mut param_displays: Query<(&ParamValueDisplay, &mut Text), Without<NumberInputDisplay>>,
    mut keyboard: MessageReader<KeyboardInput>,
) {
    // Collect keyboard events once.
    let key_events: Vec<KeyboardInput> = keyboard.read().cloned().collect();

    // Accumulate (param_id, axis_index, committed_value).
    let mut commits: Vec<(ParamId, usize, f32)> = Vec::new();

    // Click-to-focus / blur.
    if mouse.just_pressed(MouseButton::Left) {
        for (mut input, interaction, mut border_color) in &mut inputs {
            if *interaction == Interaction::Pressed {
                if !input.focused {
                    input.focused = true;
                    input.input_buffer = format_number_input_value(input.param_id, input.value);
                    *border_color = BorderColor::all(colors::ORANGE);
                }
            } else if input.focused {
                // Blur — commit whatever is in the buffer.
                if let Ok(v) = input.input_buffer.trim().parse::<f32>() {
                    input.value = v;
                    commits.push((input.param_id, input.axis_index, v));
                }
                input.focused = false;
                input.input_buffer.clear();
                *border_color = BorderColor::all(colors::BG3);
            }
        }
    }

    // Keyboard entry for focused fields.
    for (mut input, _, mut border_color) in &mut inputs {
        if !input.focused {
            continue;
        }
        for ev in &key_events {
            if ev.state != bevy::input::ButtonState::Pressed {
                continue;
            }
            match ev.key_code {
                KeyCode::Enter | KeyCode::NumpadEnter => {
                    if let Ok(v) = input.input_buffer.trim().parse::<f32>() {
                        input.value = v;
                        commits.push((input.param_id, input.axis_index, v));
                    }
                    input.focused = false;
                    input.input_buffer.clear();
                    *border_color = BorderColor::all(colors::BG3);
                }
                KeyCode::Escape => {
                    input.focused = false;
                    input.input_buffer.clear();
                    *border_color = BorderColor::all(colors::BG3);
                }
                KeyCode::Backspace => {
                    input.input_buffer.pop();
                }
                _ => {
                    if let Some(text) = &ev.text {
                        for ch in text.chars() {
                            if ch.is_ascii_digit() || ch == '.' || ch == '-' {
                                input.input_buffer.push(ch);
                            }
                        }
                    }
                }
            }
        }

        // Keep per-axis display in sync with buffer + cursor indicator.
        let param_id = input.param_id;
        let axis_index = input.axis_index;
        let buffer_text = format!("{}|", input.input_buffer);
        for (display, mut text) in &mut displays {
            if display.param_id == param_id && display.axis_index == axis_index {
                text.0 = buffer_text.clone();
            }
        }
    }

    // Apply committed values.
    for (param_id, axis_index, val) in &commits {
        // Update the individual axis display.
        let axis_text = format_number_input_value(*param_id, *val);
        for (display, mut text) in &mut displays {
            if display.param_id == *param_id && display.axis_index == *axis_index {
                text.0 = axis_text.clone();
            }
        }

        // Rebuild the composite right-column display by reading all current
        // axis values. Use NaN as sentinel for axes with no widget.
        let mut axis_vals = [f32::NAN; 3];
        let mut axis_count = 0usize;
        for (input, _, _) in inputs.iter() {
            if input.param_id == *param_id {
                let i = input.axis_index.min(2);
                axis_vals[i] = input.value;
                axis_count += 1;
            }
        }

        let display_str = if axis_count <= 1 {
            // Single-axis param (e.g. SensorRadius).
            format_number_input_value(*param_id, axis_vals[0])
        } else {
            // XYZ / XY group: rebuild composite string using per-value formatting.
            let fmt =
                |v: f32| format_number_input_value(*param_id, if v.is_nan() { 0.0 } else { v });
            let xs = fmt(axis_vals[0]);
            let ys = fmt(axis_vals[1]);
            if axis_vals[2].is_nan() {
                // 2-axis group (e.g. SA Node Center)
                format!("({xs},{ys})")
            } else {
                let zs = fmt(axis_vals[2]);
                format!("({xs},{ys},{zs})")
            }
        };

        for (d, mut text) in &mut param_displays {
            if d.param_id == *param_id {
                text.0 = display_str.clone();
            }
        }
    }
}

/// Updates number input displays when values change (e.g. after external sync).
#[tracing::instrument(skip_all)]
pub fn update_number_inputs(
    inputs: Query<&NumberInputWidget, Changed<NumberInputWidget>>,
    mut displays: Query<(&NumberInputDisplay, &mut Text)>,
) {
    for input in &inputs {
        if !input.focused {
            for (display, mut text) in &mut displays {
                if display.param_id == input.param_id && display.axis_index == input.axis_index {
                    text.0 = format_number_input_value(input.param_id, input.value);
                }
            }
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Returns the display string for a single number input value, respecting
/// integer-only and high-precision params.
#[tracing::instrument(level = "trace")]
pub(super) fn format_number_input_value(param_id: ParamId, value: f32) -> String {
    if param_id.is_integer_display() {
        format!("{value:.0}")
    } else if param_id.is_two_decimal() {
        format!("{value:.2}")
    } else {
        format!("{value:.1}")
    }
}
