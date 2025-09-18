use std::{
    mem::discriminant,
    sync::{mpsc::channel, Mutex},
    thread,
};

use bevy::prelude::*;
use tracing::error;

use crate::{
    core::scenario::{run, Status},
    ScenarioList,
};

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub struct SchedulerPlugin;

impl Plugin for SchedulerPlugin {
    #[tracing::instrument(level = "info", skip(app))]
    fn build(&self, app: &mut App) {
        info!("Initializing scheduler plugin.");
        app.init_state::<SchedulerState>()
            .init_resource::<NumberOfJobs>()
            .add_systems(
                Update,
                start_scenarios.run_if(in_state(SchedulerState::Available)),
            )
            .add_systems(Update, check_scenarios);
    }
}

/// An enum representing the possible states of the scheduler.
///
/// `Paused` - The default state where the scheduler is not actively running scenarios.
///
/// `Available` - The scheduler is available to start running scenarios.
///  
/// `Unavailable` - The scheduler is currently occupied running scenarios and is unavailable to start new ones.
#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
#[allow(clippy::module_name_repetitions)]
pub enum SchedulerState {
    #[default]
    Paused,
    Available,
    Unavailale,
}

#[derive(Resource, Debug)]
pub struct NumberOfJobs {
    pub value: usize,
}

impl Default for NumberOfJobs {
    /// Returns a `NumberOfJobs` instance with the default value of 4 for `value`.
    #[tracing::instrument(level = "info")]
    fn default() -> Self {
        info!("Initializing number of jobs resource.");
        Self { value: 4 }
    }
}

/// Starts scenarios from the scenario list that are scheduled, spawning threads
/// to run them and tracking their status. Limits number of concurrent scenarios
/// based on provided resource. Updates state if max concurrent reached.
#[allow(clippy::needless_pass_by_value)]
#[tracing::instrument(level = "trace", skip(commands))]
pub fn start_scenarios(
    mut commands: Commands,
    mut scenario_list: ResMut<ScenarioList>,
    number_of_jobs: Res<NumberOfJobs>,
) {
    trace!("Running start_scenarios system.");
    if scenario_list
        .entries
        .iter()
        .filter(|entry| {
            discriminant(entry.scenario.get_status()) == discriminant(&Status::Running(1))
                || entry.scenario.get_status() == &Status::Simulating
        })
        .count()
        >= number_of_jobs.value
    {
        commands.insert_resource(NextState::Pending(SchedulerState::Unavailale));
    } else if let Some(entry) = scenario_list
        .entries
        .iter_mut()
        .find(|entry| *entry.scenario.get_status() == Status::Scheduled)
    {
        let send_scenario = entry.scenario.clone();
        let (epoch_tx, epoch_rx) = channel();
        let (summary_tx, summary_rx) = channel();
        let handle = thread::spawn(move || {
            if let Err(e) = run(send_scenario, &epoch_tx, &summary_tx) {
                tracing::error!("Scenario failed: {:?}", e);
            }
        });
        entry.scenario.set_simulating();
        entry.join_handle = Some(handle);
        entry.epoch_rx = Some(Mutex::new(epoch_rx));
        entry.summary_rx = Some(Mutex::new(summary_rx));
    }
}

/// Checks the status of running scenarios, updating their epoch and summary if
/// available. Removes finished scenarios from tracking. Checks if the scheduler
/// should be marked as available based on running scenario count and current
/// scheduler state.
///
/// # Panics
///
/// Panics if a running scenario has no epoch receiver, summary receiver or
/// join handle.
#[allow(clippy::needless_pass_by_value)]
#[tracing::instrument(level = "trace", skip(commands))]
pub fn check_scenarios(
    mut commands: Commands,
    mut scenario_list: ResMut<ScenarioList>,
    number_of_jobs: Res<NumberOfJobs>,
    scheduler_state: Res<State<SchedulerState>>,
) {
    trace!("Running check_scenarios system.");
    scenario_list
        .entries
        .iter_mut()
        .filter(|entry| {
            discriminant(entry.scenario.get_status()) == discriminant(&Status::Running(1))
                || entry.scenario.get_status() == &Status::Simulating
        })
        .for_each(|entry| {
            match &entry.epoch_rx {
                Some(epoch_rx) => {
                    let epoch = epoch_rx
                        .lock()
                        .expect("Lock to not already be held")
                        .try_recv();
                    if let Ok(epoch) = epoch {
                        entry.scenario.set_running(epoch);
                    }
                }
                None => panic!("Running scenario has to epoch receiver."),
            }
            match &entry.summary_rx {
                Some(summary_rx) => {
                    let summary = summary_rx
                        .lock()
                        .expect("Lock to not already be held")
                        .try_recv();
                    if let Ok(summary) = summary {
                        entry.scenario.summary = Some(summary);
                    }
                }
                None => panic!("Running scenario has no summary receiver."),
            }

            match &entry.join_handle {
                Some(join_handle) => {
                    if join_handle.is_finished() {
                        entry.scenario.set_done();
                        entry.join_handle = None;
                        entry.epoch_rx = None;
                        entry.summary_rx = None;
                        if let Err(e) = entry.scenario.save() {
                            error!("Failed to save scenario {}: {}", entry.scenario.get_id(), e);
                        }
                    }
                }
                None => panic!("Running scenario does not have a join handle."),
            }
        });

    if (scenario_list
        .entries
        .iter()
        .filter(|entry| {
            discriminant(entry.scenario.get_status()) == discriminant(&Status::Running(1))
        })
        .count()
        < number_of_jobs.value)
        && (scheduler_state.get() == &SchedulerState::Unavailale)
    {
        commands.insert_resource(NextState::Pending(SchedulerState::Available));
    }
}
