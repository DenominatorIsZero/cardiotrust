//! Scenario card widget — spawn and update systems.
//!
//! Each `ScenarioCard` entity is a Bevy UI `Node` tree displaying:
//! status badge · thumbnail area · metrics row · display-name (comment) · id · timestamp.
//!
//! # Interaction model
//!
//! - **Single click** — selects the card (sets `SelectedSenario`); does NOT navigate.
//! - **Click on already-selected card** — enters inline comment/name edit mode.
//! - **Enter / Escape** — commits or cancels the inline edit.
//! - **Double-click (navigate)** — use the sidebar or keyboard shortcut instead.

use bevy::{input::keyboard::KeyboardInput, prelude::*};

use super::thumbnail::{ThumbnailCache, ThumbnailState};
use crate::{
    core::scenario::Status,
    ui::colors,
    ScenarioList, SelectedSenario,
};

// ── Resources ─────────────────────────────────────────────────────────────────

/// Tracks which scenario card (by index) is currently in inline-edit mode, and the
/// draft text being edited.
#[derive(Resource, Debug, Default)]
pub struct CardEditMode {
    /// Index of the scenario being edited, or `None` if not editing.
    pub editing_index: Option<usize>,
    /// Draft text (mirrors the scenario `comment` while the user types).
    pub draft: String,
}

/// Tracks the last click time and card index for double-click detection.
#[derive(Resource, Debug, Default)]
pub struct LastCardClick {
    /// Time (in seconds since startup) of the last click.
    pub time: f64,
    /// Card index that was last clicked.
    pub index: Option<usize>,
}

/// Maximum interval (seconds) between two clicks to count as a double-click.
const DOUBLE_CLICK_SECS: f64 = 0.4;

// ── Components ────────────────────────────────────────────────────────────────

/// Marker for a scenario card root node.
#[derive(Component, Debug, Clone)]
pub struct ScenarioCard {
    /// The scenario ID this card represents.
    pub scenario_id: String,
    /// Index into `ScenarioList::entries`.
    pub index: usize,
}

/// Marker for the "New Scenario" action card.
#[derive(Component, Debug)]
pub struct NewScenarioActionCard;

/// Marker for the thumbnail area node inside a card.
#[derive(Component, Debug)]
pub struct CardThumbnailArea {
    pub scenario_id: String,
}

/// Marker for the primary display-name text node (shows comment or ID).
#[derive(Component, Debug)]
pub struct CardNameLabel {
    pub index: usize,
}

/// Marker for the secondary ID text node.
#[derive(Component, Debug)]
pub struct CardIdLabel {
    pub index: usize,
}

/// Marker for the inline-edit name field inside a new scenario card.
#[derive(Component, Debug)]
pub struct InlineEditName;

/// Marker for the inline-edit comment field inside a new scenario card.
#[derive(Component, Debug)]
pub struct InlineEditComment;

/// Quick-action button on a card (delete, copy, schedule).
/// Carries the scenario index so no parent-entity walk is needed.
#[derive(Component, Debug, Clone)]
pub struct CardQuickAction {
    pub index: usize,
    pub kind: CardQuickActionKind,
}

/// The kind of quick action.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CardQuickActionKind {
    Delete,
    Copy,
    Schedule,
}

// ── Spawn helpers ─────────────────────────────────────────────────────────────

