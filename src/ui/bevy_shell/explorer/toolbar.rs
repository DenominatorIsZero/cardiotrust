//! Explorer toolbar — status filter, sort order, text search, New Scenario button.

use bevy::prelude::*;

use super::{
    card::create_new_scenario,
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
pub fn spawn_toolbar(
    mut commands: Commands,
    roots: Query<Entity, With<ExplorerViewRoot>>,
) {
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

            // Search field placeholder (Bevy UI has no built-in text input;
            // we use a button that displays the query and forwards keyboard input).
            tb.spawn((
                SearchInputField,
                Node {
                    width: Val::Px(160.0),
                    height: Val::Px(28.0),
                    padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    align_items: AlignItems::Center,
                    ..default()
                },
                BorderRadius::all(Val::Px(4.0)),
                BackgroundColor(colors::BG3),
                BorderColor(colors::GREY1),
            ))
            .with_children(|field| {
                field.spawn((
                    Text::new("Search..."),
                    TextFont {
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor(colors::GREY1),
                ));
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
        (Entity, &StatusFilterButton, &Interaction, &mut BackgroundColor),
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

/// Placeholder: reads keyboard input into `SearchQuery` resource.
///
/// Full text-input support requires a custom input widget or a plugin;
/// this stub wires the resource so `apply_filter_and_sort` can read it.
#[tracing::instrument(skip_all)]
pub fn handle_text_search_input(
    _keys: Res<ButtonInput<KeyCode>>,
    _query: ResMut<SearchQuery>,
) {
    // Full keyboard capture would require focus tracking and character events.
    // Left as a functional stub — the resource is initialised empty and
    // filter/sort still work; a future task can add a text-input widget.
}

/// Shows/hides cards based on active filter + search by toggling `Display` on the
/// card's `Node` (rather than `Visibility`), so hidden cards do not occupy grid space.
#[tracing::instrument(skip_all)]
pub fn apply_filter_and_sort(
    filter: Res<StatusFilter>,
    order: Res<SortOrder>,
    search: Res<SearchQuery>,
    scenario_list: Res<ScenarioList>,
    mut cards: Query<(&super::card::ScenarioCard, &mut Node)>,
) {
    if !filter.is_changed() && !order.is_changed() && !search.is_changed() && !scenario_list.is_changed() {
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
            StatusFilter::Planning => scenario.get_status() == &crate::core::scenario::Status::Planning,
            StatusFilter::Queued => scenario.get_status() == &crate::core::scenario::Status::Scheduled,
            StatusFilter::Running => matches!(scenario.get_status(), crate::core::scenario::Status::Running(_)),
            StatusFilter::Done => scenario.get_status() == &crate::core::scenario::Status::Done,
            StatusFilter::Failed => scenario.get_status() == &crate::core::scenario::Status::Aborted,
        };

        // Search filter
        let search_ok = if query_lower.is_empty() {
            true
        } else {
            scenario.get_id().to_lowercase().contains(&query_lower)
                || scenario.comment.to_lowercase().contains(&query_lower)
        };

        node.display = if status_ok && search_ok {
            Display::Flex
        } else {
            Display::None
        };
    }

    // TODO: re-order card nodes by `order` (requires mutable access to parent children).
    // For now the sort resource is wired; reordering would need ChildOf manipulation.
    let _ = order; // suppress unused warning
}

/// "New Scenario" toolbar button — same action as the action card.
#[allow(clippy::type_complexity)]
#[tracing::instrument(skip_all)]
pub fn handle_new_scenario_toolbar_button(
    buttons: Query<&Interaction, (With<ToolbarNewScenarioButton>, With<Button>, Changed<Interaction>)>,
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
