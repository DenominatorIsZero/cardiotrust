//! Bevy-native Scheduler dashboard view.

use bevy::prelude::*;
use bevy_ui_widgets::{ControlOrientation, CoreScrollbarThumb, Scrollbar};

use crate::{
    core::scenario::Status,
    scheduler::{NumberOfJobs, SchedulerState},
    ui::{bevy_shell::content_area::ContentSlot, colors, UiState},
    Scenario, ScenarioList,
};

const UNKNOWN_ETA_LABEL: &str = "Unknown until running progress is observed";
const MAX_CONCURRENCY: usize = 32;

#[derive(Component, Debug)]
pub struct SchedulerViewRoot;

#[derive(Component, Debug)]
struct SchedulerSummaryValue {
    field: SummaryField,
}

#[derive(Component, Debug)]
struct SchedulerActionButton {
    action: SchedulerAction,
}

#[derive(Component, Debug)]
struct SchedulerActionButtonLabel {
    action: SchedulerAction,
}

#[derive(Component, Debug)]
struct SchedulerConcurrencyButton {
    delta: i32,
}

#[derive(Component, Debug)]
struct SchedulerConcurrencyButtonLabel {
    delta: i32,
}

#[derive(Component, Debug)]
struct SchedulerConcurrencyValue;

#[derive(Component, Debug)]
struct SchedulerSectionCount {
    section: SchedulerSection,
}

