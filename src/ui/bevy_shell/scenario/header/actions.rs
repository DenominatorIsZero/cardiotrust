use bevy::{input::keyboard::KeyboardInput, prelude::*};
use tracing::warn;

use super::{
    CommentInputState, DeleteConfirmCancelButton, DeleteConfirmModal, DeleteConfirmOkButton,
    HeaderCommentDisplay, HeaderCopyButton, HeaderDeleteButton, HeaderSaveButton,
    HeaderScheduleButton,
};
use crate::{
    core::scenario::{Scenario, Status},
    ui::{bevy_shell::scenario::ScenarioViewRoot, colors},
    ScenarioList, SelectedSenario,
};

/// Handles keyboard input for the comment field.
#[tracing::instrument(skip_all)]
pub fn handle_comment_input(
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut keyboard: MessageReader<KeyboardInput>,
    mut comment_fields: Query<
        (
            &mut CommentInputState,
            &Interaction,
            &mut BorderColor,
            &Children,
        ),
        With<HeaderCommentDisplay>,
    >,
    mut texts: Query<(&mut Text, &mut TextColor)>,
    mut scenario_list: ResMut<ScenarioList>,
    selected: Res<SelectedSenario>,
) {
    let shift = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);

    // Handle focus/unfocus on mouse click
    if mouse.just_pressed(MouseButton::Left) {
        for (mut state, interaction, mut border_color, _) in &mut comment_fields {
            if *interaction == Interaction::Pressed {
                if !state.focused {
                    state.focused = true;
                    *border_color = BorderColor::all(colors::ORANGE);
                }
            } else if state.focused {
                // Blur — commit to scenario
                commit_comment(&state.content.clone(), &mut scenario_list, &selected);
                state.focused = false;
                *border_color = BorderColor::all(colors::BG3);
            }
        }
    }

    // Collect keyboard events
    let key_events: Vec<KeyboardInput> = keyboard.read().cloned().collect();

    for (mut state, _, mut border_color, children) in &mut comment_fields {
        if !state.focused {
            continue;
        }

        for ev in &key_events {
            if ev.state != bevy::input::ButtonState::Pressed {
                continue;
            }
            match ev.key_code {
                KeyCode::Escape => {
                    // Cancel — revert from scenario
                    if let Some(index) = selected.index {
                        if let Some(entry) = scenario_list.entries.get(index) {
                            state.content = entry.scenario.comment.clone();
                        }
                    }
                    state.focused = false;
                    *border_color = BorderColor::all(colors::BG3);
                }
                KeyCode::Enter | KeyCode::NumpadEnter => {
                    if shift {
                        // Shift+Enter — insert a newline (if under the cap).
                        if state.content.len() < 100 {
                            state.content.push('\n');
                        }
                    } else {
                        // Plain Enter — commit and close.
                        commit_comment(&state.content.clone(), &mut scenario_list, &selected);
                        state.focused = false;
                        *border_color = BorderColor::all(colors::BG3);
                    }
                }
                KeyCode::Backspace => {
                    state.content.pop();
                }
                _ => {
                    if let Some(text) = &ev.text {
                        for ch in text.chars() {
                            if !ch.is_control() && state.content.len() < 100 {
                                state.content.push(ch);
                            }
                        }
                    }
                }
            }
        }

        // Update display text
        let display_text = if state.focused {
            format!("{}|", state.content)
        } else if state.content.is_empty() {
            "add comment…".to_string()
        } else {
            state.content.clone()
        };
        let is_placeholder = !state.focused && state.content.is_empty();

        for child in children.iter() {
            if let Ok((mut text, mut color)) = texts.get_mut(child) {
                text.0 = display_text.clone();
                color.0 = if is_placeholder {
                    colors::GREY0
                } else {
                    colors::FG0
                };
            }
        }
    }
}

/// Commits comment content to the scenario.
#[tracing::instrument(skip_all)]
fn commit_comment(
    content: &str,
    scenario_list: &mut ResMut<ScenarioList>,
    selected: &Res<SelectedSenario>,
) {
    let Some(index) = selected.index else {
        return;
    };
    let Some(entry) = scenario_list.entries.get_mut(index) else {
        return;
    };
    entry.scenario.comment = content.to_string();
}

