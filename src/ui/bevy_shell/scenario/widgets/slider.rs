//! Slider widget — drag-to-adjust and click-to-type numeric input.

use bevy::{input::keyboard::KeyboardInput, prelude::*, ui::UiGlobalTransform};

use super::param_id::ParamId;
use crate::ui::colors;

// ── Components ────────────────────────────────────────────────────────────────

/// Attached to the right-column value container of a slider row to enable
/// click-to-type exact number entry.
#[derive(Component, Debug, Clone)]
pub struct SliderValueInput {
    pub param_id: ParamId,
    pub min: f32,
    pub max: f32,
    pub log_scale: bool,
    /// True while the user is typing a value.
    pub focused: bool,
    /// Accumulated text while typing.
    pub input_buffer: String,
}

/// Marker on the slider track root node, carrying config-path context.
#[derive(Component, Debug, Clone)]
pub struct SliderWidget {
    pub min: f32,
    pub max: f32,
    pub current: f32,
    pub param_id: ParamId,
    pub log_scale: bool,
    pub dragging: bool,
}

/// Marker on the filled portion of a slider track.
#[derive(Component, Debug)]
pub struct SliderFill {
    pub param_id: ParamId,
}

/// Marker on the slider thumb.
#[derive(Component, Debug)]
pub struct SliderThumb {
    pub param_id: ParamId,
}

/// Marker on the value display text node.
#[derive(Component, Debug)]
pub struct ParamValueDisplay {
    pub param_id: ParamId,
}

use super::Disabled;

// ── Spawn ─────────────────────────────────────────────────────────────────────

/// Spawns a horizontal slider control into the given parent.
#[tracing::instrument(skip_all)]
pub fn spawn_slider(
    commands: &mut ChildSpawnerCommands,
    param_id: ParamId,
    min: f32,
    max: f32,
    current: f32,
    log_scale: bool,
) {
    let fill_pct = value_to_pct(min, max, current, log_scale);

    commands
        .spawn((
            SliderWidget {
                min,
                max,
                current,
                param_id,
                log_scale,
                dragging: false,
            },
            Button,
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(20.0),
                align_items: AlignItems::Center,
                border_radius: BorderRadius::all(Val::Px(3.0)),
                overflow: Overflow::visible(),
                ..default()
            },
            BackgroundColor(colors::BG3),
            ZIndex(1),
        ))
        .with_children(|slider| {
            // Filled portion
            slider.spawn((
                SliderFill { param_id },
                Node {
                    width: Val::Percent(fill_pct * 100.0),
                    height: Val::Percent(100.0),
                    border_radius: BorderRadius::all(Val::Px(3.0)),
                    ..default()
                },
                BackgroundColor(colors::ORANGE),
            ));

            // Thumb
            slider.spawn((
                SliderThumb { param_id },
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Percent(fill_pct * 100.0),
                    width: Val::Px(14.0),
                    height: Val::Px(14.0),
                    border_radius: BorderRadius::all(Val::Px(7.0)),
                    margin: UiRect::left(Val::Px(-7.0)),
                    ..default()
                },
                BackgroundColor(colors::FG0),
                ZIndex(2),
            ));
        });
}

// ── Systems ───────────────────────────────────────────────────────────────────

