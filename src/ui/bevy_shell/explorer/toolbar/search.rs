use bevy::{input::keyboard::KeyboardInput, prelude::*};

use super::{
    super::card::{create_new_scenario, CardEditMode},
    SearchClearButton, SearchDisplayText, SearchInputField, SearchQuery, ToolbarNewScenarioButton,
};
use crate::{ui::colors, ProjectState, ScenarioList, SelectedSenario};

/// Whether the search field currently has keyboard focus.
#[derive(Resource, Debug, Clone, Default)]
pub struct SearchFocused(pub bool);

/// Sets `SearchFocused(true)` when the search field is clicked.
#[allow(clippy::type_complexity)]
#[tracing::instrument(skip_all)]
pub fn handle_search_field_click(
    query: Query<&Interaction, (With<SearchInputField>, With<Button>, Changed<Interaction>)>,
    mut focused: ResMut<SearchFocused>,
) {
    for interaction in &query {
        if *interaction == Interaction::Pressed {
            focused.0 = true;
        }
    }
}

/// Toggles `SearchClearButton` visibility and `SearchInputField` border color
/// based on query content and focus state.
#[allow(clippy::type_complexity)]
#[tracing::instrument(skip_all)]
pub fn update_search_field_visuals(
    search: Res<SearchQuery>,
    focused: Res<SearchFocused>,
    mut clear_buttons: Query<&mut Node, With<SearchClearButton>>,
    mut field_borders: Query<&mut BorderColor, With<SearchInputField>>,
) {
    if !search.is_changed() && !focused.is_changed() {
        return;
    }

    let show_clear = !search.0.is_empty();
    for mut node in &mut clear_buttons {
        node.display = if show_clear {
            Display::Flex
        } else {
            Display::None
        };
    }

    let border_color = if focused.0 {
        colors::ORANGE
    } else {
        colors::GREY1
    };
    for mut border in &mut field_borders {
        border.top = border_color;
        border.right = border_color;
        border.bottom = border_color;
        border.left = border_color;
    }
}

/// Resets `SearchQuery` and `SearchFocused` when the clear button is clicked.
#[allow(clippy::type_complexity)]
#[tracing::instrument(skip_all)]
pub fn handle_search_clear_click(
    buttons: Query<&Interaction, (With<SearchClearButton>, With<Button>, Changed<Interaction>)>,
    mut search: ResMut<SearchQuery>,
    mut focused: ResMut<SearchFocused>,
) {
    for interaction in &buttons {
        if *interaction == Interaction::Pressed {
            search.0.clear();
            focused.0 = false;
        }
    }
}

/// Updates the search field display text to reflect `SearchQuery`.
///
/// Shows the current query when non-empty, or the placeholder "Search..." when empty.
#[tracing::instrument(skip_all)]
pub fn update_search_display_text(
    search: Res<SearchQuery>,
    focused: Res<SearchFocused>,
    mut texts: Query<(&mut Text, &mut TextColor), With<SearchDisplayText>>,
) {
    if !search.is_changed() && !focused.is_changed() {
        return;
    }

    let (display, text_color) = if search.0.is_empty() {
        ("Search...".to_string(), colors::GREY1)
    } else if focused.0 {
        // Show query with cursor indicator
        (format!("{}_", search.0), colors::FG0)
    } else {
        (search.0.clone(), colors::FG0)
    };

    for (mut text, mut color) in &mut texts {
        text.0.clone_from(&display);
        color.0 = text_color;
    }
}

/// Unfocuses the search field when the user clicks outside it.
#[tracing::instrument(skip_all)]
pub fn handle_search_outside_click(
    mouse: Res<ButtonInput<MouseButton>>,
    search_field: Query<&Interaction, With<SearchInputField>>,
    clear_button: Query<&Interaction, With<SearchClearButton>>,
    mut focused: ResMut<SearchFocused>,
) {
    if !focused.0 || !mouse.just_pressed(MouseButton::Left) {
        return;
    }
    // If the press landed on the search field or its clear button, keep focus.
    let over_field = search_field
        .iter()
        .any(|i| matches!(i, Interaction::Pressed | Interaction::Hovered));
    let over_clear = clear_button
        .iter()
        .any(|i| matches!(i, Interaction::Pressed | Interaction::Hovered));
    if !over_field && !over_clear {
        focused.0 = false;
    }
}

/// Reads keyboard input into `SearchQuery` resource when the search field is focused.
#[tracing::instrument(skip_all)]
pub fn handle_text_search_input(
    mut keyboard: MessageReader<KeyboardInput>,
    mut search: ResMut<SearchQuery>,
    mut focused: ResMut<SearchFocused>,
    edit_mode: Res<CardEditMode>,
) {
    if edit_mode.editing_index.is_some() || !focused.0 {
        keyboard.clear();
        return;
    }

    for event in keyboard.read() {
        if event.state != bevy::input::ButtonState::Pressed {
            continue;
        }
        match event.key_code {
            KeyCode::Escape | KeyCode::Enter | KeyCode::NumpadEnter => {
                focused.0 = false;
                // Keep the query — just release focus
            }
            KeyCode::Backspace => {
                search.0.pop();
            }
            _ => {
                if let Some(text) = &event.text {
                    for ch in text.chars() {
                        if !ch.is_control() {
                            search.0.push(ch);
                        }
                    }
                }
            }
        }
    }
}

/// Character-subsequence fuzzy match.
///
/// Returns byte offsets `(start, end)` of the matched span (first matching char
/// to last matching char inclusive) within `target`, or `None` if the query
/// cannot be matched. Both strings are compared case-insensitively.
#[tracing::instrument(level = "trace", skip_all)]
pub fn fuzzy_match(query: &str, target: &str) -> Option<(usize, usize)> {
    if query.is_empty() {
        return None;
    }

    let query_lower: Vec<char> = query.to_lowercase().chars().collect();
    let mut q_iter = query_lower.iter().peekable();

    let mut match_start: Option<usize> = None;
    let mut match_end: Option<usize> = None;

    for (byte_offset, ch) in target.char_indices() {
        if let Some(&&qch) = q_iter.peek() {
            if ch.to_lowercase().next() == Some(qch) {
                if match_start.is_none() {
                    match_start = Some(byte_offset);
                }
                // advance end to include this char
                match_end = Some(byte_offset + ch.len_utf8());
                q_iter.next();
            }
        }
        if q_iter.peek().is_none() {
            break;
        }
    }

    if q_iter.peek().is_none() {
        Some((match_start.unwrap_or(0), match_end.unwrap_or(0)))
    } else {
        None
    }
}

/// "New Scenario" toolbar button — same action as the action card.
#[allow(clippy::type_complexity)]
#[tracing::instrument(skip_all)]
pub fn handle_new_scenario_toolbar_button(
    buttons: Query<
        &Interaction,
        (
            With<ToolbarNewScenarioButton>,
            With<Button>,
            Changed<Interaction>,
        ),
    >,
    mut scenario_list: ResMut<ScenarioList>,
    mut selected: ResMut<SelectedSenario>,
    project_state: Res<ProjectState>,
) {
    for interaction in &buttons {
        if *interaction == Interaction::Pressed {
            create_new_scenario(&mut scenario_list, &mut selected, &project_state);
        }
    }
}