// ── Button handlers ───────────────────────────────────────────────────────────

/// Handles Save button click.
#[tracing::instrument(skip_all)]
pub fn handle_save_button(
    save_btns: Query<&Interaction, (With<HeaderSaveButton>, With<Button>, Changed<Interaction>)>,
    mut scenario_list: ResMut<ScenarioList>,
    selected: Res<SelectedSenario>,
) {
    for interaction in &save_btns {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let Some(index) = selected.index else {
            continue;
        };
        let Some(entry) = scenario_list.entries.get_mut(index) else {
            continue;
        };
        if let Err(e) = entry.scenario.save() {
            warn!("Failed to save scenario: {e}");
        }
    }
}

/// Handles Schedule / Unschedule button click.
#[tracing::instrument(skip_all)]
pub fn handle_schedule_button(
    btns: Query<
        &Interaction,
        (
            With<HeaderScheduleButton>,
            With<Button>,
            Changed<Interaction>,
        ),
    >,
    mut scenario_list: ResMut<ScenarioList>,
    selected: Res<SelectedSenario>,
) {
    for interaction in &btns {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let Some(index) = selected.index else {
            continue;
        };
        let Some(entry) = scenario_list.entries.get_mut(index) else {
            continue;
        };
        let scenario = &mut entry.scenario;
        match scenario.get_status() {
            Status::Planning => {
                if let Err(e) = scenario.schedule() {
                    warn!("Failed to schedule scenario: {e}");
                }
            }
            Status::Scheduled => {
                if let Err(e) = scenario.unschedule() {
                    warn!("Failed to unschedule scenario: {e}");
                }
            }
            _ => {}
        }
    }
}

/// Handles Copy button click.
#[tracing::instrument(skip_all)]
pub fn handle_copy_button(
    btns: Query<&Interaction, (With<HeaderCopyButton>, With<Button>, Changed<Interaction>)>,
    mut scenario_list: ResMut<ScenarioList>,
    mut selected: ResMut<SelectedSenario>,
) {
    for interaction in &btns {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let Some(index) = selected.index else {
            continue;
        };
        let Some(entry) = scenario_list.entries.get(index) else {
            continue;
        };
        let config = entry.scenario.config.clone();
        let comment = entry.scenario.comment.clone();

        match Scenario::build(None) {
            Ok(mut new_scenario) => {
                new_scenario.config = config;
                new_scenario.comment = format!(
                    "Copy of {}",
                    comment.trim().trim_start_matches("Copy of ").trim()
                );
                if let Err(e) = new_scenario.save() {
                    warn!("Failed to save copied scenario: {e}");
                }
                let new_index = scenario_list.entries.len();
                scenario_list.entries.push(crate::ScenarioBundle {
                    scenario: new_scenario,
                    join_handle: None,
                    epoch_rx: None,
                    summary_rx: None,
                });
                selected.index = Some(new_index);
            }
            Err(e) => {
                warn!("Failed to create copy of scenario: {e}");
            }
        }
    }
}

