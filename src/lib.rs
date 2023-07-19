#![warn(clippy::pedantic, clippy::nursery)]
pub mod core;
pub mod scheduler;
pub mod ui;
pub mod vis;

use crate::core::scenario::summary::Summary;
use std::{
    fs,
    path::Path,
    sync::{mpsc::Receiver, Mutex},
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
        Self { index: Some(0) }
    }
}

#[derive(Debug)]
pub struct ScenarioBundle {
    pub scenario: Scenario,
    pub join_handle: Option<JoinHandle<()>>,
    pub epoch_rx: Option<Mutex<Receiver<usize>>>,
    pub summary_rx: Option<Mutex<Receiver<Summary>>>,
}

#[derive(Resource, Debug)]
pub struct ScenarioList {
    pub entries: Vec<ScenarioBundle>,
}

impl Default for ScenarioList {
    fn default() -> Self {
        let mut scenario_list = Self {
            entries: Vec::<ScenarioBundle>::new(),
        };
        let dir = Path::new("./results");
        for entry in fs::read_dir(dir)
            .unwrap_or_else(|_| panic!("No such directory {}", dir.to_string_lossy()))
        {
            let entry = entry.expect("Invalid path found");
            let path = entry.path();
            if path.is_dir() {
                scenario_list.entries.push(ScenarioBundle {
                    scenario: Scenario::load(&path),
                    join_handle: None,
                    epoch_rx: None,
                    summary_rx: None,
                });
            }
        }
        scenario_list
    }
}