#[derive(Component, Debug)]
struct SchedulerSectionBody {
    section: SchedulerSection,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum SummaryField {
    State,
    RunningCount,
    ScheduledCount,
    QueueEta,
}

#[derive(Component, Debug, Clone, Copy, Eq, PartialEq)]
enum SchedulerAction {
    Start,
    Stop,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum SchedulerSection {
    Running,
    Scheduled,
}

#[derive(Debug)]
struct SchedulerSnapshot<'a> {
    running: Vec<&'a Scenario>,
    scheduled: Vec<&'a Scenario>,
    observed_seconds_per_epoch: Option<f32>,
    queue_eta: Option<i64>,
}

#[derive(Debug)]
pub struct SchedulerViewPlugin;

impl Plugin for SchedulerViewPlugin {
    #[tracing::instrument(level = "info", skip(app))]
    fn build(&self, app: &mut App) {
        let scheduler_condition = in_state(UiState::Scheduler);

        app.add_systems(OnEnter(UiState::Scheduler), spawn_scheduler_view)
            .add_systems(OnExit(UiState::Scheduler), despawn_scheduler_view)
            .add_systems(
                Update,
                (
                    sync_scheduler_summary,
                    sync_scheduler_controls,
                    sync_scheduler_sections,
                    handle_scheduler_action_buttons,
                    handle_scheduler_concurrency_buttons,
                )
                    .run_if(scheduler_condition),
            );
    }
}

#[tracing::instrument(skip_all)]
pub fn spawn_scheduler_view(
    mut commands: Commands,
    content_slots: Query<Entity, With<ContentSlot>>,
    scenario_list: Res<ScenarioList>,
    scheduler_state: Res<State<SchedulerState>>,
    number_of_jobs: Res<NumberOfJobs>,
) {
    let Ok(slot) = content_slots.single() else {
        return;
    };

    let snapshot = SchedulerSnapshot::from_list(&scenario_list);

    let root = commands
        .spawn((
            SchedulerViewRoot,
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
    spawn_dashboard_header(
        &mut commands,
        root,
        &snapshot,
        scheduler_state.get(),
        number_of_jobs.value,
    );
    spawn_scheduler_body(&mut commands, root, &snapshot);
}

#[tracing::instrument(skip_all)]
pub fn despawn_scheduler_view(
    mut commands: Commands,
    roots: Query<Entity, With<SchedulerViewRoot>>,
) {
    for entity in &roots {
        commands.entity(entity).despawn();
    }
}

#[tracing::instrument(skip_all)]
#[allow(clippy::trivially_copy_pass_by_ref)]
fn spawn_dashboard_header(
    commands: &mut Commands,
    parent: Entity,
    snapshot: &SchedulerSnapshot,
    scheduler_state: &SchedulerState,
    number_of_jobs: usize,
) {
    let header = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(16.0),
                padding: UiRect::all(Val::Px(20.0)),
                border: UiRect::bottom(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(colors::BG1),
            BorderColor::all(colors::BG3),
        ))
        .id();
    commands.entity(parent).add_child(header);

    commands.entity(header).with_children(|header| {
        header
            .spawn(Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(12.0),
                padding: UiRect::all(Val::Px(16.0)),
                border: UiRect::all(Val::Px(1.0)),
                border_radius: BorderRadius::all(Val::Px(8.0)),
                ..default()
            })
            .with_children(|panel| {
                panel
                    .spawn(Node {
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(16.0),
                        flex_wrap: FlexWrap::Wrap,
                        ..default()
                    })
                    .with_children(|row| {
                        row.spawn((
                            Text::new("Dashboard"),
                            TextFont {
                                font_size: 18.0,
                                ..default()
                            },
                            TextColor(colors::FG0),
                        ));
                    });

                panel
                    .spawn(Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(8.0),
                        ..default()
                    })
                    .with_children(|summary| {
                        spawn_summary_row(
                            summary,
                            "Scheduler State",
                            SummaryField::State,
                            scheduler_state_label(scheduler_state),
                        );
                        spawn_summary_row(
                            summary,
                            "Running",
                            SummaryField::RunningCount,
                            snapshot.running.len().to_string(),
                        );
                        spawn_summary_row(
                            summary,
                            "Scheduled",
                            SummaryField::ScheduledCount,
                            snapshot.scheduled.len().to_string(),
                        );
                        spawn_summary_row(
                            summary,
                            "Queue Remaining",
                            SummaryField::QueueEta,
                            format_eta_value(snapshot.queue_eta),
                        );
                    });

                panel
                    .spawn(Node {
                        width: Val::Percent(100.0),
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(16.0),
                        flex_wrap: FlexWrap::Wrap,
                        ..default()
                    })
                    .with_children(|row| {
                        row.spawn(Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(12.0),
                            align_items: AlignItems::Center,
                            ..default()
                        })
                        .with_children(|controls| {
                            spawn_action_button(
                                controls,
                                SchedulerAction::Start,
                                action_enabled(SchedulerAction::Start, scheduler_state),
                            );
                            spawn_action_button(
                                controls,
                                SchedulerAction::Stop,
                                action_enabled(SchedulerAction::Stop, scheduler_state),
                            );
                        });

                        row.spawn(Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(12.0),
                            align_items: AlignItems::Center,
                            flex_wrap: FlexWrap::Wrap,
                            ..default()
                        })
                        .with_children(|controls| {
                            controls.spawn((
                                Text::new("Runtime Concurrency"),
                                TextFont {
                                    font_size: 13.0,
                                    ..default()
                                },
                                TextColor(colors::GREY1),
                            ));
                            spawn_concurrency_button(controls, -1, number_of_jobs > 1);
                            controls.spawn((
                                SchedulerConcurrencyValue,
                                Text::new(number_of_jobs.to_string()),
                                TextFont {
                                    font_size: 16.0,
                                    ..default()
                                },
                                TextColor(colors::FG0),
                            ));
                            spawn_concurrency_button(controls, 1, number_of_jobs < MAX_CONCURRENCY);
                        });
                    });
            });
    });
}

#[tracing::instrument(skip_all)]
fn spawn_summary_row(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    field: SummaryField,
    value: String,
) {
    parent
        .spawn(Node {
            width: Val::Percent(100.0),
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::FlexStart,
            column_gap: Val::Px(16.0),
            ..default()
        })
        .with_children(|row| {
            row.spawn((
                Text::new(label),
                TextFont {
                    font_size: 13.0,
                    ..default()
                },
                TextColor(colors::GREY1),
            ));
            row.spawn((
                SchedulerSummaryValue { field },
                Text::new(value),
                TextFont {
                    font_size: 15.0,
                    ..default()
                },
                TextColor(colors::FG0),
            ));
        });
}