/// Handles click-to-focus and keyboard entry on slider value display boxes.
///
/// When the user clicks the right-column value text, it turns orange-bordered
/// and accepts keyboard input. Enter/blur commits the value (clamped to
/// `[min, max]`) back to the matching [`SliderWidget`].
#[tracing::instrument(skip_all)]
pub fn handle_slider_value_input(
    mouse: Res<ButtonInput<MouseButton>>,
    mut keyboard: MessageReader<KeyboardInput>,
    mut inputs: Query<
        (
            &mut SliderValueInput,
            &Interaction,
            &mut BorderColor,
            &mut BackgroundColor,
        ),
        (With<Button>, Without<Disabled>),
    >,
    mut sliders: Query<&mut SliderWidget>,
    mut value_texts: Query<(&ParamValueDisplay, &mut Text)>,
) {
    let key_events: Vec<KeyboardInput> = keyboard.read().cloned().collect();

    // Focus / blur on click.
    if mouse.just_pressed(MouseButton::Left) {
        for (mut input, interaction, mut border, mut bg) in &mut inputs {
            if *interaction == Interaction::Pressed {
                if !input.focused {
                    // Seed buffer with current displayed value (no unit).
                    let current_text = value_texts
                        .iter()
                        .find(|(d, _)| d.param_id == input.param_id)
                        .map(|(_, t)| t.0.clone())
                        .unwrap_or_default();
                    input.input_buffer = current_text;
                    input.focused = true;
                    *border = BorderColor::all(colors::ORANGE);
                    *bg = BackgroundColor(colors::BG2);
                }
            } else if input.focused {
                // Blur without committing — restore display.
                input.focused = false;
                input.input_buffer.clear();
                *border = BorderColor::all(Color::NONE);
                *bg = BackgroundColor(Color::NONE);
            }
        }
    }

    // Collect updates to apply after the mutable borrow of `inputs`.
    let mut commits: Vec<(ParamId, f32)> = Vec::new();

    for (mut input, _, mut border, mut bg) in &mut inputs {
        if !input.focused {
            continue;
        }

        let param_id = input.param_id;

        for ev in &key_events {
            if ev.state != bevy::input::ButtonState::Pressed {
                continue;
            }
            match ev.key_code {
                KeyCode::Enter | KeyCode::NumpadEnter => {
                    if let Ok(v) = input.input_buffer.trim().parse::<f32>() {
                        let clamped = v.clamp(input.min, input.max);
                        commits.push((param_id, clamped));
                    }
                    input.focused = false;
                    input.input_buffer.clear();
                    *border = BorderColor::all(Color::NONE);
                    *bg = BackgroundColor(Color::NONE);
                }
                KeyCode::Escape => {
                    input.focused = false;
                    input.input_buffer.clear();
                    *border = BorderColor::all(Color::NONE);
                    *bg = BackgroundColor(Color::NONE);
                }
                KeyCode::Backspace => {
                    input.input_buffer.pop();
                }
                _ => {
                    if let Some(text) = &ev.text {
                        for ch in text.chars() {
                            if ch.is_ascii_digit() || ch == '.' || ch == '-' || ch == 'e' {
                                input.input_buffer.push(ch);
                            }
                        }
                    }
                }
            }
        }

        // Update display to show buffer + cursor while focused.
        if input.focused {
            let buffer_text = format!("{}|", input.input_buffer);
            for (display, mut text) in &mut value_texts {
                if display.param_id == param_id {
                    text.0 = buffer_text.clone();
                }
            }
        }
    }

    // Apply committed values to matching SliderWidgets and displays.
    for (param_id, clamped) in commits {
        for mut slider in &mut sliders {
            if slider.param_id == param_id {
                slider.current = clamped;
            }
        }
        for (display, mut text) in &mut value_texts {
            if display.param_id == param_id {
                text.0 = format_param_value(param_id, clamped);
            }
        }
    }
}

/// Updates slider fill widths and thumb positions to match current values.
#[tracing::instrument(skip_all)]
pub fn update_slider_fills(
    sliders: Query<&SliderWidget, Changed<SliderWidget>>,
    mut fills: Query<(&SliderFill, &mut Node), Without<SliderThumb>>,
    mut thumbs: Query<(&SliderThumb, &mut Node), Without<SliderFill>>,
) {
    for slider in &sliders {
        let fill_pct = value_to_pct(slider.min, slider.max, slider.current, slider.log_scale);
        for (fill, mut node) in &mut fills {
            if fill.param_id == slider.param_id {
                node.width = Val::Percent(fill_pct * 100.0);
            }
        }
        for (thumb, mut node) in &mut thumbs {
            if thumb.param_id == slider.param_id {
                node.left = Val::Percent(fill_pct * 100.0);
            }
        }
    }
}

