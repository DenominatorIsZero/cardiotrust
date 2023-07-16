use std::{
    mem::discriminant,
    sync::{mpsc::channel, Mutex},
    thread,
};

use bevy::prelude::*;

use crate::{
    core::scenario::{run_scenario, summary, Status},
    ScenarioList,
};

pub struct SchedulerPlugin;

impl Plugin for SchedulerPlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<SchedulerState>()
            .add_system(start_scenarios.run_if(in_state(SchedulerState::Available)))
            .add_system(check_scenarios.run_if(in_state(SchedulerState::Unavailale)));
    }
}

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum SchedulerState {
    #[default]
    Paused,
    Available,
    Unavailale,
}

pub fn start_scenarios(mut commands: Commands, mut scenario_list: ResMut<ScenarioList>) {
    match scenario_list
        .entries
        .iter_mut()
        .filter(|entry| *entry.scenario.get_status() == Status::Scheduled)
        .next()
    {
        Some(entry) => {
            println!("Starting scenario with id {}", entry.scenario.get_id());
            let send_scenario = entry.scenario.clone();
            let (epoch_tx, epoch_rx) = channel();
            let (summary_tx, summary_rx) = channel();
            let handle = thread::spawn(move || run_scenario(send_scenario, epoch_tx, summary_tx));
            entry.scenario.set_running(0);
            entry.join_handle = Some(handle);
            entry.epoch_rx = Some(Mutex::new(epoch_rx));
            entry.summary_rx = Some(Mutex::new(summary_rx));
            println!("Moving scheduler to state unavailable.");
            commands.insert_resource(NextState(Some(SchedulerState::Unavailale)));
        }
        None => (),
    }
}

pub fn check_scenarios(mut commands: Commands, mut scenario_list: ResMut<ScenarioList>) {
    scenario_list
        .entries
        .iter_mut()
        .filter(|entry| {
            discriminant(entry.scenario.get_status()) == discriminant(&Status::Running(1))
        })
        .for_each(|entry| {
            match &entry.epoch_rx {
                Some(epoch_rx) => {
                    let epoch_rx = epoch_rx.lock().unwrap();
                    let epoch = epoch_rx.try_recv();
                    if epoch.is_ok() {
                        entry.scenario.set_running(epoch.unwrap());
                    }
                }
                None => panic!("Running scenario has to epoch receiver."),
            }
            match &entry.summary_rx {
                Some(summary_rx) => {
                    let summary_rx = summary_rx.lock().unwrap();
                    let summary = summary_rx.try_recv();
                    if summary.is_ok() {
                        entry.scenario.summary = Some(summary.unwrap());
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
                        entry.scenario.save().unwrap();
                    }
                }
                None => panic!("Running scenario does not a join handle."),
            }
        });

    if !scenario_list
        .entries
        .iter()
        .any(|entry| discriminant(entry.scenario.get_status()) == discriminant(&Status::Running(1)))
    {
        println!("Moving scheduler to state available.");
        commands.insert_resource(NextState(Some(SchedulerState::Available)));
    }
}
