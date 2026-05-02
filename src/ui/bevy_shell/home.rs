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

use super::{
    content_area::ContentSlot,
    project::{is_project_switch_blocked, PROJECT_SWITCH_BLOCKED_MESSAGE},
};
use crate::{ui::colors, PendingProjectLoad, ProjectState, ScenarioList};

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

#[derive(Component, Debug)]
pub struct OpenProjectPanel;

#[derive(Component, Debug)]
pub struct ProjectSwitchBlockedNotice;

#[derive(Component, Debug)]
pub struct HomeProjectSwitchDisabled;

// ── Spawn / despawn ───────────────────────────────────────────────────────────

/// Spawns the Home view node tree as a child of [`ContentSlot`].
#[tracing::instrument(skip_all)]
pub fn spawn_home_view(
    mut commands: Commands,
    content_slots: Query<Entity, With<ContentSlot>>,
    existing_roots: Query<Entity, With<HomeViewRoot>>,
    project_state: Res<ProjectState>,
    scenario_list: Res<ScenarioList>,
) {
    if !existing_roots.is_empty() {
        return;
    }

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
                    let switching_blocked = is_project_switch_blocked(&scenario_list);

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
                    spawn_open_project_panel(col, switching_blocked);

                    // Recent Projects panel
                    spawn_recent_projects_panel(col, &project_state.recent, switching_blocked);

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
fn spawn_open_project_panel(parent: &mut ChildSpawnerCommands, switching_blocked: bool) {
    parent
        .spawn((
            OpenProjectPanel,
            Node {
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(16.0)),
                row_gap: Val::Px(12.0),
                border: UiRect::all(Val::Px(1.0)),
                border_radius: BorderRadius::all(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(colors::BG1),
            BorderColor::all(colors::GREY1),
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
            let mut open_button = panel.spawn((
                OpenProjectButton,
                Button,
                Node {
                    padding: UiRect::axes(Val::Px(16.0), Val::Px(10.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border_radius: BorderRadius::all(Val::Px(4.0)),
                    ..default()
                },
                BackgroundColor(if switching_blocked {
                    colors::BG3
                } else {
                    colors::ORANGE
                }),
            ));
            if switching_blocked {
                open_button.insert(HomeProjectSwitchDisabled);
            }
            open_button.with_children(|btn| {
                btn.spawn((
                    Text::new("Open Project Folder"),
                    TextFont {
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(colors::BG0),
                ));
            });

            if switching_blocked {
                panel.spawn((
                    ProjectSwitchBlockedNotice,
                    Text::new(PROJECT_SWITCH_BLOCKED_MESSAGE),
                    TextFont {
                        font_size: 13.0,
                        ..default()
                    },
                    TextColor(colors::YELLOW),
                ));
            }
        });
}

/// Spawns the Recent Projects panel listing `recent` paths.
#[tracing::instrument(skip_all)]
fn spawn_recent_projects_panel(
    parent: &mut ChildSpawnerCommands,
    recent: &[PathBuf],
    switching_blocked: bool,
) {
    parent
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(16.0)),
                row_gap: Val::Px(8.0),
                border_radius: BorderRadius::all(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(colors::BG1),
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
                    let mut recent_button = panel.spawn((
                        RecentProjectEntry { path: path.clone() },
                        Button,
                        Node {
                            padding: UiRect::axes(Val::Px(12.0), Val::Px(8.0)),
                            align_items: AlignItems::Center,
                            border_radius: BorderRadius::all(Val::Px(4.0)),
                            ..default()
                        },
                        BackgroundColor(colors::BG3),
                    ));
                    if switching_blocked {
                        recent_button.insert(HomeProjectSwitchDisabled);
                    }
                    recent_button.with_children(|btn| {
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
                border_radius: BorderRadius::all(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(colors::BG1),
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
                            border_radius: BorderRadius::all(Val::Px(4.0)),
                            ..default()
                        },
                        BackgroundColor(colors::BG3),
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
#[allow(clippy::type_complexity)]
pub fn handle_open_project_button(
    buttons: Query<
        (&Interaction, Option<&HomeProjectSwitchDisabled>),
        (With<OpenProjectButton>, Changed<Interaction>),
    >,
    mut dialog_rx: ResMut<FolderDialogReceiver>,
) {
    for (interaction, disabled) in &buttons {
        if disabled.is_none() && *interaction == Interaction::Pressed && dialog_rx.0.is_none() {
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
    mut pending_project_load: ResMut<PendingProjectLoad>,
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
                    pending_project_load.0 = Some(path);
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
    entries: Query<
        (
            &RecentProjectEntry,
            &Interaction,
            Option<&HomeProjectSwitchDisabled>,
        ),
        Changed<Interaction>,
    >,
    mut project_state: ResMut<ProjectState>,
    mut pending_project_load: ResMut<PendingProjectLoad>,
) {
    for (entry, interaction, disabled) in &entries {
        if disabled.is_none() && *interaction == Interaction::Pressed {
            let path = entry.path.clone();
            project_state.push_recent(path.clone());
            if let Err(e) = project_state.save_recent() {
                warn!("Failed to save recent projects: {}", e);
            }
            pending_project_load.0 = Some(path);
        }
    }
}

#[tracing::instrument(skip_all)]
pub fn sync_home_project_switch_guard(
    scenario_list: Res<ScenarioList>,
    open_project_panel: Query<Entity, With<OpenProjectPanel>>,
    mut open_buttons: Query<
        (
            Entity,
            &mut BackgroundColor,
            Option<&HomeProjectSwitchDisabled>,
        ),
        With<OpenProjectButton>,
    >,
    recent_buttons: Query<(Entity, Option<&HomeProjectSwitchDisabled>), With<RecentProjectEntry>>,
    notices: Query<Entity, With<ProjectSwitchBlockedNotice>>,
    mut commands: Commands,
) {
    if !scenario_list.is_changed() {
        return;
    }

    let blocked = is_project_switch_blocked(&scenario_list);

    for (entity, mut background, disabled) in &mut open_buttons {
        background.0 = if blocked { colors::BG3 } else { colors::ORANGE };
        if blocked && disabled.is_none() {
            commands.entity(entity).insert(HomeProjectSwitchDisabled);
        } else if !blocked && disabled.is_some() {
            commands
                .entity(entity)
                .remove::<HomeProjectSwitchDisabled>();
        }
    }

    for (entity, disabled) in &recent_buttons {
        if blocked && disabled.is_none() {
            commands.entity(entity).insert(HomeProjectSwitchDisabled);
        } else if !blocked && disabled.is_some() {
            commands
                .entity(entity)
                .remove::<HomeProjectSwitchDisabled>();
        }
    }

    let has_notice = !notices.is_empty();
    if blocked && !has_notice {
        if let Ok(panel) = open_project_panel.single() {
            commands.entity(panel).with_children(|parent| {
                parent.spawn((
                    ProjectSwitchBlockedNotice,
                    Text::new(PROJECT_SWITCH_BLOCKED_MESSAGE),
                    TextFont {
                        font_size: 13.0,
                        ..default()
                    },
                    TextColor(colors::YELLOW),
                ));
            });
        }
    } else if !blocked {
        for notice in &notices {
            commands.entity(notice).despawn();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        core::scenario::{Scenario, ScenarioStorage},
        ScenarioBundle,
    };

    #[test]
    fn recent_project_click_is_ignored_when_disabled() {
        let mut app = App::new();
        app.insert_resource(ProjectState::default());
        app.insert_resource(PendingProjectLoad::default());
        app.add_systems(Update, handle_recent_project_click);
        app.world_mut().spawn((
            RecentProjectEntry {
                path: PathBuf::from("/tmp/blocked-project"),
            },
            Interaction::Pressed,
            HomeProjectSwitchDisabled,
        ));

        app.update();

        assert!(app.world().resource::<PendingProjectLoad>().0.is_none());
        assert!(app.world().resource::<ProjectState>().recent.is_empty());
    }

    #[test]
    fn sync_home_guard_marks_buttons_disabled_for_running_scenarios() {
        let mut app = App::new();
        let mut scenario_list = ScenarioList::empty();
        let mut running = Scenario::build(Some("running".to_string()));
        running.set_running(1);
        scenario_list.entries.push(ScenarioBundle::new(
            running,
            ScenarioStorage::new("./results/tests"),
        ));

        app.insert_resource(scenario_list);
        app.add_systems(Update, sync_home_project_switch_guard);

        let panel = app.world_mut().spawn(OpenProjectPanel).id();
        let open_button = app
            .world_mut()
            .spawn((OpenProjectButton, BackgroundColor(colors::ORANGE)))
            .id();
        let recent_button = app
            .world_mut()
            .spawn(RecentProjectEntry {
                path: PathBuf::from("/tmp/project"),
            })
            .id();

        app.update();
        app.world_mut().flush();

        assert!(app
            .world()
            .entity(open_button)
            .contains::<HomeProjectSwitchDisabled>());
        assert!(app
            .world()
            .entity(recent_button)
            .contains::<HomeProjectSwitchDisabled>());
        assert!(app.world().get_entity(panel).is_ok());
        let mut query = app.world_mut().query::<&ProjectSwitchBlockedNotice>();
        let notice_count = query.iter(app.world()).count();
        assert_eq!(notice_count, 1);
    }
}
