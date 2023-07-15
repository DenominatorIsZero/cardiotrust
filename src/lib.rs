#![allow(dead_code)]
pub mod core;
pub mod scheduler;
pub mod ui;
pub mod vis;

use std::{
    fs,
    path::Path,
    sync::{mpsc::Receiver, Arc, Mutex},
    thread::JoinHandle,
};

use bevy::prelude::*;

use crate::core::scenario::Scenario;

#[derive(Resource, Debug)]
pub struct SelectedSenario {
    pub index: Option<usize>,
}

impl Default for SelectedSenario {
    fn default() -> Self {
        SelectedSenario { index: Some(0) }
    }
}

#[derive(Debug)]
pub struct ScenarioBundle {
    pub scenario: Scenario,
    pub join_handle: Option<JoinHandle<()>>,
    pub epoch_rx: Option<Mutex<Receiver<usize>>>,
}

#[derive(Resource, Debug)]
pub struct ScenarioList {
    pub entries: Vec<ScenarioBundle>,
}

impl Default for ScenarioList {
    fn default() -> Self {
        let mut scenario_list = ScenarioList {
            entries: Vec::<ScenarioBundle>::new(),
        };
        let dir = Path::new("./results");
        for entry in
            fs::read_dir(dir).expect(&format!("No such directory {}", dir.to_string_lossy()))
        {
            let entry = entry.expect("Invalid path found");
            let path = entry.path();
            if path.is_dir() {
                scenario_list.entries.push(ScenarioBundle {
                    scenario: Scenario::load(&path),
                    join_handle: None,
                    epoch_rx: None,
                });
            }
        }
        scenario_list
    }
}