/// Spawns a scenario card entity tree.
///
/// The card contains (top to bottom):
/// 1. Status badge (pill)
/// 2. Thumbnail area
/// 3. Metrics row (Done only)
/// 4. Title
/// 5. Comment (when non-empty)
/// 6. Timestamp
#[tracing::instrument(skip_all)]
pub fn spawn_card(
    commands: &mut Commands,
    scenario_id: &str,
    index: usize,
    status: &Status,
    title: &str,
    comment: &str,
    timestamp: Option<&str>,
    metrics: Option<(f32, f32)>, // (dice, loss)
    progress: Option<f32>,
    etc: Option<&str>,
    thumbnail: Option<&ThumbnailState>,
) -> Entity {
    let badge_color = status_badge_color(status);
    let badge_label = status_label(status);

    let card = commands
        .spawn((
            ScenarioCard {
                scenario_id: scenario_id.to_string(),
                index,
            },
            Button,
            Node {
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(12.0)),
                row_gap: Val::Px(8.0),
                border: UiRect::all(Val::Px(2.0)),
                min_width: Val::Px(200.0),
                min_height: Val::Px(280.0),
                ..default()
            },
            BorderRadius::all(Val::Px(8.0)),
            BackgroundColor(colors::BG1),
            BorderColor(Color::NONE),
        ))
        .with_children(|card| {
            // ── Status badge ──────────────────────────────────────────────────
            card.spawn((
                Node {
                    padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
                    align_self: AlignSelf::FlexStart,
                    ..default()
                },
                BorderRadius::all(Val::Px(12.0)),
                BackgroundColor(badge_color),
            ))
            .with_children(|badge| {
                badge.spawn((
                    Text::new(badge_label),
                    TextFont {
                        font_size: 11.0,
                        ..default()
                    },
                    TextColor(colors::BG0),
                ));
            });

            // ── Thumbnail area ────────────────────────────────────────────────
            spawn_thumbnail_area(card, scenario_id, status, progress, etc, thumbnail);

            // ── Metrics row (Done only) ───────────────────────────────────────
            if let Some((dice, loss)) = metrics {
                card.spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(12.0),
                    ..default()
                })
                .with_children(|row| {
                    row.spawn((
                        Text::new(format!("Dice: {dice:.3}")),
                        TextFont {
                            font_size: 11.0,
                            ..default()
                        },
                        TextColor(colors::GREEN),
                    ));
                    row.spawn((
                        Text::new(format!("Loss: {loss:.4}")),
                        TextFont {
                            font_size: 11.0,
                            ..default()
                        },
                        TextColor(colors::YELLOW),
                    ));
                });
            }

            // ── Primary display name (comment if set, else ID) ────────────────
            let display_name = if comment.is_empty() { title } else { comment };
            card.spawn((
                CardNameLabel { index },
                Text::new(truncate_str(display_name, 36)),
                TextFont {
                    font_size: 13.0,
                    ..default()
                },
                TextColor(colors::FG0),
            ));

            // ── Secondary ID (smaller, dimmed) ────────────────────────────────
            card.spawn((
                CardIdLabel { index },
                Text::new(truncate_str(title, 32)),
                TextFont {
                    font_size: 10.0,
                    ..default()
                },
                TextColor(colors::GREY1),
            ));

            // ── Timestamp ─────────────────────────────────────────────────────
            if let Some(ts) = timestamp {
                card.spawn((
                    Text::new(ts.to_string()),
                    TextFont {
                        font_size: 10.0,
                        ..default()
                    },
                    TextColor(colors::GREY1),
                ));
            }

            // Spacer pushes the quick-action row to the bottom of the card.
            card.spawn(Node {
                flex_grow: 1.0,
                ..default()
            });

            // ── Quick-action row ──────────────────────────────────────────────
            spawn_quick_actions(card, index, status);
        })
        .id();

    card
}

