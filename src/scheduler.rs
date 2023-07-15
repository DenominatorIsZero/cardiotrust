use std::{mem::discriminant, thread};

use bevy::{prelude::*, transform::commands};

use crate::{
    core::scenario::{run_scenario, Status},
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
            let handle = thread::spawn(move || run_scenario(send_scenario));
            entry.scenario.set_running(0);
            entry.join_handle = Some(handle);
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
        .for_each(|entry| match &entry.join_handle {
            Some(join_handle) => {
                if join_handle.is_finished() {
                    entry.scenario.set_done();
                    entry.join_handle = None;
                }
            }
            None => panic!("Running scenario does not a join handle."),
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
