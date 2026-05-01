//! Gallery shell: root node, tab bar, toolbar, and scrollable body.

#![allow(clippy::type_complexity)]

use bevy::prelude::*;
use bevy_ui_widgets::{ControlOrientation, CoreScrollbarThumb, Scrollbar};

use super::{GalleryTab, ResultsViewRoot, ResultsViewState};

/// Marker on the CSS-grid node inside a tab body (cards live here).
#[derive(Component, Debug, Clone, Copy)]
pub struct GalleryGridNode {
    pub tab: GalleryTab,
}
use std::path::PathBuf;

use super::card::spawn_gallery_cards;
use crate::{
    ui::{bevy_shell::content_area::ContentSlot, colors},
    ActiveLoadedScenario, LoadedScenario, ScenarioList, SelectedSenario,
};

// ── Components ─────────────────────────────────────────────────────────────────

/// Marker on a gallery tab button.
#[derive(Component, Debug, Clone, Copy)]
pub struct GalleryTabButton {
    pub tab: GalleryTab,
}

/// Marker on the accent bar inside a gallery tab button.
#[derive(Component, Debug, Clone, Copy)]
pub struct GalleryTabAccent {
    pub tab: GalleryTab,
}

/// Marker on the label text inside a gallery tab button.
#[derive(Component, Debug, Clone, Copy)]
pub struct GalleryTabLabel {
    pub tab: GalleryTab,
}

/// Marker on a gallery tab body container.
#[derive(Component, Debug, Clone, Copy)]
pub struct GalleryTabBody {
    pub tab: GalleryTab,
}

/// Marker for the framed scroll area that owns one tab's content + scrollbar.
#[derive(Component, Debug, Clone, Copy)]
pub struct GalleryTabFrame {
    pub tab: GalleryTab,
}

/// Marker for the actual scrollable node inside a tab body.
#[derive(Component, Debug, Clone, Copy)]
pub struct GalleryScrollArea {
    pub tab: GalleryTab,
}

/// Marker on the "Generate All in Tab" button.
#[derive(Component, Debug)]
pub struct GenerateAllInTabButton;

/// Marker on the "Generate All" button.
#[derive(Component, Debug)]
pub struct GenerateAllButton;

/// Marker on the batch progress label.
#[derive(Component, Debug)]
pub struct BatchProgressLabel;

/// Marker on the "Export to .npy" button.
#[derive(Component, Debug)]
pub struct ExportNpyButton;

/// Marker on the "Export as APNG" button.
#[derive(Component, Debug)]
pub struct ExportApngButton;

/// Marker on the "Export as MP4" button.
#[derive(Component, Debug)]
pub struct ExportMp4Button;

/// Marker on the export status label.
#[derive(Component, Debug)]
pub struct ExportStatusLabel;

// ── Spawn / despawn ────────────────────────────────────────────────────────────

/// Spawns the Results gallery root node as a child of [`ContentSlot`].
///
/// Also eagerly loads `data` and `results` from disk for the selected scenario
/// so that image generation can access them immediately.
#[tracing::instrument(skip_all)]
pub fn spawn_results_view(
    mut commands: Commands,
    content_slots: Query<Entity, With<ContentSlot>>,
    mut view_state: ResMut<ResultsViewState>,
    mut image_cache: ResMut<super::ResultImageCache>,
    mut anim_cache: ResMut<super::ResultAnimCache>,
    scenario_list: Res<ScenarioList>,
    selected: Res<SelectedSenario>,
    mut active_loaded_scenario: ResMut<ActiveLoadedScenario>,
) {
    let Ok(slot) = content_slots.single() else {
        return;
    };

    // Load data + results from disk so generation systems can access them.
    if let Some(index) = selected.index {
        if let Some(entry) = scenario_list.entries.get(index) {
            match entry.load_payload() {
                Ok(payload) => {
                    active_loaded_scenario.0 = Some(LoadedScenario::from_bundle(entry, payload));
                }
                Err(e) => {
                    error!("Failed to load scenario payload for Results view: {e}");
                    active_loaded_scenario.0 = None;
                }
            }
        }
    }

    *view_state = ResultsViewState::default();

    if let Some(index) = selected.index {
        if let Some(entry) = scenario_list.entries.get(index) {
            preload_existing_results(
                &entry.storage,
                entry.scenario.get_id(),
                &mut image_cache,
                &mut anim_cache,
                view_state.playback_speed,
            );
        }
    }

    let root = commands
        .spawn((
            ResultsViewRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                min_height: Val::Px(0.0),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(colors::BG0),
        ))
        .id();

    commands.entity(slot).add_child(root);

    spawn_tab_bar(&mut commands, root, &view_state);
    spawn_toolbar(&mut commands, root);

    for tab in [
        GalleryTab::SpatialMaps,
        GalleryTab::Metrics,
        GalleryTab::Losses,
        GalleryTab::TimeFunctions,
    ] {
        let body = spawn_tab_body(&mut commands, root, tab, &view_state);
        spawn_gallery_cards(&mut commands, body, tab);
    }

    spawn_action_bar(&mut commands, root);
}

