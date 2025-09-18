#![warn(clippy::pedantic, clippy::nursery)]
#![allow(
    clippy::too_many_arguments,
    clippy::module_name_repetitions,
    clippy::missing_panics_doc,
    clippy::missing_errors_doc,
    clippy::too_many_lines,
    clippy::cognitive_complexity,
    clippy::needless_pass_by_value,
    clippy::needless_pass_by_ref_mut,
    dead_code,
    private_interfaces
)]
pub mod core;
pub mod scheduler;
pub mod tests;
pub mod ui;
pub mod vis;

use std::{
    fs::{self, create_dir_all},
    path::Path,
    sync::{mpsc::Receiver, Mutex},
    thread::JoinHandle,
};

use anyhow::{Context, Result};
use bevy::prelude::*;
use tracing::{info, warn};

use crate::core::scenario::{summary::Summary, Scenario};

#[derive(Resource, Debug, Default)]
pub struct SelectedSenario {
    pub index: Option<usize>,
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

impl ScenarioList {
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Loads existing scenario results from the `./results` directory into a
    /// [`ScenarioList`], sorting them by scenario ID. Creates the `./results`
    /// directory if it does not exist.
    ///
    /// # Errors
    ///
    /// Returns an error if the results directory cannot be created or read.
    #[tracing::instrument(level = "info")]
    pub fn load() -> Result<Self> {
        info!("Loading scenarios from ./results");
        let mut scenario_list = Self {
            entries: Vec::<ScenarioBundle>::new(),
        };
        let dir = Path::new("./results");
        create_dir_all(dir).context("Failed to create ./results directory")?;

        let dir_entries = fs::read_dir(dir)
            .context("Failed to read ./results directory")?;

        for entry in dir_entries {
            let entry = entry.context("Failed to read directory entry")?;
            let path = entry.path();
            if path.is_dir() {
                match Scenario::load(&path) {
                    Ok(scenario) => {
                        scenario_list.entries.push(ScenarioBundle {
                            scenario,
                            join_handle: None,
                            epoch_rx: None,
                            summary_rx: None,
                        });
                    }
                    Err(e) => {
                        warn!("Failed to load scenario from {}: {}", path.display(), e);
                    }
                }
            }
        }
        if !scenario_list.entries.is_empty() {
            scenario_list
                .entries
                .sort_by_key(|entry| entry.scenario.get_id().clone());
        }
        Ok(scenario_list)
    }
}

impl Default for ScenarioList {
    /// Loads existing scenario results from the `./results` directory into a
    /// [`ScenarioList`], sorting them by scenario ID. Creates the `./results`
    /// directory if it does not exist.
    ///
    /// This provides the default initialized state for the scenario list resource,
    /// populated from any existing results. If loading fails, returns an empty list.
    #[tracing::instrument(level = "info")]
    fn default() -> Self {
        match Self::load() {
            Ok(scenario_list) => scenario_list,
            Err(e) => {
                warn!("Failed to load scenarios from ./results directory: {}", e);
                Self::empty()
            }
        }
    }
}
