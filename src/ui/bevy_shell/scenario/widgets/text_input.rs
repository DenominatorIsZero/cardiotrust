//! TextInput widget — single-line text entry field.

use bevy::{input::keyboard::KeyboardInput, prelude::*};

use super::{param_id::ParamId, slider::ParamValueDisplay, Disabled};
use crate::ui::colors;

// ── Components ────────────────────────────────────────────────────────────────

/// Marker on a single-line text input node.
#[derive(Component, Debug, Clone)]
pub struct TextInputWidget {
    pub param_id: ParamId,
    pub content: String,
    pub focused: bool,
}

/// Marker for the text display node inside a text input.
#[derive(Component, Debug)]
pub struct TextInputDisplay {
    pub param_id: ParamId,
}

// ── Spawn ─────────────────────────────────────────────────────────────────────

/// Spawns a single-line text input into the given parent.
#[tracing::instrument(skip_all)]
pub fn spawn_text_input(commands: &mut ChildSpawnerCommands, param_id: ParamId, initial: &str) {
    commands
        .spawn((
            TextInputWidget {
                param_id,
                content: initial.to_string(),
                focused: false,
            },
            Button,
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(24.0),
                padding: UiRect::horizontal(Val::Px(6.0)),
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
                TextInputDisplay { param_id },
                Text::new(initial.to_string()),
                TextFont {
                    font_size: 11.0,
                    ..default()
                },
                TextColor(colors::FG0),
            ));
        });
}

// ── Systems ───────────────────────────────────────────────────────────────────

/// Handles text input focus and keyboard entry.
#[tracing::instrument(skip_all)]
pub fn handle_text_input(
    mouse: Res<ButtonInput<MouseButton>>,
    mut keyboard: MessageReader<KeyboardInput>,
    mut inputs: Query<
        (&mut TextInputWidget, &Interaction, &mut BorderColor),
        (With<Button>, Without<Disabled>),
    >,
    mut displays: Query<(&TextInputDisplay, &mut Text)>,
    mut param_displays: Query<(&ParamValueDisplay, &mut Text), Without<TextInputDisplay>>,
) {
    // Click to focus
    if mouse.just_pressed(MouseButton::Left) {
        for (mut input, interaction, mut border_color) in &mut inputs {
            if *interaction == Interaction::Pressed {
                input.focused = true;
                *border_color = BorderColor::all(colors::ORANGE);
            } else if input.focused {
                // Commit on blur
                input.focused = false;
                *border_color = BorderColor::all(colors::BG3);
            }
        }
    }

    // Collect keyboard events
    let key_events: Vec<KeyboardInput> = keyboard.read().cloned().collect();

    for (mut input, _, mut border_color) in &mut inputs {
        if !input.focused {
            continue;
        }

        for ev in &key_events {
            if ev.state != bevy::input::ButtonState::Pressed {
                continue;
            }
            match ev.key_code {
                KeyCode::Escape => {
                    input.focused = false;
                    *border_color = BorderColor::all(colors::BG3);
                }
                KeyCode::Enter | KeyCode::NumpadEnter => {
                    input.focused = false;
                    *border_color = BorderColor::all(colors::BG3);
                    let param_id = input.param_id;
                    let val = input.content.clone();
                    for (d, mut text) in &mut param_displays {
                        if d.param_id == param_id {
                            text.0 = val.clone();
                        }
                    }
                }
                KeyCode::Backspace => {
                    input.content.pop();
                }
                _ => {
                    if let Some(text) = &ev.text {
                        for ch in text.chars() {
                            if !ch.is_control() {
                                input.content.push(ch);
                            }
                        }
                    }
                }
            }
        }

        if input.focused {
            // Update display with cursor indicator
            let param_id = input.param_id;
            let display_text = format!("{}|", input.content);
            for (d, mut text) in &mut displays {
                if d.param_id == param_id {
                    text.0 = display_text.clone();
                }
            }
        }
    }
}