#[tracing::instrument(skip_all)]
fn spawn_action_button(parent: &mut ChildSpawnerCommands, action: SchedulerAction, enabled: bool) {
    parent
        .spawn((
            SchedulerActionButton { action },
            Button,
            Node {
                padding: UiRect::axes(Val::Px(16.0), Val::Px(10.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border_radius: BorderRadius::all(Val::Px(6.0)),
                ..default()
            },
            BackgroundColor(action_button_color(action, enabled)),
        ))
        .with_children(|button| {
            button.spawn((
                SchedulerActionButtonLabel { action },
                Text::new(action_label(action)),
                TextFont {
                    font_size: 13.0,
                    ..default()
                },
                TextColor(action_button_text_color(enabled)),
            ));
        });
}

#[tracing::instrument(skip_all)]
fn spawn_concurrency_button(parent: &mut ChildSpawnerCommands, delta: i32, enabled: bool) {
    parent
        .spawn((
            SchedulerConcurrencyButton { delta },
            Button,
            Node {
                width: Val::Px(28.0),
                height: Val::Px(28.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border_radius: BorderRadius::all(Val::Px(4.0)),
                ..default()
            },
            BackgroundColor(if enabled { colors::BG3 } else { colors::BG1 }),
        ))
        .with_children(|button| {
            button.spawn((
                SchedulerConcurrencyButtonLabel { delta },
                Text::new(if delta < 0 { "-" } else { "+" }),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(action_button_text_color(enabled)),
            ));
        });
}

#[tracing::instrument(skip_all)]
fn spawn_scheduler_body(commands: &mut Commands, parent: Entity, snapshot: &SchedulerSnapshot) {
    let body = commands
        .spawn((
            Node {
                display: Display::Grid,
                width: Val::Percent(100.0),
                flex_grow: 1.0,
                min_height: Val::Px(0.0),
                grid_template_columns: vec![
                    RepeatedGridTrack::flex(1, 1.0),
                    RepeatedGridTrack::px(1, 10.0),
                ],
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

    commands.entity(scroll_area).with_children(|scroll| {
        scroll
            .spawn(Node {
                flex_direction: FlexDirection::Column,
                width: Val::Percent(100.0),
                row_gap: Val::Px(16.0),
                padding: UiRect::all(Val::Px(16.0)),
                ..default()
            })
            .with_children(|content| {
                spawn_section(content, SchedulerSection::Running, snapshot);
                spawn_section(content, SchedulerSection::Scheduled, snapshot);
            });
    });

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
}

#[tracing::instrument(skip_all)]
fn spawn_section(
    parent: &mut ChildSpawnerCommands,
    section: SchedulerSection,
    snapshot: &SchedulerSnapshot,
) {
    parent
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(12.0),
                padding: UiRect::all(Val::Px(16.0)),
                border: UiRect::all(Val::Px(1.0)),
                border_radius: BorderRadius::all(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(colors::BG1),
            BorderColor::all(colors::BG3),
        ))
        .with_children(|section_root| {
            section_root
                .spawn(Node {
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_children(|header| {
                    header.spawn((
                        Text::new(section_title(section)),
                        TextFont {
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(colors::FG0),
                    ));
                    header.spawn((
                        SchedulerSectionCount { section },
                        Text::new(section_count(section, snapshot).to_string()),
                        TextFont {
                            font_size: 13.0,
                            ..default()
                        },
                        TextColor(colors::GREY1),
                    ));
                });

            section_root.spawn((
                SchedulerSectionBody { section },
                Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(10.0),
                    ..default()
                },
            ));
        });
}

#[tracing::instrument(skip_all)]
fn sync_scheduler_summary(
    scenario_list: Res<ScenarioList>,
    scheduler_state: Res<State<SchedulerState>>,
    mut summary_values: Query<(&SchedulerSummaryValue, &mut Text)>,
) {
    let snapshot = SchedulerSnapshot::from_list(&scenario_list);

    for (field, mut text) in &mut summary_values {
        text.0 = match field.field {
            SummaryField::State => scheduler_state_label(scheduler_state.get()),
            SummaryField::RunningCount => snapshot.running.len().to_string(),
            SummaryField::ScheduledCount => snapshot.scheduled.len().to_string(),
            SummaryField::QueueEta => format_eta_value(snapshot.queue_eta),
        };
    }
}

#[tracing::instrument(skip_all)]
#[allow(clippy::type_complexity)]
fn sync_scheduler_controls(
    scheduler_state: Res<State<SchedulerState>>,
    number_of_jobs: Res<NumberOfJobs>,
    mut button_backgrounds: ParamSet<(
        Query<(&SchedulerActionButton, &mut BackgroundColor)>,
        Query<(&SchedulerConcurrencyButton, &mut BackgroundColor)>,
    )>,
    mut button_labels: ParamSet<(
        Query<(&SchedulerActionButtonLabel, &mut TextColor)>,
        Query<(&SchedulerConcurrencyButtonLabel, &mut TextColor)>,
    )>,
    mut concurrency_value: Query<&mut Text, With<SchedulerConcurrencyValue>>,
) {
    for (button, mut background) in &mut button_backgrounds.p0() {
        let enabled = action_enabled(button.action, scheduler_state.get());
        background.0 = action_button_color(button.action, enabled);
    }

    for (label, mut color) in &mut button_labels.p0() {
        color.0 = action_button_text_color(action_enabled(label.action, scheduler_state.get()));
    }

    for (button, mut background) in &mut button_backgrounds.p1() {
        let enabled = concurrency_button_enabled(button.delta, number_of_jobs.value);
        background.0 = if enabled { colors::BG3 } else { colors::BG1 };
    }

    for (label, mut color) in &mut button_labels.p1() {
        color.0 = action_button_text_color(concurrency_button_enabled(
            label.delta,
            number_of_jobs.value,
        ));
    }

    for mut text in &mut concurrency_value {
        text.0 = number_of_jobs.value.to_string();
    }
}

#[tracing::instrument(skip_all)]
fn sync_scheduler_sections(
    mut commands: Commands,
    scenario_list: Res<ScenarioList>,
    body_nodes: Query<(Entity, &SchedulerSectionBody, Option<&Children>)>,
    mut count_labels: Query<(&SchedulerSectionCount, &mut Text)>,
) {
    let snapshot = SchedulerSnapshot::from_list(&scenario_list);

    for (count, mut text) in &mut count_labels {
        text.0 = section_count(count.section, &snapshot).to_string();
    }

    for (entity, body, children) in &body_nodes {
        if let Some(children) = children {
            for child in children.iter() {
                commands.entity(child).despawn();
            }
        }
        commands
            .entity(entity)
            .with_children(|parent| match body.section {
                SchedulerSection::Running => {
                    if snapshot.running.is_empty() {
                        spawn_empty_section_message(parent, "No running scenarios");
                    } else {
                        for scenario in &snapshot.running {
                            spawn_running_row(parent, scenario);
                        }
                    }
                }
                SchedulerSection::Scheduled => {
                    if snapshot.scheduled.is_empty() {
                        spawn_empty_section_message(parent, "No scheduled scenarios");
                    } else {
                        for scenario in &snapshot.scheduled {
                            spawn_scheduled_row(
                                parent,
                                scenario,
                                snapshot.observed_seconds_per_epoch,
                            );
                        }
                    }
                }
            });
    }
}

#[tracing::instrument(skip_all)]
fn spawn_empty_section_message(parent: &mut ChildSpawnerCommands, message: &str) {
    parent.spawn((
        Text::new(message),
        TextFont {
            font_size: 13.0,
            ..default()
        },
        TextColor(colors::GREY1),
    ));
}

#[tracing::instrument(skip_all)]
fn spawn_running_row(parent: &mut ChildSpawnerCommands, scenario: &Scenario) {
    let current_epoch = match scenario.get_status() {
        Status::Running(epoch) => *epoch,
        _ => 0,
    };
    let total_epochs = scenario.config.algorithm.epochs;
    let progress_percent = scenario.get_progress() * 100.0;

    spawn_scenario_row(
        parent,
        scenario.get_id(),
        scenario.get_status_str(),
        &format!("Epoch {current_epoch}/{total_epochs}"),
        &format!("Progress {progress_percent:.1}%"),
        scenario_eta_text(scenario),
    );
}

#[tracing::instrument(skip_all)]
#[allow(clippy::cast_precision_loss)]
fn spawn_scheduled_row(
    parent: &mut ChildSpawnerCommands,
    scenario: &Scenario,
    observed_seconds_per_epoch: Option<f32>,
) {
    let total_epochs = scenario.config.algorithm.epochs;
    let predicted_eta = observed_seconds_per_epoch
        .map(|seconds_per_epoch| format_duration(seconds_per_epoch * total_epochs as f32));

    spawn_scenario_row(
        parent,
        scenario.get_id(),
        scenario.get_status_str(),
        &format!("Queued epochs {total_epochs}"),
        "Waiting for scheduler capacity",
        predicted_eta.unwrap_or_else(|| UNKNOWN_ETA_LABEL.to_string()),
    );
}

#[tracing::instrument(skip_all)]
fn spawn_scenario_row(
    parent: &mut ChildSpawnerCommands,
    identifier: &str,
    status: String,
    detail_a: &str,
    detail_b: &str,
    eta: String,
) {
    parent
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.0),
                padding: UiRect::all(Val::Px(14.0)),
                border: UiRect::all(Val::Px(1.0)),
                border_radius: BorderRadius::all(Val::Px(6.0)),
                ..default()
            },
            BackgroundColor(colors::BG0),
            BorderColor::all(colors::BG3),
        ))
        .with_children(|row| {
            row.spawn((
                Text::new(identifier.to_string()),
                TextFont {
                    font_size: 15.0,
                    ..default()
                },
                TextColor(colors::FG0),
            ));

            row.spawn(Node {
                flex_direction: FlexDirection::Row,
                flex_wrap: FlexWrap::Wrap,
                column_gap: Val::Px(16.0),
                row_gap: Val::Px(6.0),
                ..default()
            })
            .with_children(|details| {
                spawn_detail(details, "Status", &status, colors::BLUE);
                spawn_detail(details, "Detail", detail_a, colors::GREY2);
                spawn_detail(details, "Info", detail_b, colors::GREY2);
                spawn_detail(details, "ETA", &eta, colors::AQUA);
            });
        });
}

#[tracing::instrument(skip_all)]
fn spawn_detail(parent: &mut ChildSpawnerCommands, label: &str, value: &str, accent: Color) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(2.0),
            min_width: Val::Px(120.0),
            ..default()
        })
        .with_children(|column| {
            column.spawn((
                Text::new(label),
                TextFont {
                    font_size: 11.0,
                    ..default()
                },
                TextColor(colors::GREY1),
            ));
            column.spawn((
                Text::new(value.to_string()),
                TextFont {
                    font_size: 13.0,
                    ..default()
                },
                TextColor(accent),
            ));
        });
}

