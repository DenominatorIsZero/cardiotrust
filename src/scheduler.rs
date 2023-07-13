use std::mem::discriminant;

use bevy::{prelude::*, transform::commands};

use crate::{core::scenario::Status, Scenarios};

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

pub fn start_scenarios(mut commands: Commands, mut scenarios: ResMut<Scenarios>) {
    match scenarios
        .scenarios
        .iter_mut()
        .filter(|scenario| *scenario.get_status() == Status::Scheduled)
        .next()
    {
        Some(scenario) => {
            println!("Starting scenario with id {}", scenario.get_id());
            scenario.run();
            println!("Moving scheduler to state unavailable.");
            commands.insert_resource(NextState(Some(SchedulerState::Unavailale)));
        }
        None => (),
    }
}

pub fn check_scenarios(mut commands: Commands, scenarios: ResMut<Scenarios>) {
    if !scenarios
        .scenarios
        .iter()
        .any(|scenario| discriminant(scenario.get_status()) == discriminant(&Status::Running(1)))
    {
        println!("Moving scheduler to state available.");
        commands.insert_resource(NextState(Some(SchedulerState::Available)));
    }
}
