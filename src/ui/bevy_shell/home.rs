//! Home view — project selection panel shown at startup and when no project is
//! loaded.
//!
//! Spawned via `OnEnter(UiState::Home)` and despawned via `OnExit(UiState::Home)`.

use std::{
    path::PathBuf,
    sync::{
        mpsc::{self, Receiver},
        Mutex,
    },
};

use bevy::prelude::*;
use tracing::warn;

use super::content_area::ContentSlot;
use crate::{ui::colors, ProjectState};

// ── Folder-dialog channel resource ───────────────────────────────────────────

/// Holds the receiving end of a channel written to by the background folder-
/// dialog thread. `None` when no dialog is in flight.
///
/// The `Mutex` is required so `Receiver<PathBuf>` satisfies Bevy's `Resource`
/// bound (`Send + Sync`).
#[derive(Resource, Default)]
pub struct FolderDialogReceiver(pub Option<Mutex<Receiver<PathBuf>>>);

// ── Marker components ─────────────────────────────────────────────────────────

/// Marker for the root entity of the Home view node tree.
/// Despawning this (recursively) cleans up the entire view.
#[derive(Component, Debug)]
pub struct HomeViewRoot;

/// Marker for the "Open Project Folder" button.
#[derive(Component, Debug)]
pub struct OpenProjectButton;

/// Attached to each recent-project button; holds the path it represents.
#[derive(Component, Debug, Clone)]
pub struct RecentProjectEntry {
    pub path: PathBuf,
}

// ── Spawn / despawn ───────────────────────────────────────────────────────────

/// Spawns the Home view node tree as a child of [`ContentSlot`].
#[tracing::instrument(skip_all)]
pub fn spawn_home_view(
    mut commands: Commands,
    content_slots: Query<Entity, With<ContentSlot>>,
    project_state: Res<ProjectState>,
) {
    let Ok(slot) = content_slots.single() else {
        return;
    };

    let home_root = commands
        .spawn((
            HomeViewRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::FlexStart,
                overflow: Overflow::scroll_y(),
                ..default()
            },
            BackgroundColor(colors::BG0),
        ))
        .with_children(|outer| {
            // Centered column, max-width ~800px
            outer
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    max_width: Val::Px(800.0),
                    width: Val::Percent(100.0),
                    padding: UiRect::all(Val::Px(32.0)),
                    row_gap: Val::Px(24.0),
                    ..default()
                })
                .with_children(|col| {
                    // Title
                    col.spawn((
                        Text::new("CardioTrust"),
                        TextFont {
                            font_size: 36.0,
                            ..default()
                        },
                        TextColor(colors::FG0),
                    ));

                    // Subtitle
                    col.spawn((
                        Text::new("Cardiac Electrophysiological Simulation"),
                        TextFont {
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(colors::GREY1),
                    ));

                    // Open Project panel
                    spawn_open_project_panel(col);

                    // Recent Projects panel
                    spawn_recent_projects_panel(col, &project_state.recent);

                    // WASM-only demo placeholder
                    #[cfg(target_arch = "wasm32")]
                    spawn_demo_projects_panel(col);
                });
        })
        .id();

    commands.entity(slot).add_child(home_root);
}

/// Spawns the "Open Project Folder" panel.
#[tracing::instrument(skip_all)]
fn spawn_open_project_panel(parent: &mut ChildSpawnerCommands) {
    parent
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(16.0)),
                row_gap: Val::Px(12.0),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(colors::BG1),
            BorderColor(colors::GREY1),
            BorderRadius::all(Val::Px(8.0)),
        ))
        .with_children(|panel| {
            panel.spawn((
                Text::new("Open Project"),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(colors::FG0),
            ));

            // Open Project Folder button
            panel
                .spawn((
                    OpenProjectButton,
                    Button,
                    Node {
                        padding: UiRect::axes(Val::Px(16.0), Val::Px(10.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(colors::ORANGE),
                    BorderRadius::all(Val::Px(4.0)),
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new("Open Project Folder"),
                        TextFont {
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(colors::BG0),
                    ));
                });
        });
}

/// Spawns the Recent Projects panel listing `recent` paths.
#[tracing::instrument(skip_all)]
fn spawn_recent_projects_panel(parent: &mut ChildSpawnerCommands, recent: &[PathBuf]) {
    parent
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(16.0)),
                row_gap: Val::Px(8.0),
                ..default()
            },
            BackgroundColor(colors::BG1),
            BorderRadius::all(Val::Px(8.0)),
        ))
        .with_children(|panel| {
            panel.spawn((
                Text::new("Recent Projects"),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(colors::FG0),
            ));

            if recent.is_empty() {
                panel.spawn((
                    Text::new("No recent projects"),
                    TextFont {
                        font_size: 13.0,
                        ..default()
                    },
                    TextColor(colors::GREY1),
                ));
            } else {
                for path in recent.iter().take(8) {
                    let display = path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or_else(|| path.to_str().unwrap_or("(invalid path)"))
                        .to_owned();
                    panel
                        .spawn((
                            RecentProjectEntry { path: path.clone() },
                            Button,
                            Node {
                                padding: UiRect::axes(Val::Px(12.0), Val::Px(8.0)),
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(colors::BG3),
                            BorderRadius::all(Val::Px(4.0)),
                        ))
                        .with_children(|btn| {
                            btn.spawn((
                                Text::new(display),
                                TextFont {
                                    font_size: 13.0,
                                    ..default()
                                },
                                TextColor(colors::FG1),
                            ));
                        });
                }
            }
        });
}