#[tracing::instrument(skip_all)]
#[allow(clippy::type_complexity)]
fn handle_scheduler_action_buttons(
    buttons: Query<(&SchedulerActionButton, &Interaction), (With<Button>, Changed<Interaction>)>,
    scheduler_state: Res<State<SchedulerState>>,
    mut next_scheduler_state: ResMut<NextState<SchedulerState>>,
) {
    for (button, interaction) in &buttons {
        if *interaction != Interaction::Pressed
            || !action_enabled(button.action, scheduler_state.get())
        {
            continue;
        }

        match button.action {
            SchedulerAction::Start => next_scheduler_state.set(SchedulerState::Available),
            SchedulerAction::Stop => next_scheduler_state.set(SchedulerState::Paused),
        }
    }
}

#[tracing::instrument(skip_all)]
#[allow(clippy::type_complexity)]
fn handle_scheduler_concurrency_buttons(
    buttons: Query<
        (&SchedulerConcurrencyButton, &Interaction),
        (With<Button>, Changed<Interaction>),
    >,
    mut number_of_jobs: ResMut<NumberOfJobs>,
) {
    for (button, interaction) in &buttons {
        if *interaction != Interaction::Pressed
            || !concurrency_button_enabled(button.delta, number_of_jobs.value)
        {
            continue;
        }

        if button.delta < 0 {
            number_of_jobs.value -= 1;
        } else {
            number_of_jobs.value += 1;
        }
    }
}