/// Spawns the thumbnail area child node.
#[tracing::instrument(skip_all)]
fn spawn_thumbnail_area(
    parent: &mut ChildSpawnerCommands,
    scenario_id: &str,
    status: &Status,
    progress: Option<f32>,
    etc: Option<&str>,
    thumbnail: Option<&ThumbnailState>,
) {
    parent
        .spawn((
            CardThumbnailArea {
                scenario_id: scenario_id.to_string(),
            },
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(120.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                overflow: Overflow::clip(),
                ..default()
            },
            BorderRadius::all(Val::Px(4.0)),
            BackgroundColor(colors::BG3),
        ))
        .with_children(|area| {
            match (status, thumbnail) {
                (Status::Done, Some(ThumbnailState::Ready(handle))) => {
                    area.spawn((
                        ImageNode::new(handle.clone()),
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            ..default()
                        },
                    ));
                }
                (
                    Status::Done,
                    None | Some(ThumbnailState::Generating | ThumbnailState::Pending),
                ) => {
                    area.spawn((
                        Text::new("Loading..."),
                        TextFont {
                            font_size: 12.0,
                            ..default()
                        },
                        TextColor(colors::GREY1),
                    ));
                }
                (Status::Done, Some(ThumbnailState::Failed(msg))) => {
                    area.spawn((
                        Text::new(format!("! {}", truncate_str(msg, 24))),
                        TextFont {
                            font_size: 12.0,
                            ..default()
                        },
                        TextColor(colors::RED),
                    ));
                }
                (Status::Running(_), _) => {
                    let pct = progress.unwrap_or(0.0) * 100.0;
                    let etc_text = etc.unwrap_or("ETC: ???");
                    area.spawn(Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: Val::Px(4.0),
                        ..default()
                    })
                    .with_children(|col| {
                        col.spawn((
                            Text::new(format!("{pct:.0}%")),
                            TextFont {
                                font_size: 18.0,
                                ..default()
                            },
                            TextColor(colors::YELLOW),
                        ));
                        col.spawn((
                            Text::new(etc_text.to_string()),
                            TextFont {
                                font_size: 11.0,
                                ..default()
                            },
                            TextColor(colors::GREY1),
                        ));
                    });
                }
                (Status::Planning | Status::Scheduled, _) => {
                    area.spawn((
                        Text::new("[?]"),
                        TextFont {
                            font_size: 24.0,
                            ..default()
                        },
                        TextColor(colors::GREY1),
                    ));
                }
                (Status::Aborted, _) => {
                    area.spawn((
                        Text::new("[X]"),
                        TextFont {
                            font_size: 24.0,
                            ..default()
                        },
                        TextColor(colors::RED),
                    ));
                }
                (Status::Simulating, _) => {
                    area.spawn((
                        Text::new("Simulating..."),
                        TextFont {
                            font_size: 12.0,
                            ..default()
                        },
                        TextColor(colors::BLUE),
                    ));
                }
            }
        });
}

/// Spawns the `[D] [C] [S]` quick-action button row at the bottom of a card.
#[tracing::instrument(skip_all)]
fn spawn_quick_actions(parent: &mut ChildSpawnerCommands, index: usize, status: &Status) {
    let show_schedule = matches!(status, Status::Planning | Status::Aborted);

    parent
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::FlexEnd,
            column_gap: Val::Px(4.0),
            width: Val::Percent(100.0),
            ..default()
        })
        .with_children(|row| {
            spawn_quick_button(row, "[D]", CardQuickActionKind::Delete, colors::RED, index);
            spawn_quick_button(row, "[C]", CardQuickActionKind::Copy, colors::GREY1, index);
            if show_schedule {
                spawn_quick_button(row, "[S]", CardQuickActionKind::Schedule, colors::BLUE, index);
            }
        });
}

