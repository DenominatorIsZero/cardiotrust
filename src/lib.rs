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
    path::{Path, PathBuf},
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

    /// Loads existing scenario results from the given `path` directory into a
    /// [`ScenarioList`], sorting them by scenario ID. Creates the directory if
    /// it does not exist.
    ///
    /// # Errors
    ///
    /// Returns an error if the directory cannot be created or read.
    #[tracing::instrument(level = "info")]
    pub fn load_from(path: &Path) -> Result<Self> {
        info!("Loading scenarios from {}", path.display());
        let mut scenario_list = Self {
            entries: Vec::<ScenarioBundle>::new(),
        };
        create_dir_all(path)
            .with_context(|| format!("Failed to create directory {}", path.display()))?;

        let dir_entries = fs::read_dir(path)
            .with_context(|| format!("Failed to read directory {}", path.display()))?;

        for entry in dir_entries {
            let entry = entry.context("Failed to read directory entry")?;
            let entry_path = entry.path();
            if entry_path.is_dir() {
                match Scenario::load(&entry_path) {
                    Ok(scenario) => {
                        scenario_list.entries.push(ScenarioBundle {
                            scenario,
                            join_handle: None,
                            epoch_rx: None,
                            summary_rx: None,
                        });
                    }
                    Err(e) => {
                        warn!(
                            "Failed to load scenario from {}: {}",
                            entry_path.display(),
                            e
                        );
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

    /// Loads existing scenario results from the `./results` directory into a
    /// [`ScenarioList`], sorting them by scenario ID. Creates the `./results`
    /// directory if it does not exist.
    ///
    /// # Errors
    ///
    /// Returns an error if the results directory cannot be created or read.
    #[tracing::instrument(level = "info")]
    pub fn load() -> Result<Self> {
        Self::load_from(Path::new("./results"))
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

/// Holds the currently-open project folder and a history of recently-opened
/// project folders (capped at 8 entries).
///
/// Used by the Home view to let the user select a project, and by the project
/// loading system to reload [`ScenarioList`] when the path changes.
#[derive(Resource, Debug, Default)]
pub struct ProjectState {
    /// The currently loaded project folder, or `None` if no project is open.
    pub current_path: Option<PathBuf>,
    /// Recently opened project folders, most-recent first, capped at 8.
    pub recent: Vec<PathBuf>,
}

impl ProjectState {
    /// Prepend `path` to the recent list, remove any duplicate, and truncate
    /// to a maximum of 8 entries.
    #[tracing::instrument(skip(self))]
    pub fn push_recent(&mut self, path: PathBuf) {
        self.recent.retain(|p| p != &path);
        self.recent.insert(0, path);
        self.recent.truncate(8);
    }

    /// Writes the recent-project list as a TOML file to
    /// `~/.config/cardiotrust/recent_projects.toml`.
    ///
    /// This is a no-op on WASM targets.
    ///
    /// # Errors
    ///
    /// Returns an error if the config directory cannot be created or the file
    /// cannot be written.
    #[cfg(not(target_arch = "wasm32"))]
    #[tracing::instrument(skip(self))]
    pub fn save_recent(&self) -> Result<()> {
        let config_dir = dirs_config_path()?;
        create_dir_all(&config_dir).context("Failed to create cardiotrust config directory")?;
        let file_path = config_dir.join("recent_projects.toml");

        let recent: Vec<String> = self
            .recent
            .iter()
            .filter_map(|p| p.to_str().map(str::to_owned))
            .collect();
        let contents = toml::to_string(&RecentProjectsToml { recent })
            .context("Failed to serialize recent projects")?;
        fs::write(&file_path, contents)
            .with_context(|| format!("Failed to write {}", file_path.display()))?;
        Ok(())
    }

    #[cfg(target_arch = "wasm32")]
    #[tracing::instrument(skip(self))]
    pub fn save_recent(&self) -> Result<()> {
        Ok(())
    }

    /// Reads the recent-project list from
    /// `~/.config/cardiotrust/recent_projects.toml`.
    ///
    /// Returns an empty `Vec` on any error (missing file, parse failure, …).
    ///
    /// On WASM targets this always returns an empty `Vec`.
    #[cfg(not(target_arch = "wasm32"))]
    #[tracing::instrument]
    pub fn load_recent() -> Vec<PathBuf> {
        (|| -> Result<Vec<PathBuf>> {
            let config_dir = dirs_config_path()?;
            let file_path = config_dir.join("recent_projects.toml");
            let contents = fs::read_to_string(&file_path)
                .with_context(|| format!("Failed to read {}", file_path.display()))?;
            let data: RecentProjectsToml =
                toml::from_str(&contents).context("Failed to parse recent_projects.toml")?;
            Ok(data.recent.into_iter().map(PathBuf::from).collect())
        })()
        .unwrap_or_default()
    }

    #[cfg(target_arch = "wasm32")]
    #[tracing::instrument]
    pub fn load_recent() -> Vec<PathBuf> {
        Vec::new()
    }
}

/// Returns the path to the cardiotrust config directory
/// (`~/.config/cardiotrust` on most platforms).
///
/// # Errors
///
/// Returns an error if the home config directory cannot be determined.
#[cfg(not(target_arch = "wasm32"))]
#[tracing::instrument]
fn dirs_config_path() -> Result<PathBuf> {
    let base = dirs::config_dir().context("Could not determine config directory")?;
    Ok(base.join("cardiotrust"))
}

/// TOML serialization helper for the recent-projects config file.
#[cfg(not(target_arch = "wasm32"))]
#[derive(serde::Serialize, serde::Deserialize)]
struct RecentProjectsToml {
    recent: Vec<String>,
}