impl<'a> SchedulerSnapshot<'a> {
    #[tracing::instrument(level = "trace", skip_all)]
    fn from_list(scenario_list: &'a ScenarioList) -> Self {
        let running = scenario_list
            .entries
            .iter()
            .filter_map(|entry| match entry.scenario.get_status() {
                Status::Running(_) | Status::Simulating => Some(&entry.scenario),
                _ => None,
            })
            .collect::<Vec<_>>();
        let scheduled = scenario_list
            .entries
            .iter()
            .filter_map(|entry| {
                (*entry.scenario.get_status() == Status::Scheduled).then_some(&entry.scenario)
            })
            .collect::<Vec<_>>();
        let observed_seconds_per_epoch = observed_seconds_per_epoch(&running);
        let queue_eta = queue_remaining_seconds(&running, &scheduled, observed_seconds_per_epoch);

        Self {
            running,
            scheduled,
            observed_seconds_per_epoch,
            queue_eta,
        }
    }
}

#[tracing::instrument(level = "trace", skip_all)]
#[allow(clippy::cast_precision_loss)]
fn observed_seconds_per_epoch(running: &[&Scenario]) -> Option<f32> {
    let mut samples = Vec::new();

    for scenario in running {
        let Status::Running(epoch) = scenario.get_status() else {
            continue;
        };
        if *epoch == 0 {
            continue;
        }
        let (Some(started), Some(last_update)) = (scenario.started, scenario.last_update) else {
            continue;
        };

        let elapsed_seconds = (last_update - started).num_milliseconds() as f32 / 1000.0;
        if elapsed_seconds <= 0.0 {
            continue;
        }

        samples.push(elapsed_seconds / *epoch as f32);
    }

    if samples.is_empty() {
        None
    } else {
        Some(samples.iter().sum::<f32>() / samples.len() as f32)
    }
}

