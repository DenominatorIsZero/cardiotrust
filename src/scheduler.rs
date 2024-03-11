use bevy::prelude::*;
use std::{
    mem::discriminant,
    sync::{mpsc::channel, Mutex},
    thread,
};

use crate::{
    core::scenario::{run, Status},
    ScenarioList,
};

#[allow(clippy::module_name_repetitions)]
pub struct SchedulerPlugin;

impl Plugin for SchedulerPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<SchedulerState>()
            .init_resource::<NumberOfJobs>()
            .add_systems(
                Update,
                start_scenarios.run_if(in_state(SchedulerState::Available)),
            )
            .add_systems(Update, check_scenarios);
    }
}

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
#[allow(clippy::module_name_repetitions)]
pub enum SchedulerState {
    #[default]
    Paused,
    Available,
    Unavailale,
}

#[derive(Resource)]
pub struct NumberOfJobs {
    pub value: usize,
}

impl Default for NumberOfJobs {
    fn default() -> Self {
        Self { value: 4 }
    }
}

#[allow(clippy::needless_pass_by_value)]
pub fn start_scenarios(
    mut commands: Commands,
    mut scenario_list: ResMut<ScenarioList>,
    number_of_jobs: Res<NumberOfJobs>,
) {
    if let Some(entry) = scenario_list
        .entries
        .iter_mut()
        .find(|entry| *entry.scenario.get_status() == Status::Scheduled)
    {
        let send_scenario = entry.scenario.clone();
        let (epoch_tx, epoch_rx) = channel();
        let (summary_tx, summary_rx) = channel();
        let handle = thread::spawn(move || run(send_scenario, &epoch_tx, &summary_tx));
        entry.scenario.set_running(0);
        entry.join_handle = Some(handle);
        entry.epoch_rx = Some(Mutex::new(epoch_rx));
        entry.summary_rx = Some(Mutex::new(summary_rx));
    }
    if scenario_list
        .entries
        .iter()
        .filter(|entry| {
            discriminant(entry.scenario.get_status()) == discriminant(&Status::Running(1))
        })
        .count()
        >= number_of_jobs.value
    {
        commands.insert_resource(NextState(Some(SchedulerState::Unavailale)));
    }
}

/// .
///
/// # Panics
///
/// Panics if a running scenario has no epoch receiver, summary receiver or
/// join handle.
#[allow(clippy::needless_pass_by_value)]
pub fn check_scenarios(
    mut commands: Commands,
    mut scenario_list: ResMut<ScenarioList>,
    number_of_jobs: Res<NumberOfJobs>,
    scheduler_state: Res<State<SchedulerState>>,
) {
    scenario_list
        .entries
        .iter_mut()
        .filter(|entry| {
            discriminant(entry.scenario.get_status()) == discriminant(&Status::Running(1))
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
                        entry.scenario.save().expect("Scenarion to be parseable.");
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
        commands.insert_resource(NextState(Some(SchedulerState::Available)));
    }
}