/// Spawns a single quick-action button inside the row.
#[tracing::instrument(skip_all)]
fn spawn_quick_button(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    kind: CardQuickActionKind,
    text_color: Color,
    index: usize,
) {
    parent
        .spawn((
            CardQuickAction { index, kind },
            Button,
            Node {
                padding: UiRect::axes(Val::Px(6.0), Val::Px(2.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BorderRadius::all(Val::Px(3.0)),
            BackgroundColor(Color::NONE),
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new(label.to_string()),
                TextFont {
                    font_size: 11.0,
                    ..default()
                },
                TextColor(text_color),
            ));
        });
}

// ── Update systems ────────────────────────────────────────────────────────────

/// Changes card background to `BG2` on hover, restoring `BG1` otherwise.
/// Skips disabled / active cards.
#[tracing::instrument(skip_all)]
pub fn update_card_hover(
    selected: Res<SelectedSenario>,
    mut cards: Query<(&ScenarioCard, &Interaction, &mut BackgroundColor)>,
) {
    for (card, interaction, mut bg) in &mut cards {
        let is_active = selected.index == Some(card.index);
        if is_active {
            continue; // active-card styling is handled by `update_active_card_border`
        }
        let target = match interaction {
            Interaction::Hovered => colors::BG2,
            _ => colors::BG1,
        };
        bg.set_if_neq(BackgroundColor(target));
    }
}

/// Applies orange 2 px border on the active-scenario card; removes it from others.
#[tracing::instrument(skip_all)]
pub fn update_active_card_border(
    selected: Res<SelectedSenario>,
    mut cards: Query<(&ScenarioCard, &mut BorderColor, &mut BackgroundColor)>,
) {
    for (card, mut border, mut bg) in &mut cards {
        if selected.index == Some(card.index) {
            border.set_if_neq(BorderColor(colors::ORANGE));
            bg.set_if_neq(BackgroundColor(colors::BG2));
        } else {
            border.set_if_neq(BorderColor(Color::NONE));
        }
    }
}

/// Rebuilds all scenario cards whenever `ScenarioList` or `ThumbnailCache` changes.
///
/// A full rebuild (despawn all, respawn all) is used instead of incremental diffing
/// because:
/// - Copies share the same ID as the original, so ID-based diffing misses them.
/// - Deletions shift indices, invalidating `ScenarioCard::index` on surviving cards.
/// - The list is never large enough for the overhead to matter.
///
/// Rebuilds are suppressed while a card is in inline-edit mode to avoid
/// interrupting the user mid-type.
#[allow(clippy::too_many_arguments)]
#[tracing::instrument(skip_all)]
pub fn sync_cards_to_scenarios(
    mut commands: Commands,
    scenario_list: Res<ScenarioList>,
    thumbnail_cache: Res<ThumbnailCache>,
    edit_mode: Res<CardEditMode>,
    grids: Query<Entity, With<super::ExplorerGridNode>>,
    existing_cards: Query<Entity, With<ScenarioCard>>,
    action_cards: Query<Entity, With<NewScenarioActionCard>>,
) {
    if !scenario_list.is_changed() && !thumbnail_cache.is_changed() {
        return;
    }

    // Don't tear down cards while the user is typing a new name.
    if edit_mode.editing_index.is_some() {
        return;
    }

    let Ok(grid) = grids.single() else {
        return;
    };

    // Despawn every existing scenario card and the action card.
    for entity in &existing_cards {
        commands.entity(entity).despawn();
    }
    for entity in &action_cards {
        commands.entity(entity).despawn();
    }

    // Respawn all cards in index order.
    for (index, entry) in scenario_list.entries.iter().enumerate() {
        let scenario = &entry.scenario;
        let id = scenario.get_id().clone();

        let thumbnail = thumbnail_cache.states.get(&id);
        let metrics = if scenario.get_status() == &Status::Done {
            scenario.summary.as_ref().map(|s| (s.dice, s.loss))
        } else {
            None
        };
        let progress = if matches!(scenario.get_status(), Status::Running(_)) {
            Some(scenario.get_progress())
        } else {
            None
        };
        let etc_str = scenario.get_etc();
        let etc = if etc_str.is_empty() { None } else { Some(etc_str.as_str()) };

        let ts = scenario
            .started
            .map(|dt| dt.format("%Y-%m-%d").to_string())
            .unwrap_or_default();
        let ts_opt = if ts.is_empty() { None } else { Some(ts.as_str()) };

        let card_entity = spawn_card(
            &mut commands,
            &id,
            index,
            scenario.get_status(),
            &id,
            &scenario.comment,
            ts_opt,
            metrics,
            progress,
            etc,
            thumbnail,
        );
        commands.entity(grid).add_child(card_entity);
    }

    // Spawn the "New Scenario" action card at the end.
    let action = spawn_new_scenario_action_card(&mut commands);
    commands.entity(grid).add_child(action);
}

/// Spawns the "New Scenario" dashed-border action card.
#[tracing::instrument(skip_all)]
pub fn spawn_new_scenario_action_card(commands: &mut Commands) -> Entity {
    commands
        .spawn((
            NewScenarioActionCard,
            Button,
            Node {
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(24.0)),
                border: UiRect::all(Val::Px(2.0)),
                row_gap: Val::Px(8.0),
                min_width: Val::Px(200.0),
                min_height: Val::Px(120.0),
                ..default()
            },
            BorderRadius::all(Val::Px(8.0)),
            BackgroundColor(Color::NONE),
            BorderColor(colors::GREY1),
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new("+"),
                TextFont {
                    font_size: 28.0,
                    ..default()
                },
                TextColor(colors::GREY1),
            ));
            btn.spawn((
                Text::new("New Scenario"),
                TextFont {
                    font_size: 13.0,
                    ..default()
                },
                TextColor(colors::GREY1),
            ));
        })
        .id()
}