#[tracing::instrument(level = "trace", skip_all)]
#[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
fn queue_remaining_seconds(
    running: &[&Scenario],
    scheduled: &[&Scenario],
    observed_seconds_per_epoch: Option<f32>,
) -> Option<i64> {
    let seconds_per_epoch = observed_seconds_per_epoch?;
    let running_epochs = running
        .iter()
        .map(|scenario| running_remaining_epochs(scenario))
        .sum::<usize>();
    let scheduled_epochs = scheduled
        .iter()
        .map(|scenario| scenario.config.algorithm.epochs)
        .sum::<usize>();
    let remaining_epochs = running_epochs + scheduled_epochs;

    Some((seconds_per_epoch * remaining_epochs as f32).round() as i64)
}

#[tracing::instrument(level = "trace", skip_all)]
fn running_remaining_epochs(scenario: &Scenario) -> usize {
    match scenario.get_status() {
        Status::Running(epoch) => scenario.config.algorithm.epochs.saturating_sub(*epoch),
        Status::Simulating => scenario.config.algorithm.epochs,
        _ => 0,
    }
}

#[tracing::instrument(level = "trace")]
#[allow(clippy::trivially_copy_pass_by_ref)]
fn scheduler_state_label(state: &SchedulerState) -> String {
    match state {
        SchedulerState::Paused => "Paused".to_string(),
        SchedulerState::Available => "Running".to_string(),
        SchedulerState::Unavailale => "At Capacity".to_string(),
    }
}

#[tracing::instrument(level = "trace")]
fn action_label(action: SchedulerAction) -> &'static str {
    match action {
        SchedulerAction::Start => "Start",
        SchedulerAction::Stop => "Stop",
    }
}

#[tracing::instrument(level = "trace")]
#[allow(clippy::trivially_copy_pass_by_ref)]
fn action_enabled(action: SchedulerAction, scheduler_state: &SchedulerState) -> bool {
    match action {
        SchedulerAction::Start => *scheduler_state == SchedulerState::Paused,
        SchedulerAction::Stop => *scheduler_state != SchedulerState::Paused,
    }
}

#[tracing::instrument(level = "trace")]
fn action_button_color(action: SchedulerAction, enabled: bool) -> Color {
    if enabled {
        match action {
            SchedulerAction::Start => colors::ORANGE,
            SchedulerAction::Stop => colors::RED,
        }
    } else {
        colors::BG3
    }
}

#[tracing::instrument(level = "trace")]
fn action_button_text_color(enabled: bool) -> Color {
    if enabled {
        colors::BG0
    } else {
        colors::GREY1
    }
}