/// Despawns all [`ResultsViewRoot`] entities.
#[tracing::instrument(skip_all)]
pub fn despawn_results_view(mut commands: Commands, roots: Query<Entity, With<ResultsViewRoot>>) {
    for entity in &roots {
        commands.entity(entity).despawn();
    }
}

// ── Tab bar ────────────────────────────────────────────────────────────────────

#[tracing::instrument(skip_all)]
fn spawn_tab_bar(commands: &mut Commands, parent: Entity, view_state: &ResultsViewState) {
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
        (GalleryTab::SpatialMaps, "SPATIAL MAPS"),
        (GalleryTab::Metrics, "METRICS"),
        (GalleryTab::Losses, "LOSSES"),
        (GalleryTab::TimeFunctions, "TIME FUNCTIONS"),
    ];

    commands.entity(tab_bar).with_children(|bar| {
        for (tab, label) in tabs {
            spawn_tab_button(bar, tab, label, view_state.active_tab == tab);
        }
    });
}

#[tracing::instrument(skip_all)]
fn spawn_tab_button(parent: &mut ChildSpawnerCommands, tab: GalleryTab, label: &str, active: bool) {
    let text_color = if active { colors::FG0 } else { colors::GREY1 };
    let accent_display = if active { Display::Flex } else { Display::None };

    parent
        .spawn((
            GalleryTabButton { tab },
            Button,
            Node {
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                padding: UiRect::horizontal(Val::Px(16.0)),
                height: Val::Px(40.0),
                ..default()
            },
            BackgroundColor(Color::NONE),
        ))
        .with_children(|btn| {
            btn.spawn(Node {
                flex_grow: 1.0,
                ..default()
            });
            btn.spawn((
                GalleryTabLabel { tab },
                Text::new(label),
                TextFont {
                    font_size: 11.0,
                    ..default()
                },
                TextColor(text_color),
            ));
            btn.spawn((
                GalleryTabAccent { tab },
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
    tab: GalleryTab,
    view_state: &ResultsViewState,
) -> Entity {
    let display = if view_state.active_tab == tab {
        Display::Flex
    } else {
        Display::None
    };

    // Outer scrollable container — fills remaining height, clips + scrolls.
    let body = commands
        .spawn((
            GalleryTabBody { tab },
            GalleryTabFrame { tab },
            Node {
                display,
                grid_template_columns: vec![
                    RepeatedGridTrack::flex(1, 1.0),
                    RepeatedGridTrack::px(1, 10.0),
                ],
                width: Val::Percent(100.0),
                flex_grow: 1.0,
                min_height: Val::Px(0.0),
                column_gap: Val::Px(8.0),
                padding: UiRect::all(Val::Px(8.0)),
                overflow: Overflow::clip(),
                ..default()
            },
            BackgroundColor(colors::BG0),
        ))
        .id();
    commands.entity(parent).add_child(body);

    let scroll_area = commands
        .spawn((
            GalleryScrollArea { tab },
            Node {
                flex_direction: FlexDirection::Column,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                min_height: Val::Px(0.0),
                overflow: Overflow::scroll_y(),
                ..default()
            },
            ScrollPosition::default(),
            BackgroundColor(colors::BG0),
        ))
        .id();
    commands.entity(body).add_child(scroll_area);

    // Inner CSS-grid node — cards are children of this.
    // Column count is driven reactively by `update_gallery_grid_columns`.
    let grid = commands
        .spawn((
            GalleryGridNode { tab },
            Node {
                width: Val::Percent(100.0),
                display: Display::Grid,
                grid_template_columns: RepeatedGridTrack::flex(3, 1.0),
                column_gap: Val::Px(16.0),
                row_gap: Val::Px(16.0),
                padding: UiRect::all(Val::Px(16.0)),
                ..default()
            },
        ))
        .id();
    commands.entity(scroll_area).add_child(grid);

    let scrollbar = commands
        .spawn((
            Node {
                width: Val::Px(10.0),
                height: Val::Percent(100.0),
                min_height: Val::Px(0.0),
                ..default()
            },
            Scrollbar::new(scroll_area, ControlOrientation::Vertical, 24.0),
            BackgroundColor(colors::BG1),
        ))
        .with_children(|bar| {
            bar.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    border_radius: BorderRadius::all(Val::Px(5.0)),
                    ..default()
                },
                BackgroundColor(colors::GREY1),
                CoreScrollbarThumb,
            ));
        })
        .id();
    commands.entity(body).add_child(scrollbar);

    // Return the grid — cards are spawned directly into it.
    grid
}

#[tracing::instrument(skip_all)]
fn preload_existing_results(
    storage: &crate::core::scenario::ScenarioStorage,
    scenario_id: &str,
    image_cache: &mut super::ResultImageCache,
    anim_cache: &mut super::ResultAnimCache,
    playback_speed: f32,
) {
    for tab in [
        GalleryTab::SpatialMaps,
        GalleryTab::Metrics,
        GalleryTab::Losses,
        GalleryTab::TimeFunctions,
    ] {
        for image_type in super::card::static_cards_for_tab(tab) {
            if image_cache.0.contains_key(&image_type) {
                continue;
            }
            let path = storage.image_path(scenario_id, &image_type.to_string());
            let Some(path) = path.is_file().then_some(path) else {
                continue;
            };
            image_cache.0.insert(
                image_type,
                super::ResultImageState::Loading {
                    channel: spawn_image_load(path),
                },
            );
        }

        for anim_type in super::card::anim_cards_for_tab(tab) {
            if anim_cache.0.contains_key(&anim_type) {
                continue;
            }
            let anim_dir = storage.animation_dir(scenario_id, anim_type.dir_name());
            let Some(frame_paths) = super::generate::detect_existing_frames(&anim_dir) else {
                continue;
            };
            let channels = frame_paths
                .into_iter()
                .map(spawn_image_load)
                .collect::<Vec<_>>();
            let loaded = vec![None; channels.len()];
            let _ = playback_speed;
            anim_cache
                .0
                .insert(anim_type, super::AnimState::Loading { channels, loaded });
        }
    }
}

#[tracing::instrument(level = "trace")]
fn spawn_image_load(path: PathBuf) -> super::AsyncChannel<(Vec<u8>, u32, u32)> {
    let channel = super::new_channel::<(Vec<u8>, u32, u32)>();
    let writer = channel.clone();
    std::thread::spawn(move || {
        let result = load_image_bytes(&path);
        if let Ok(mut guard) = writer.lock() {
            *guard = Some(result);
        }
    });
    channel
}

#[tracing::instrument(level = "trace", skip_all)]
fn load_image_bytes(path: &std::path::Path) -> anyhow::Result<(Vec<u8>, u32, u32)> {
    let img = image::open(path)
        .map_err(|e| anyhow::anyhow!("Failed to open image {}: {}", path.display(), e))?;
    let rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();
    Ok((rgba.into_raw(), w, h))
}

// ── Toolbar row ────────────────────────────────────────────────────────────────

#[tracing::instrument(skip_all)]
fn spawn_toolbar(commands: &mut Commands, parent: Entity) {
    let toolbar = commands
        .spawn((
            Node {
                flex_direction: FlexDirection::Row,
                width: Val::Percent(100.0),
                height: Val::Px(40.0),
                flex_shrink: 0.0,
                padding: UiRect::horizontal(Val::Px(12.0)),
                column_gap: Val::Px(8.0),
                align_items: AlignItems::Center,
                border: UiRect::bottom(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(colors::BG1),
            BorderColor::all(colors::BG3),
        ))
        .id();
    commands.entity(parent).add_child(toolbar);

    commands.entity(toolbar).with_children(|row| {
        row.spawn((
            GenerateAllInTabButton,
            Button,
            Node {
                padding: UiRect::axes(Val::Px(10.0), Val::Px(4.0)),
                border_radius: BorderRadius::all(Val::Px(4.0)),
                ..default()
            },
            BackgroundColor(colors::BG3),
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new("Generate Tab"),
                TextFont {
                    font_size: 11.0,
                    ..default()
                },
                TextColor(colors::FG0),
            ));
        });

        row.spawn((
            GenerateAllButton,
            Button,
            Node {
                padding: UiRect::axes(Val::Px(10.0), Val::Px(4.0)),
                border_radius: BorderRadius::all(Val::Px(4.0)),
                ..default()
            },
            BackgroundColor(colors::BG3),
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new("Generate All"),
                TextFont {
                    font_size: 11.0,
                    ..default()
                },
                TextColor(colors::FG0),
            ));
        });

        row.spawn(Node {
            flex_grow: 1.0,
            ..default()
        });

        row.spawn((
            BatchProgressLabel,
            Text::new(""),
            TextFont {
                font_size: 11.0,
                ..default()
            },
            TextColor(colors::GREY1),
        ));
    });
}

