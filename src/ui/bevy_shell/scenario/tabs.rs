//! Tab bar for the scenario editor.
//!
//! Spawns a horizontal row of tabs (Simulation / Algorithm / Ground Truth / Initial Model).
//! Clicking a tab updates `ScenarioViewState.active_tab` and shows/hides
//! the corresponding body node via `Display::Flex / None`.

#![allow(clippy::type_complexity)]

use bevy::prelude::*;

use super::{ScenarioTab, ScenarioViewState};
use crate::ui::colors;

// ── Components ────────────────────────────────────────────────────────────────

/// Marker on a tab button node.
#[derive(Component, Debug, Clone, Copy)]
pub struct TabButton {
    pub tab: ScenarioTab,
}

/// Marker on the label text node inside a tab button.
#[derive(Component, Debug, Clone, Copy)]
pub struct TabLabel {
    pub tab: ScenarioTab,
}

/// Marker on a tab body container node.
#[derive(Component, Debug, Clone, Copy)]
pub struct TabBody {
    pub tab: ScenarioTab,
}

/// Marker for the bottom-border accent node on a tab button (used for active style).
#[derive(Component, Debug, Clone, Copy)]
pub struct TabAccent {
    pub tab: ScenarioTab,
}

// ── Spawn ─────────────────────────────────────────────────────────────────────

/// Spawns the tab bar and four body containers as children of `parent`.
///
/// Returns `(tab_bar_entity, sim_body, algo_body, gt_body, init_body)` so the caller
/// can populate each body.
#[tracing::instrument(skip_all)]
pub fn spawn_tab_bar(
    commands: &mut Commands,
    parent: Entity,
    view_state: &ScenarioViewState,
) -> (Entity, Entity, Entity, Entity, Entity) {
    // Tab bar row
    let tab_bar = commands
        .spawn((
            Node {
                flex_direction: FlexDirection::Row,
                width: Val::Percent(100.0),
                height: Val::Px(40.0),
                flex_shrink: 0.0,
                border: UiRect::bottom(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(colors::BG1),
            BorderColor::all(colors::BG3),
        ))
        .id();
    commands.entity(parent).add_child(tab_bar);

    let tabs = [
        (ScenarioTab::Simulation, "SIMULATION"),
        (ScenarioTab::Algorithm, "ALGORITHM"),
        (ScenarioTab::GroundTruth, "GROUND TRUTH"),
        (ScenarioTab::InitialModel, "INITIAL MODEL"),
    ];

    commands.entity(tab_bar).with_children(|bar| {
        for (tab, label) in tabs {
            spawn_tab_button(bar, tab, label, view_state.active_tab == tab);
        }
    });

    // Tab body scroll containers
    let sim_body = spawn_tab_body(commands, parent, ScenarioTab::Simulation, view_state);
    let algo_body = spawn_tab_body(commands, parent, ScenarioTab::Algorithm, view_state);
    let gt_body = spawn_tab_body(commands, parent, ScenarioTab::GroundTruth, view_state);
    let init_body = spawn_tab_body(commands, parent, ScenarioTab::InitialModel, view_state);

    (tab_bar, sim_body, algo_body, gt_body, init_body)
}

#[tracing::instrument(skip_all)]
fn spawn_tab_button(
    parent: &mut ChildSpawnerCommands,
    tab: ScenarioTab,
    label: &str,
    active: bool,
) {
    let text_color = if active { colors::FG0 } else { colors::GREY1 };
    let accent_display = if active { Display::Flex } else { Display::None };

    parent
        .spawn((
            TabButton { tab },
            Button,
            Node {
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                padding: UiRect::horizontal(Val::Px(20.0)),
                height: Val::Px(40.0),
                ..default()
            },
            BackgroundColor(Color::NONE),
        ))
        .with_children(|btn| {
            // Spacer at top
            btn.spawn(Node {
                flex_grow: 1.0,
                ..default()
            });

            // Label — carries TabLabel so update_tab_visuals can find it
            btn.spawn((
                TabLabel { tab },
                Text::new(label.to_string()),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(text_color),
            ));

            // Bottom accent (3px ORANGE bar for active state)
            btn.spawn((
                TabAccent { tab },
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(3.0),
                    display: accent_display,
                    ..default()
                },
                BackgroundColor(colors::ORANGE),
            ));
        });
}

#[tracing::instrument(skip_all)]
fn spawn_tab_body(
    commands: &mut Commands,
    parent: Entity,
    tab: ScenarioTab,
    view_state: &ScenarioViewState,
) -> Entity {
    let display = if view_state.active_tab == tab {
        Display::Flex
    } else {
        Display::None
    };

    let body = commands
        .spawn((
            TabBody { tab },
            Node {
                flex_direction: FlexDirection::Column,
                flex_grow: 1.0,
                width: Val::Percent(100.0),
                overflow: Overflow::scroll_y(),
                display,
                ..default()
            },
            BackgroundColor(colors::BG0),
        ))
        .id();
    commands.entity(parent).add_child(body);
    body
}

// ── Click handler ─────────────────────────────────────────────────────────────

/// Handles tab button clicks: updates state and shows/hides bodies.
#[tracing::instrument(skip_all)]
pub fn handle_tab_click(
    tabs: Query<(&TabButton, &Interaction), (With<Button>, Changed<Interaction>)>,
    mut view_state: ResMut<ScenarioViewState>,
    mut bodies: Query<(&TabBody, &mut Node), Without<TabAccent>>,
    mut accents: Query<(&TabAccent, &mut Node), Without<TabBody>>,
    mut labels: Query<(&TabLabel, &mut TextColor)>,
) {
    for (tab_btn, interaction) in &tabs {
        if *interaction != Interaction::Pressed {
            continue;
        }
        view_state.active_tab = tab_btn.tab;
        let active = tab_btn.tab;

        // Show/hide bodies
        for (body, mut node) in &mut bodies {
            node.display = if body.tab == active {
                Display::Flex
            } else {
                Display::None
            };
        }

        // Update accent bars
        for (accent, mut node) in &mut accents {
            node.display = if accent.tab == active {
                Display::Flex
            } else {
                Display::None
            };
        }

        // Update tab label text colors
        for (label, mut color) in &mut labels {
            color.0 = if label.tab == active {
                colors::FG0
            } else {
                colors::GREY1
            };
        }
    }
}

/// Reactively updates tab text colors and accents to match active tab.
#[tracing::instrument(skip_all)]
pub fn update_tab_visuals(
    view_state: Res<ScenarioViewState>,
    mut accents: Query<(&TabAccent, &mut Node)>,
    mut labels: Query<(&TabLabel, &mut TextColor)>,
) {
    if !view_state.is_changed() {
        return;
    }
    for (accent, mut node) in &mut accents {
        node.display = if accent.tab == view_state.active_tab {
            Display::Flex
        } else {
            Display::None
        };
    }
    for (label, mut color) in &mut labels {
        color.0 = if label.tab == view_state.active_tab {
            colors::FG0
        } else {
            colors::GREY1
        };
    }
}