// ── Click handlers ────────────────────────────────────────────────────────────

#[allow(clippy::type_complexity)]
/// Handles card clicks:
/// - First click → select the card.
/// - Click on already-selected card → enter inline name/comment edit mode.
/// - Double-click → navigate: Results view if Done, Scenario view otherwise.
///
/// Skips processing when a quick-action child button was pressed (those are
/// handled by `handle_card_quick_actions`).
#[tracing::instrument(skip_all)]
pub fn handle_card_click(
    cards: Query<(&ScenarioCard, &Interaction), (With<Button>, Changed<Interaction>)>,
    quick_actions: Query<&Interaction, (With<CardQuickAction>, With<Button>)>,
    mut selected: ResMut<SelectedSenario>,
    mut edit_mode: ResMut<CardEditMode>,
    mut last_click: ResMut<LastCardClick>,
    mut next_state: ResMut<NextState<crate::ui::UiState>>,
    scenario_list: Res<ScenarioList>,
    time: Res<Time>,
) {
    // If any quick-action button is currently pressed, don't also fire card logic.
    let quick_action_pressed = quick_actions.iter().any(|i| *i == Interaction::Pressed);

    for (card, interaction) in &cards {
        if *interaction != Interaction::Pressed {
            continue;
        }
        if quick_action_pressed {
            continue;
        }

        let now = time.elapsed_secs_f64();
        let is_double_click = last_click.index == Some(card.index)
            && (now - last_click.time) < DOUBLE_CLICK_SECS;

        // Update last-click tracking.
        last_click.time = now;
        last_click.index = Some(card.index);

        if is_double_click {
            // Double-click: navigate to the appropriate view.
            selected.index = Some(card.index);
            edit_mode.editing_index = None;
            edit_mode.draft = String::new();
            let is_done = scenario_list
                .entries
                .get(card.index)
                .is_some_and(|e| e.scenario.get_status() == &Status::Done);
            if is_done {
                next_state.set(crate::ui::UiState::Results);
            } else {
                next_state.set(crate::ui::UiState::Scenario);
            }
        } else if selected.index == Some(card.index) {
            // Second single-click on already-selected card → inline edit.
            let draft = scenario_list
                .entries
                .get(card.index)
                .map(|e| e.scenario.comment.clone())
                .unwrap_or_default();
            edit_mode.editing_index = Some(card.index);
            edit_mode.draft = draft;
        } else {
            // First click → select; cancel any in-progress edit.
            selected.index = Some(card.index);
            if edit_mode.editing_index.is_some() {
                edit_mode.editing_index = None;
                edit_mode.draft = String::new();
            }
        }
    }
}

