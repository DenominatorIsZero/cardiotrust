//! Explorer toolbar — status filter, sort order, text search, New Scenario button.

use bevy::{input::keyboard::KeyboardInput, prelude::*};

use super::{
    card::{create_new_scenario, CardEditMode},
    ExplorerViewRoot,
};
use crate::{ui::colors, ProjectState, ScenarioList, SelectedSenario};

// ── Resources ─────────────────────────────────────────────────────────────────

/// Which status to show in the Explorer grid.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StatusFilter {
    #[default]
    All,
    Planning,
    Queued,
    Running,
    Done,
    Failed,
}

/// How to order scenario cards.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortOrder {
    #[default]
    DateNewest,
    LossLowest,
    DiceHighest,
    Name,
}

/// Current text search query (lower-cased for matching).
#[derive(Resource, Debug, Clone, Default)]
pub struct SearchQuery(pub String);

/// Whether the search field currently has keyboard focus.
#[derive(Resource, Debug, Clone, Default)]
pub struct SearchFocused(pub bool);

// ── Marker components ─────────────────────────────────────────────────────────

/// Marker for the toolbar root node.
#[derive(Component, Debug)]
pub struct ToolbarNode;

/// Marks a status-filter button with the filter it represents.
#[derive(Component, Debug, Clone, Copy)]
pub struct StatusFilterButton(pub StatusFilter);

/// Marks a sort-order button with the order it represents.
#[derive(Component, Debug, Clone, Copy)]
pub struct SortOrderButton(pub SortOrder);

/// Marker for the text search input field.
#[derive(Component, Debug)]
pub struct SearchInputField;

/// Marker for the clear button inside the search field.
#[derive(Component, Debug)]
pub struct SearchClearButton;

/// Marker for the text node inside the search field that displays the query.
#[derive(Component, Debug)]
pub struct SearchDisplayText;

/// Marker for the "New Scenario" button in the toolbar.
#[derive(Component, Debug)]
pub struct ToolbarNewScenarioButton;

// ── Spawn ─────────────────────────────────────────────────────────────────────

/// Spawns the toolbar node as the first child of `ExplorerViewRoot` and registers
/// `StatusFilter`, `SortOrder`, and `SearchQuery` resources.
///
/// Runs after `spawn_explorer_view` in the `.chain()` so the root is guaranteed
/// to exist.
#[tracing::instrument(skip_all)]
pub fn spawn_toolbar(mut commands: Commands, roots: Query<Entity, With<ExplorerViewRoot>>) {
    // Resources
    commands.insert_resource(StatusFilter::default());
    commands.insert_resource(SortOrder::default());
    commands.insert_resource(SearchQuery::default());

    let Ok(root) = roots.single() else {
        return;
    };
    spawn_toolbar_into(&mut commands, root);
}

/// Actually spawns the toolbar node tree inside `parent`.
#[tracing::instrument(skip_all)]
fn spawn_toolbar_into(commands: &mut Commands, parent: Entity) {
    let toolbar = commands
        .spawn((
            ToolbarNode,
            Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(8.0),
                padding: UiRect::axes(Val::Px(16.0), Val::Px(8.0)),
                flex_wrap: FlexWrap::Wrap,
                row_gap: Val::Px(8.0),
                ..default()
            },
            BackgroundColor(colors::BG1),
        ))
        .with_children(|tb| {
            // Status filter buttons
            tb.spawn((
                Text::new("Filter:"),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(colors::GREY1),
            ));

            for (label, filter) in [
                ("All", StatusFilter::All),
                ("Planning", StatusFilter::Planning),
                ("Queued", StatusFilter::Queued),
                ("Running", StatusFilter::Running),
                ("Done", StatusFilter::Done),
                ("Failed", StatusFilter::Failed),
            ] {
                spawn_filter_button(tb, label, filter);
            }

            // Sort order buttons
            tb.spawn((
                Text::new("Sort:"),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(colors::GREY1),
            ));

            for (label, order) in [
                ("Date", SortOrder::DateNewest),
                ("Loss", SortOrder::LossLowest),
                ("Dice", SortOrder::DiceHighest),
                ("Name", SortOrder::Name),
            ] {
                spawn_sort_button(tb, label, order);
            }

            // Search field — a clickable Button that captures keyboard events.
            tb.spawn((
                SearchInputField,
                Button,
                Node {
                    width: Val::Px(160.0),
                    height: Val::Px(28.0),
                    padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::SpaceBetween,
                    ..default()
                },
                BorderRadius::all(Val::Px(4.0)),
                BackgroundColor(colors::BG3),
                BorderColor(colors::GREY1),
            ))
            .with_children(|field| {
                field.spawn((
                    SearchDisplayText,
                    Text::new("Search..."),
                    TextFont {
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor(colors::GREY1),
                ));
                // Clear button — hidden when query is empty
                field
                    .spawn((
                        SearchClearButton,
                        Button,
                        Node {
                            display: Display::None,
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            padding: UiRect::axes(Val::Px(2.0), Val::Px(0.0)),
                            ..default()
                        },
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new("x"),
                            TextFont {
                                font_size: 14.0,
                                ..default()
                            },
                            TextColor(colors::GREY1),
                        ));
                    });
            });

            // Spacer
            tb.spawn(Node {
                flex_grow: 1.0,
                ..default()
            });

            // New Scenario button (right-aligned)
            tb.spawn((
                ToolbarNewScenarioButton,
                Button,
                Node {
                    padding: UiRect::axes(Val::Px(16.0), Val::Px(8.0)),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                BorderRadius::all(Val::Px(4.0)),
                BackgroundColor(colors::ORANGE),
            ))
            .with_children(|btn| {
                btn.spawn((
                    Text::new("+ New Scenario"),
                    TextFont {
                        font_size: 13.0,
                        ..default()
                    },
                    TextColor(colors::BG0),
                ));
            });
        })
        .id();

    commands.entity(parent).insert_children(0, &[toolbar]);
}

