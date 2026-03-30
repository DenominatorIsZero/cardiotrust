//! Explorer toolbar — status filter, sort order, text search, New Scenario button.

pub mod filter_sort;
pub mod search;

use bevy::prelude::*;
pub use filter_sort::{
    apply_filter_and_sort, handle_sort_click, handle_status_filter_click,
    update_toolbar_button_visuals,
};
pub use search::{
    fuzzy_match, handle_new_scenario_toolbar_button, handle_search_clear_click,
    handle_search_field_click, handle_search_outside_click, handle_text_search_input,
    update_search_display_text, update_search_field_visuals, SearchFocused,
};

use super::ExplorerViewRoot;
use crate::ui::colors;

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
                    border_radius: BorderRadius::all(Val::Px(4.0)),
                    ..default()
                },
                BackgroundColor(colors::BG3),
                BorderColor::all(colors::GREY1),
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
                    border_radius: BorderRadius::all(Val::Px(4.0)),
                    ..default()
                },
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
                border_radius: BorderRadius::all(Val::Px(12.0)),
                ..default()
            },
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
                border_radius: BorderRadius::all(Val::Px(12.0)),
                ..default()
            },
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