#[tracing::instrument(level = "trace")]
fn concurrency_button_enabled(delta: i32, number_of_jobs: usize) -> bool {
    if delta < 0 {
        number_of_jobs > 1
    } else {
        number_of_jobs < MAX_CONCURRENCY
    }
}

#[tracing::instrument(level = "trace")]
fn section_title(section: SchedulerSection) -> &'static str {
    match section {
        SchedulerSection::Running => "Running",
        SchedulerSection::Scheduled => "Scheduled",
    }
}

#[tracing::instrument(level = "trace", skip_all)]
fn section_count(section: SchedulerSection, snapshot: &SchedulerSnapshot) -> usize {
    match section {
        SchedulerSection::Running => snapshot.running.len(),
        SchedulerSection::Scheduled => snapshot.scheduled.len(),
    }
}

#[tracing::instrument(level = "trace")]
fn format_eta_value(queue_eta: Option<i64>) -> String {
    queue_eta.map_or_else(|| UNKNOWN_ETA_LABEL.to_string(), format_duration_seconds)
}

#[tracing::instrument(level = "trace", skip_all)]
fn scenario_eta_text(scenario: &Scenario) -> String {
    let etc = scenario.get_etc();
    if etc.is_empty() || etc.ends_with("???") {
        UNKNOWN_ETA_LABEL.to_string()
    } else {
        etc.trim_start_matches("ETC: ").to_string()
    }
}

#[tracing::instrument(level = "trace")]
#[allow(clippy::cast_possible_truncation)]
fn format_duration(seconds: f32) -> String {
    format_duration_seconds(seconds.round() as i64)
}

#[tracing::instrument(level = "trace")]
fn format_duration_seconds(total_seconds: i64) -> String {
    let seconds = total_seconds.max(0);
    let days = seconds / 86_400;
    let hours = (seconds / 3_600) % 24;
    let minutes = (seconds / 60) % 60;
    let secs = seconds % 60;

    if days > 0 {
        format!("{days}d {hours}h")
    } else if hours > 0 {
        format!("{hours}h {minutes}m")
    } else if minutes > 0 {
        format!("{minutes}m {secs}s")
    } else {
        format!("{secs}s")
    }
}

#[cfg(test)]
mod tests {
    use chrono::{Duration, Utc};

    use super::*;
    use crate::core::scenario::Status;

    #[test]
    fn observed_seconds_per_epoch_uses_running_progress_only() {
        let now = Utc::now();

        let mut running_a = Scenario::build(Some("running-a".to_string()));
        running_a.set_running(2);
        running_a.started = Some(now - Duration::seconds(20));
        running_a.last_update = Some(now);

        let mut running_b = Scenario::build(Some("running-b".to_string()));
        running_b.set_running(4);
        running_b.started = Some(now - Duration::seconds(24));
        running_b.last_update = Some(now);

        let mut simulating = Scenario::build(Some("simulating".to_string()));
        simulating.set_simulating();

        let running = vec![&running_a, &running_b, &simulating];
        let observed = observed_seconds_per_epoch(&running);

        assert_eq!(observed, Some(8.0));
    }

    #[test]
    fn queue_remaining_seconds_combines_running_and_scheduled_epochs() {
        let now = Utc::now();

        let mut running = Scenario::build(Some("running".to_string()));
        running.config.algorithm.epochs = 10;
        running.set_running(4);
        running.started = Some(now - Duration::seconds(40));
        running.last_update = Some(now);

        let mut scheduled = Scenario::build(Some("scheduled".to_string()));
        scheduled.config.algorithm.epochs = 5;
        scheduled.force_scheduled();

        let queue_eta = queue_remaining_seconds(&[&running], &[&scheduled], Some(10.0));

        assert_eq!(queue_eta, Some(110));
    }

    #[test]
    fn scenario_eta_text_reports_unknown_when_eta_is_not_observable() {
        let running = Scenario::build(Some("running".to_string()));

        assert_eq!(scenario_eta_text(&running), UNKNOWN_ETA_LABEL);
        assert_eq!(running.get_status(), &Status::Planning);
    }
}