#[tracing::instrument(skip_all)]
fn spawn_filter_button(parent: &mut ChildSpawnerCommands, label: &str, filter: StatusFilter) {
    parent
        .spawn((
            StatusFilterButton(filter),
            Button,
            Node {
                padding: UiRect::axes(Val::Px(10.0), Val::Px(4.0)),
                ..default()
            },
            BorderRadius::all(Val::Px(12.0)),
            BackgroundColor(colors::BG3),
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new(label.to_string()),
                TextFont {
                    font_size: 11.0,
                    ..default()
                },
                TextColor(colors::FG1),
            ));
        });
}

#[tracing::instrument(skip_all)]
fn spawn_sort_button(parent: &mut ChildSpawnerCommands, label: &str, order: SortOrder) {
    parent
        .spawn((
            SortOrderButton(order),
            Button,
            Node {
                padding: UiRect::axes(Val::Px(10.0), Val::Px(4.0)),
                ..default()
            },
            BorderRadius::all(Val::Px(12.0)),
            BackgroundColor(colors::BG3),
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new(label.to_string()),
                TextFont {
                    font_size: 11.0,
                    ..default()
                },
                TextColor(colors::FG1),
            ));
        });
}

// ── Update systems ────────────────────────────────────────────────────────────

#[allow(clippy::type_complexity)]
/// Writes the selected filter to `StatusFilter` resource.
#[tracing::instrument(skip_all)]
pub fn handle_status_filter_click(
    buttons: Query<(&StatusFilterButton, &Interaction), (With<Button>, Changed<Interaction>)>,
    mut filter: ResMut<StatusFilter>,
) {
    for (btn, interaction) in &buttons {
        if *interaction == Interaction::Pressed {
            *filter = btn.0;
        }
    }
}

#[allow(clippy::type_complexity)]
/// Writes the selected sort order to `SortOrder` resource.
#[tracing::instrument(skip_all)]
pub fn handle_sort_click(
    buttons: Query<(&SortOrderButton, &Interaction), (With<Button>, Changed<Interaction>)>,
    mut order: ResMut<SortOrder>,
) {
    for (btn, interaction) in &buttons {
        if *interaction == Interaction::Pressed {
            *order = btn.0;
        }
    }
}