/// Handles slider drag interaction.
///
/// Uses `ComputedNode::contains_point` with `UiGlobalTransform` — the correct
/// Bevy 0.18 UI hit-test API — on mouse-down to start a drag, then continues
/// dragging while the mouse is held regardless of where the cursor moves.
///
/// **Coordinate note**: `ComputedNode.size()` and `UiGlobalTransform` work in
/// *physical* pixels. `Window::cursor_position()` returns *logical* pixels.
/// We scale by `window.scale_factor()` before calling `contains_point`.
///
/// For the drag value computation we also need the left edge of the slider in
/// physical pixels. `UiGlobalTransform::affine().to_scale_angle_translation()`
/// gives `(scale, angle, translation)` where `translation` is the **top-left
/// corner** of the node in physical pixels.
#[tracing::instrument(skip_all)]
pub fn handle_slider_drag(
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    mut sliders: Query<(&mut SliderWidget, &UiGlobalTransform, &ComputedNode), Without<Disabled>>,
    mut param_displays: Query<(&ParamValueDisplay, &mut Text)>,
) {
    let Ok(window) = windows.single() else {
        return;
    };

    // On mouse-up, clear all drag states and exit.
    if mouse.just_released(MouseButton::Left) {
        for (mut slider, _, _) in &mut sliders {
            slider.dragging = false;
        }
        return;
    }

    let Some(logical_cursor) = window.cursor_position() else {
        return;
    };

    // Physical-pixel cursor — the coordinate system used by UiGlobalTransform.
    let scale = window.scale_factor();
    let cursor = logical_cursor * scale;

    // On mouse-down: use contains_point to decide which slider to drag.
    if mouse.just_pressed(MouseButton::Left) {
        for (mut slider, ui_transform, computed) in &mut sliders {
            slider.dragging = computed.contains_point(*ui_transform, cursor);
        }
    }

    if !mouse.pressed(MouseButton::Left) {
        return;
    }

    let mut updates: Vec<(ParamId, f32)> = Vec::new();

    for (mut slider, ui_transform, computed) in &mut sliders {
        if !slider.dragging {
            continue;
        }
        let size = computed.size();
        if size.x < 1.0 {
            continue;
        }
        // The inverse UiGlobalTransform maps a physical-pixel screen point into
        // node-local space where (0,0) is the node center and corners are at
        // (±size/2, ±size/2). Adding size.x/2 gives position from the left edge.
        let Some(inv) = ui_transform.try_inverse() else {
            continue;
        };
        let local = inv.transform_point2(cursor);
        let local_x = (local.x + size.x / 2.0).clamp(0.0, size.x);
        let t = local_x / size.x;
        let new_value = pct_to_value(slider.min, slider.max, t, slider.log_scale);
        if (slider.current - new_value).abs() > f32::EPSILON {
            slider.current = new_value;
            updates.push((slider.param_id, new_value));
        }
    }

    for (param_id, new_value) in updates {
        for (display, mut text) in &mut param_displays {
            if display.param_id == param_id {
                text.0 = format_param_value(param_id, new_value);
            }
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

#[tracing::instrument(level = "trace")]
fn value_to_pct(min: f32, max: f32, value: f32, log_scale: bool) -> f32 {
    if log_scale {
        let log_min = min.ln();
        let log_max = max.ln();
        let log_val = value.clamp(min, max).ln();
        ((log_val - log_min) / (log_max - log_min)).clamp(0.0, 1.0)
    } else {
        ((value - min) / (max - min)).clamp(0.0, 1.0)
    }
}

#[tracing::instrument(level = "trace")]
fn pct_to_value(min: f32, max: f32, t: f32, log_scale: bool) -> f32 {
    if log_scale {
        let log_min = min.ln();
        let log_max = max.ln();
        t.mul_add(log_max - log_min, log_min).exp().clamp(min, max)
    } else {
        t.mul_add(max - min, min).clamp(min, max)
    }
}

/// Formats a parameter value for display in the right column.
#[tracing::instrument(level = "trace")]
pub(super) fn format_param_value(param_id: ParamId, value: f32) -> String {
    match param_id {
        ParamId::SampleRate
        | ParamId::Epochs
        | ParamId::BatchSize
        | ParamId::NumberOfSensors
        | ParamId::LrReductionInterval
        | ParamId::SnapshotInterval => format!("{value:.0}"),
        ParamId::CovarianceMean => format!("{value:.2e}"),
        ParamId::LearningRate => format!("{value:.4}"),
        _ => format!("{value:.3}"),
    }
}