// ── Action bar ─────────────────────────────────────────────────────────────────

#[tracing::instrument(skip_all)]
fn spawn_action_bar(commands: &mut Commands, parent: Entity) {
    let bar = commands
        .spawn((
            Node {
                flex_direction: FlexDirection::Row,
                width: Val::Percent(100.0),
                height: Val::Px(44.0),
                flex_shrink: 0.0,
                padding: UiRect::horizontal(Val::Px(12.0)),
                column_gap: Val::Px(8.0),
                align_items: AlignItems::Center,
                border: UiRect::top(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(colors::BG1),
            BorderColor::all(colors::BG3),
        ))
        .id();
    commands.entity(parent).add_child(bar);

    commands.entity(bar).with_children(|row| {
        spawn_action_button(row, "Export .npy", ExportNpyButton);
        row.spawn(Node {
            flex_grow: 1.0,
            ..default()
        });
        spawn_action_button(row, "Export APNG", ExportApngButton);
        spawn_action_button(row, "Export MP4", ExportMp4Button);
        row.spawn((
            ExportStatusLabel,
            Text::new(""),
            TextFont {
                font_size: 11.0,
                ..default()
            },
            TextColor(colors::GREY1),
        ));
    });
}

#[tracing::instrument(skip_all)]
fn spawn_action_button<M: Component>(parent: &mut ChildSpawnerCommands, label: &str, marker: M) {
    parent
        .spawn((
            marker,
            Button,
            Node {
                padding: UiRect::axes(Val::Px(10.0), Val::Px(4.0)),
                border_radius: BorderRadius::all(Val::Px(4.0)),
                ..default()
            },
            BackgroundColor(colors::BG3),
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new(label),
                TextFont {
                    font_size: 11.0,
                    ..default()
                },
                TextColor(colors::FG0),
            ));
        });
}