/// Updates filter and sort pill button backgrounds to show which one is active.
#[allow(clippy::type_complexity)]
#[tracing::instrument(skip_all)]
pub fn update_toolbar_button_visuals(
    filter: Res<StatusFilter>,
    order: Res<SortOrder>,
    mut filter_buttons: Query<
        (
            Entity,
            &StatusFilterButton,
            &Interaction,
            &mut BackgroundColor,
        ),
        (With<Button>, Without<SortOrderButton>),
    >,
    mut sort_buttons: Query<
        (Entity, &SortOrderButton, &Interaction, &mut BackgroundColor),
        (With<Button>, Without<StatusFilterButton>),
    >,
    mut text_colors: Query<(&ChildOf, &mut TextColor)>,
) {
    // Collect active-state per entity so we can update child text colours.
    let mut active_entities: std::collections::HashSet<Entity> = std::collections::HashSet::new();

    for (entity, btn, interaction, mut bg) in &mut filter_buttons {
        let is_active = btn.0 == *filter;
        if is_active {
            active_entities.insert(entity);
        }
        bg.0 = if is_active {
            crate::ui::colors::ORANGE
        } else {
            match interaction {
                Interaction::Hovered => crate::ui::colors::BG2,
                _ => crate::ui::colors::BG3,
            }
        };
    }

    for (entity, btn, interaction, mut bg) in &mut sort_buttons {
        let is_active = btn.0 == *order;
        if is_active {
            active_entities.insert(entity);
        }
        bg.0 = if is_active {
            crate::ui::colors::ORANGE
        } else {
            match interaction {
                Interaction::Hovered => crate::ui::colors::BG2,
                _ => crate::ui::colors::BG3,
            }
        };
    }

    // Update text colour for children of all filter/sort buttons.
    let all_button_entities: std::collections::HashSet<Entity> = filter_buttons
        .iter()
        .map(|(e, _, _, _)| e)
        .chain(sort_buttons.iter().map(|(e, _, _, _)| e))
        .collect();

    for (child_of, mut text_color) in &mut text_colors {
        let parent = child_of.parent();
        if !all_button_entities.contains(&parent) {
            continue;
        }
        text_color.0 = if active_entities.contains(&parent) {
            crate::ui::colors::BG0
        } else {
            crate::ui::colors::FG1
        };
    }
}

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
        border.0 = border_color;
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
    mut keyboard: EventReader<KeyboardInput>,
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

/// Shows/hides cards based on active filter + search query.
///
/// Toggles `Display` on the card's `Node` (not `Visibility`), so hidden cards
/// do not occupy grid space. Also updates `CardMatchHighlight` on label entities.
#[tracing::instrument(skip_all)]
pub fn apply_filter_and_sort(
    filter: Res<StatusFilter>,
    order: Res<SortOrder>,
    search: Res<SearchQuery>,
    scenario_list: Res<ScenarioList>,
    mut cards: Query<(&super::card::ScenarioCard, &mut Node)>,
    mut name_labels: Query<
        (
            &super::card::CardNameLabel,
            &mut super::card::CardMatchHighlight,
        ),
        Without<super::card::CardIdLabel>,
    >,
    mut id_labels: Query<
        (
            &super::card::CardIdLabel,
            &mut super::card::CardMatchHighlight,
        ),
        Without<super::card::CardNameLabel>,
    >,
) {
    if !filter.is_changed()
        && !order.is_changed()
        && !search.is_changed()
        && !scenario_list.is_changed()
    {
        return;
    }

    let query_lower = search.0.to_lowercase();

    for (card, mut node) in &mut cards {
        let Some(entry) = scenario_list.entries.get(card.index) else {
            continue;
        };
        let scenario = &entry.scenario;

        // Status filter
        let status_ok = match *filter {
            StatusFilter::All => true,
            StatusFilter::Planning => {
                scenario.get_status() == &crate::core::scenario::Status::Planning
            }
            StatusFilter::Queued => {
                scenario.get_status() == &crate::core::scenario::Status::Scheduled
            }
            StatusFilter::Running => matches!(
                scenario.get_status(),
                crate::core::scenario::Status::Running(_)
            ),
            StatusFilter::Done => scenario.get_status() == &crate::core::scenario::Status::Done,
            StatusFilter::Failed => {
                scenario.get_status() == &crate::core::scenario::Status::Aborted
            }
        };

        // Fuzzy search filter
        let (search_ok, name_range, id_range) = if query_lower.is_empty() {
            (true, None, None)
        } else {
            let comment = &scenario.comment;
            let id = scenario.get_id();
            let display_name = if comment.is_empty() {
                id.as_str()
            } else {
                comment.as_str()
            };
            let name_match = fuzzy_match(&query_lower, &display_name.to_lowercase());
            let id_match = fuzzy_match(&query_lower, &id.to_lowercase());
            let ok = name_match.is_some() || id_match.is_some();
            (ok, name_match, id_match)
        };

        node.display = if status_ok && search_ok {
            Display::Flex
        } else {
            Display::None
        };

        // Update CardMatchHighlight on the label entities (matched by scenario index).
        for (label, mut highlight) in &mut name_labels {
            if label.index == card.index {
                highlight.name_range = name_range;
                highlight.id_range = id_range;
            }
        }
        for (label, mut highlight) in &mut id_labels {
            if label.index == card.index {
                highlight.name_range = name_range;
                highlight.id_range = id_range;
            }
        }
    }

    // TODO: re-order card nodes by `order` (requires mutable access to parent children).
    // For now the sort resource is wired; reordering would need ChildOf manipulation.
    let _ = order; // suppress unused warning
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