/// WASM-only: spawns three placeholder "Demo Project" cards.
#[cfg(target_arch = "wasm32")]
#[tracing::instrument(skip_all)]
fn spawn_demo_projects_panel(parent: &mut ChildSpawnerCommands) {
    parent
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(16.0)),
                row_gap: Val::Px(8.0),
                ..default()
            },
            BackgroundColor(colors::BG1),
            BorderRadius::all(Val::Px(8.0)),
        ))
        .with_children(|panel| {
            panel.spawn((
                Text::new("Demo Projects"),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(colors::FG0),
            ));

            for label in ["Demo Project 1", "Demo Project 2", "Demo Project 3"] {
                panel
                    .spawn((
                        Node {
                            padding: UiRect::axes(Val::Px(12.0), Val::Px(8.0)),
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(colors::BG3),
                        BorderRadius::all(Val::Px(4.0)),
                    ))
                    .with_children(|card| {
                        card.spawn((
                            Text::new(format!("{label} (Coming Soon)")),
                            TextFont {
                                font_size: 13.0,
                                ..default()
                            },
                            TextColor(colors::GREY1),
                        ));
                    });
            }
        });
}

/// Despawns all [`HomeViewRoot`] entities when leaving the Home state.
#[tracing::instrument(skip_all)]
pub fn despawn_home_view(mut commands: Commands, roots: Query<Entity, With<HomeViewRoot>>) {
    for entity in &roots {
        commands.entity(entity).despawn();
    }
}

// ── Button handlers ───────────────────────────────────────────────────────────

/// Opens a native folder dialog on a background thread when the Open Project
/// button is pressed, so the Bevy main thread is never blocked.
///
/// The result is sent through [`FolderDialogReceiver`] and picked up each
/// frame by [`poll_folder_dialog`].
#[tracing::instrument(skip_all)]
pub fn handle_open_project_button(
    buttons: Query<&Interaction, (With<OpenProjectButton>, Changed<Interaction>)>,
    mut dialog_rx: ResMut<FolderDialogReceiver>,
) {
    for interaction in &buttons {
        if *interaction == Interaction::Pressed && dialog_rx.0.is_none() {
            #[cfg(not(target_arch = "wasm32"))]
            {
                let (tx, rx) = mpsc::channel();
                std::thread::spawn(move || {
                    if let Some(path) = rfd::FileDialog::new().pick_folder() {
                        // Ignore send errors — the receiver may have been dropped.
                        let _ = tx.send(path);
                    }
                });
                dialog_rx.0 = Some(Mutex::new(rx));
            }
        }
    }
}

/// Polls [`FolderDialogReceiver`] each frame and, when a path arrives, updates
/// [`ProjectState`] to trigger project loading.
#[tracing::instrument(skip_all)]
pub fn poll_folder_dialog(
    mut dialog_rx: ResMut<FolderDialogReceiver>,
    mut project_state: ResMut<ProjectState>,
) {
    let done = if let Some(mutex) = &dialog_rx.0 {
        // Lock can only fail if the spawned thread panicked, which we treat as
        // a cancelled dialog.
        match mutex.lock() {
            Err(_) => true, // thread panicked — discard
            Ok(rx) => match rx.try_recv() {
                Ok(path) => {
                    project_state.push_recent(path.clone());
                    if let Err(e) = project_state.save_recent() {
                        warn!("Failed to save recent projects: {}", e);
                    }
                    project_state.current_path = Some(path);
                    true
                }
                Err(mpsc::TryRecvError::Empty) => false,
                Err(mpsc::TryRecvError::Disconnected) => true, // dialog cancelled
            },
        }
    } else {
        false
    };
    if done {
        dialog_rx.0 = None;
    }
}

/// Loads a recent project when its button is pressed.
#[tracing::instrument(skip_all)]
pub fn handle_recent_project_click(
    entries: Query<(&RecentProjectEntry, &Interaction), Changed<Interaction>>,
    mut project_state: ResMut<ProjectState>,
) {
    for (entry, interaction) in &entries {
        if *interaction == Interaction::Pressed {
            let path = entry.path.clone();
            project_state.push_recent(path.clone());
            if let Err(e) = project_state.save_recent() {
                warn!("Failed to save recent projects: {}", e);
            }
            project_state.current_path = Some(path);
        }
    }
}