/// Handles quick-action button presses ([D] delete, [C] copy, [S] schedule).
#[allow(clippy::type_complexity)]
#[tracing::instrument(skip_all)]
pub fn handle_card_quick_actions(
    quick_actions: Query<(&CardQuickAction, &Interaction), (With<Button>, Changed<Interaction>)>,
    mut scenario_list: ResMut<ScenarioList>,
    mut selected: ResMut<SelectedSenario>,
) {
    for (action, interaction) in &quick_actions {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let idx = action.index;
        match action.kind {
            CardQuickActionKind::Delete => {
                if idx < scenario_list.entries.len() {
                    let _ = scenario_list.entries[idx].scenario.delete();
                    scenario_list.entries.remove(idx);
                    if selected.index == Some(idx) {
                        selected.index = None;
                    }
                }
            }
            CardQuickActionKind::Copy => {
                if let Some(entry) = scenario_list.entries.get(idx) {
                    // Build a new scenario with a fresh ID, copying config and comment.
                    match crate::core::scenario::Scenario::build(None) {
                        Ok(mut new_scenario) => {
                            new_scenario.config = entry.scenario.config.clone();
                            new_scenario.comment =
                                format!("Copy of {}", entry.scenario.comment.as_str().trim().to_string().trim_start_matches("Copy of ").trim());
                            if let Err(e) = new_scenario.save() {
                                tracing::warn!("Failed to save copied scenario: {e}");
                            }
                            scenario_list.entries.push(crate::ScenarioBundle {
                                scenario: new_scenario,
                                join_handle: None,
                                epoch_rx: None,
                                summary_rx: None,
                            });
                        }
                        Err(e) => tracing::warn!("Failed to create copy of scenario: {e}"),
                    }
                }
            }
            CardQuickActionKind::Schedule => {
                if let Some(entry) = scenario_list.entries.get_mut(idx) {
                    let _ = entry.scenario.schedule();
                }
            }
        }
    }
}

/// Captures keyboard input when a card is in inline-edit mode and updates the
/// scenario comment. Enter/Escape commit or cancel.
#[tracing::instrument(skip_all)]
pub fn handle_card_inline_edit(
    mut edit_mode: ResMut<CardEditMode>,
    mut scenario_list: ResMut<ScenarioList>,
    mut keyboard: EventReader<KeyboardInput>,
) {
    let Some(editing_index) = edit_mode.editing_index else {
        keyboard.clear();
        return;
    };

    for event in keyboard.read() {
        if event.state != bevy::input::ButtonState::Pressed {
            continue;
        }
        match event.key_code {
            KeyCode::Enter | KeyCode::NumpadEnter => {
                // Commit the edit and persist to disk.
                if let Some(entry) = scenario_list.entries.get_mut(editing_index) {
                    entry.scenario.comment.clone_from(&edit_mode.draft);
                    if let Err(e) = entry.scenario.save() {
                        tracing::warn!("Failed to save scenario after comment edit: {e}");
                    }
                }
                edit_mode.editing_index = None;
                edit_mode.draft = String::new();
            }
            KeyCode::Escape => {
                // Cancel without saving.
                edit_mode.editing_index = None;
                edit_mode.draft = String::new();
            }
            KeyCode::Backspace => {
                edit_mode.draft.pop();
            }
            _ => {
                // Append printable characters.
                if let Some(text) = &event.text {
                    for ch in text.chars() {
                        if !ch.is_control() {
                            edit_mode.draft.push(ch);
                        }
                    }
                }
            }
        }
    }
}

