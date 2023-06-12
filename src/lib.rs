#![allow(dead_code)]
pub mod core;
pub mod ui;
pub mod vis;

use std::{fs, path::Path};

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

#[derive(Resource, Debug)]
pub struct Scenarios {
    pub scenarios: Vec<Scenario>,
}

impl Default for Scenarios {
    fn default() -> Self {
        let mut scenarios = Scenarios {
            scenarios: Vec::<Scenario>::new(),
        };
        let dir = Path::new("./results");
        for entry in
            fs::read_dir(dir).expect(&format!("No such directory {}", dir.to_string_lossy()))
        {
            let entry = entry.expect("Invalid path found");
            let path = entry.path();
            if path.is_dir() {
                scenarios.scenarios.push(Scenario::load(&path));
            }
        }
        scenarios
    }
}