/// Handles the Delete button — shows the confirmation modal.
#[tracing::instrument(skip_all)]
pub fn handle_delete_confirm(
    delete_btns: Query<
        &Interaction,
        (With<HeaderDeleteButton>, With<Button>, Changed<Interaction>),
    >,
    confirm_btns: Query<
        &Interaction,
        (
            With<DeleteConfirmOkButton>,
            With<Button>,
            Changed<Interaction>,
        ),
    >,
    mut commands: Commands,
    roots: Query<Entity, With<ScenarioViewRoot>>,
    modals: Query<Entity, With<DeleteConfirmModal>>,
    mut scenario_list: ResMut<ScenarioList>,
    mut selected: ResMut<SelectedSenario>,
    mut next_state: ResMut<NextState<crate::ui::UiState>>,
) {
    // Delete button → show modal
    for interaction in &delete_btns {
        if *interaction != Interaction::Pressed {
            continue;
        }
        // Only spawn modal if not already present
        if modals.iter().count() > 0 {
            continue;
        }
        let Ok(root) = roots.single() else {
            continue;
        };
        spawn_delete_modal(&mut commands, root);
    }

    // Confirm button inside modal → delete and navigate away
    for interaction in &confirm_btns {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let Some(index) = selected.index else {
            continue;
        };
        if let Some(entry) = scenario_list.entries.get(index) {
            if matches!(entry.scenario.get_status(), Status::Planning) {
                if let Err(e) = entry.scenario.delete() {
                    warn!("Failed to delete scenario from disk: {e}");
                }
                scenario_list.entries.remove(index);
                if index >= scenario_list.entries.len() {
                    selected.index = if scenario_list.entries.is_empty() {
                        None
                    } else {
                        Some(scenario_list.entries.len() - 1)
                    };
                }
                next_state.set(crate::ui::UiState::Explorer);
            } else {
                warn!("Cannot delete non-Planning scenario");
            }
        }
        // Close modal
        for entity in &modals {
            commands.entity(entity).despawn();
        }
    }
}

/// Handles dismissal of the delete confirmation modal.
#[tracing::instrument(skip_all)]
pub fn handle_delete_dismiss(
    cancel_btns: Query<
        &Interaction,
        (
            With<DeleteConfirmCancelButton>,
            With<Button>,
            Changed<Interaction>,
        ),
    >,
    mut commands: Commands,
    modals: Query<Entity, With<DeleteConfirmModal>>,
) {
    for interaction in &cancel_btns {
        if *interaction != Interaction::Pressed {
            continue;
        }
        for entity in &modals {
            commands.entity(entity).despawn();
        }
    }
}

/// Spawns the delete confirmation modal overlay.
#[tracing::instrument(skip_all)]
fn spawn_delete_modal(commands: &mut Commands, parent: Entity) {
    let modal = commands
        .spawn((
            DeleteConfirmModal,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(0.0),
                left: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            ZIndex(200),
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
        ))
        .with_children(|overlay| {
            overlay
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::all(Val::Px(24.0)),
                        row_gap: Val::Px(16.0),
                        min_width: Val::Px(320.0),
                        border_radius: BorderRadius::all(Val::Px(8.0)),
                        ..default()
                    },
                    BackgroundColor(colors::BG1),
                ))
                .with_children(|dialog| {
                    dialog.spawn((
                        Text::new("Delete Scenario?"),
                        TextFont {
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(colors::FG0),
                    ));
                    dialog.spawn((
                        Text::new(
                            "This will permanently delete the scenario and all its data.\n\
                             This action cannot be undone.",
                        ),
                        TextFont {
                            font_size: 12.0,
                            ..default()
                        },
                        TextColor(colors::GREY1),
                    ));

                    dialog
                        .spawn(Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(8.0),
                            justify_content: JustifyContent::FlexEnd,
                            ..default()
                        })
                        .with_children(|row| {
                            // Cancel
                            row.spawn((
                                DeleteConfirmCancelButton,
                                Button,
                                Node {
                                    padding: UiRect::axes(Val::Px(12.0), Val::Px(6.0)),
                                    border_radius: BorderRadius::all(Val::Px(4.0)),
                                    border: UiRect::all(Val::Px(1.0)),
                                    ..default()
                                },
                                BackgroundColor(colors::BG0),
                                BorderColor::all(colors::GREY1),
                            ))
                            .with_children(|btn| {
                                btn.spawn((
                                    Text::new("Cancel"),
                                    TextFont {
                                        font_size: 12.0,
                                        ..default()
                                    },
                                    TextColor(colors::FG0),
                                ));
                            });

                            // Confirm
                            row.spawn((
                                DeleteConfirmOkButton,
                                Button,
                                Node {
                                    padding: UiRect::axes(Val::Px(12.0), Val::Px(6.0)),
                                    border_radius: BorderRadius::all(Val::Px(4.0)),
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
                });
        })
        .id();
    commands.entity(parent).add_child(modal);
}