/// Updates `CardNameLabel` and `CardIdLabel` text nodes to reflect current scenario
/// data, shows the edit-mode draft when applicable, and gives a rename hint on the
/// selected card.
#[tracing::instrument(skip_all)]
pub fn update_card_labels(
    edit_mode: Res<CardEditMode>,
    selected: Res<SelectedSenario>,
    scenario_list: Res<ScenarioList>,
    mut name_labels: Query<(&CardNameLabel, &mut Text, &mut TextColor), Without<CardIdLabel>>,
    mut id_labels: Query<(&CardIdLabel, &mut Text), Without<CardNameLabel>>,
) {
    if !edit_mode.is_changed() && !scenario_list.is_changed() && !selected.is_changed() {
        return;
    }

    for (label, mut text, mut color) in &mut name_labels {
        let Some(entry) = scenario_list.entries.get(label.index) else {
            continue;
        };
        let is_editing = edit_mode.editing_index == Some(label.index);
        let is_selected = selected.index == Some(label.index);

        let display = if is_editing {
            // Show draft with a blinking-cursor indicator.
            color.0 = colors::ORANGE;
            format!("{}|", edit_mode.draft)
        } else if is_selected {
            // Selected but not editing: show rename hint.
            color.0 = colors::FG0;
            let comment = &entry.scenario.comment;
            let id = entry.scenario.get_id();
            let base = if comment.is_empty() {
                truncate_str(id, 28)
            } else {
                truncate_str(comment, 28)
            };
            format!("{base}  [rename]")
        } else {
            color.0 = colors::FG0;
            let comment = &entry.scenario.comment;
            let id = entry.scenario.get_id();
            if comment.is_empty() {
                truncate_str(id, 36)
            } else {
                truncate_str(comment, 36)
            }
        };
        text.0 = display;
    }

    for (label, mut text) in &mut id_labels {
        let Some(entry) = scenario_list.entries.get(label.index) else {
            continue;
        };
        text.0 = truncate_str(entry.scenario.get_id(), 32);
    }
}

#[allow(clippy::type_complexity)]
/// Handles "New Scenario" action card click: creates a new Planning scenario,
/// sets it as active, and syncs the card list (grid stays visible).
#[tracing::instrument(skip_all)]
pub fn handle_new_scenario_card_click(
    action_cards: Query<&Interaction, (With<NewScenarioActionCard>, With<Button>, Changed<Interaction>)>,
    mut scenario_list: ResMut<ScenarioList>,
    mut selected: ResMut<SelectedSenario>,
    project_state: Res<crate::ProjectState>,
) {
    for interaction in &action_cards {
        if *interaction == Interaction::Pressed {
            create_new_scenario(&mut scenario_list, &mut selected, &project_state);
        }
    }
}

/// Creates a new Planning scenario and appends it to the list.
#[tracing::instrument(skip_all)]
pub(crate) fn create_new_scenario(
    scenario_list: &mut ScenarioList,
    selected: &mut SelectedSenario,
    _project_state: &crate::ProjectState,
) {
    match crate::core::scenario::Scenario::build(None) {
        Ok(scenario) => {
            let index = scenario_list.entries.len();
            scenario_list.entries.push(crate::ScenarioBundle {
                scenario,
                join_handle: None,
                epoch_rx: None,
                summary_rx: None,
            });
            selected.index = Some(index);
        }
        Err(e) => {
            tracing::warn!("Failed to create new scenario: {}", e);
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

const fn status_badge_color(status: &Status) -> Color {
    match status {
        Status::Done => colors::GREEN,
        Status::Running(_) => colors::YELLOW,
        Status::Scheduled | Status::Simulating => colors::BLUE,
        Status::Planning => colors::GREY1,
        Status::Aborted => colors::RED,
    }
}

#[tracing::instrument(level = "trace", skip_all)]
fn status_label(status: &Status) -> String {
    match status {
        Status::Done => "Done".to_string(),
        Status::Running(_) => "Running".to_string(),
        Status::Scheduled => "Queued".to_string(),
        Status::Planning => "Planning".to_string(),
        Status::Aborted => "Failed".to_string(),
        Status::Simulating => "Simulating".to_string(),
    }
}

#[tracing::instrument(level = "trace", skip_all)]
fn truncate_str(s: &str, max_chars: usize) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() > max_chars {
        let truncated: String = chars[..max_chars].iter().collect();
        format!("{truncated}…")
    } else {
        s.to_string()
    }
}