// ── Tab click handler ─────────────────────────────────────────────────────────

/// Handles gallery tab button clicks.
#[tracing::instrument(skip_all)]
pub fn handle_gallery_tab_click(
    tabs: Query<(&GalleryTabButton, &Interaction), (With<Button>, Changed<Interaction>)>,
    mut view_state: ResMut<ResultsViewState>,
    mut bodies: Query<(&GalleryTabBody, &mut Node)>,
    mut accents: Query<(&GalleryTabAccent, &mut Node), Without<GalleryTabBody>>,
    mut labels: Query<(&GalleryTabLabel, &mut TextColor)>,
) {
    for (tab_btn, interaction) in &tabs {
        if *interaction != Interaction::Pressed {
            continue;
        }
        view_state.active_tab = tab_btn.tab;
        let active = tab_btn.tab;

        for (body, mut node) in &mut bodies {
            node.display = if body.tab == active {
                Display::Flex
            } else {
                Display::None
            };
        }
        for (accent, mut node) in &mut accents {
            node.display = if accent.tab == active {
                Display::Flex
            } else {
                Display::None
            };
        }
        for (label, mut color) in &mut labels {
            color.0 = if label.tab == active {
                colors::FG0
            } else {
                colors::GREY1
            };
        }
    }
}

// ── Responsive grid columns ────────────────────────────────────────────────────

/// Adjusts the CSS-grid column count on all `GalleryGridNode`s based on the
/// window width (same breakpoints as the Explorer view).
#[tracing::instrument(skip_all)]
pub fn update_gallery_grid_columns(
    windows: Query<&Window>,
    mut grids: Query<&mut Node, With<GalleryGridNode>>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    // Approximate content area width by subtracting sidebar (≤ 200 px).
    let content_width = window.resolution.width() - 200.0;
    let cols: u16 = if content_width >= 1400.0 {
        4
    } else if content_width >= 1000.0 {
        3
    } else if content_width >= 700.0 {
        2
    } else {
        1
    };
    for mut node in &mut grids {
        node.grid_template_columns = RepeatedGridTrack::flex(cols, 1.0);
    }
}

/// Reactively updates gallery tab visuals when `ResultsViewState` changes.
#[tracing::instrument(skip_all)]
pub fn update_gallery_tab_visuals(
    view_state: Res<ResultsViewState>,
    mut accents: Query<(&GalleryTabAccent, &mut Node), Without<GalleryTabFrame>>,
    mut labels: Query<(&GalleryTabLabel, &mut TextColor)>,
    mut frames: Query<(&GalleryTabFrame, &mut Node), Without<GalleryTabAccent>>,
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
    for (frame, mut node) in &mut frames {
        node.display = if frame.tab == view_state.active_tab {
            Display::Flex
        } else {
            Display::None
        };
    }
}
