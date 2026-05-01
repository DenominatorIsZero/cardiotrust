pub mod events;
mod persistence;
pub mod plotting;
pub mod results;
pub mod run;
pub mod status;
pub mod summary;
#[cfg(test)]
mod tests;

use anyhow::Result;
use chrono::{self, DateTime, Utc};
pub use persistence::ScenarioStorage;
pub use plotting::calculate_plotting_arrays;
pub use run::run;
use serde::{Deserialize, Serialize};
pub use status::Status;
use tracing::{debug, trace, warn};

#[cfg(test)]
use std::path::{Path, PathBuf};

use self::{results::Results, summary::Summary};
use super::config::{algorithm::AlgorithmType, Config};
use crate::core::data::Data;

#[derive(Debug, Clone)]
pub struct ScenarioPayload {
    pub data: Data,
    pub results: Results,
}

/// Struct representing a scenario configuration and results.
#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct Scenario {
    id: String,
    status: Status,
    pub config: Config,
    pub summary: Option<Summary>,
    #[serde(default)]
    pub comment: String,
    #[serde(default)]
    pub started: Option<DateTime<Utc>>,
    #[serde(default)]
    pub last_update: Option<DateTime<Utc>>,
    #[serde(default)]
    pub finished: Option<DateTime<Utc>>,
    #[serde(default)]
    pub duration_s: Option<i64>,
    #[cfg(test)]
    #[serde(skip)]
    pub data: Option<Data>,
    #[cfg(test)]
    #[serde(skip)]
    pub results: Option<Results>,
    #[cfg(test)]
    #[serde(skip)]
    storage_root: Option<PathBuf>,
}

impl Scenario {
    /// Creates an empty Scenario with default values.
    ///
    /// The id is set to "EMPTY", status to Scheduled, config to default,
    /// data and results to None, summary to None, and comment to "EMPTY".
    ///
    /// This can be useful when needing to initialize a Scenario without
    /// any specific values.
    #[must_use]
    #[tracing::instrument(level = "debug")]
    pub fn empty() -> Self {
        debug!("Creating empty scenario");
        Self {
            id: "EMPTY".into(),
            status: Status::Scheduled,
            config: Config::default(),
            summary: None,
            comment: "EMPTY".into(),
            started: None,
            last_update: None,
            finished: None,
            duration_s: None,
            #[cfg(test)]
            data: None,
            #[cfg(test)]
            results: None,
            #[cfg(test)]
            storage_root: None,
        }
    }

    /// Creates a new Scenario with a generated ID and default values.
    ///
    /// The ID is generated from the current date and time. The status is set to
    /// Planning, the config to default, data and results to None, summary to
    /// None, and comment to empty string.
    ///
    #[tracing::instrument(level = "debug")]
    pub fn build(id: Option<String>) -> Self {
        debug!("Building new scenario");
        Self {
            id: id.unwrap_or_else(|| {
                format!("{}", chrono::Utc::now().format("%Y-%m-%d-%H-%M-%S-%f"))
            }),
            status: Status::Planning,
            config: Config::default(),
            summary: None,
            comment: String::new(),
            started: None,
            last_update: None,
            finished: None,
            duration_s: None,
            #[cfg(test)]
            data: None,
            #[cfg(test)]
            results: None,
            #[cfg(test)]
            storage_root: None,
        }
    }

    /// Returns a reference to the scenario's unique ID.
    #[must_use]
    pub const fn get_id(&self) -> &String {
        &self.id
    }

    /// Returns a string representation of the scenario's status.
    /// Matches the Status enum variant names.
    #[must_use]
    #[tracing::instrument(level = "trace", skip_all)]
    pub fn get_status_str(&self) -> String {
        match self.status {
            Status::Planning => "Planning".to_string(),
            Status::Simulating => "Simulating".to_string(),
            Status::Done => {
                let total_seconds = self.duration_s.unwrap_or(0);
                let seconds = total_seconds % 60;
                let minutes = (total_seconds / 60) % 60;
                let hours = (total_seconds / 3600) % 24;
                let days = total_seconds / 86400;
                if days > 0 {
                    format!("Done ({days}d, {hours}h)")
                } else if hours > 0 {
                    format!("Done ({hours}h, {minutes}m)")
                } else if minutes > 0 {
                    format!("Done ({minutes}m, {seconds}s)")
                } else {
                    format!("Done ({seconds}s)")
                }
            }
            Status::Running(_) => "Running".to_string(),
            Status::Aborted => "Aborted".to_string(),
            Status::Scheduled => "Scheduled".to_string(),
        }
    }

    /// Checks if the scenario is in the planning phase before scheduling it.
    /// If in planning phase, sets status to scheduled and unifies configs.
    ///
    /// # Errors
    ///
    /// This function will return an error if scenario is not in plannig
    /// phase.
    #[tracing::instrument(level = "debug")]
    pub fn schedule(&mut self) -> anyhow::Result<()> {
        debug!("Scheduling scenario");
        match self.status {
            Status::Planning => {
                self.status = Status::Scheduled;
                self.unify_configs();
                Ok(())
            }
            _ => Err(anyhow::anyhow!(
                "Can only schedule scenarios that are in the planning\
             phase but scenario was in phase {:?}",
                self.get_status_str()
            )),
        }
    }

    /// Unifies the model configuration between the algorithm config and simulation config, if a simulation config exists.
    /// This ensures the algorithm and simulation are using the same model parameters.
    /// Also sets algorithm epochs to 1 if it is `PseudoInverse`.
    #[tracing::instrument(level = "debug")]
    fn unify_configs(&mut self) {
        debug!("Unifying algorithm and simulation configs");
        let model = &mut self.config.algorithm.model;
        let simulation = &self.config.simulation;
        model.common.sensor_array_geometry = simulation.model.common.sensor_array_geometry.clone();
        model.common.three_d_sensors = simulation.model.common.three_d_sensors;
        model.common.number_of_sensors = simulation.model.common.number_of_sensors;
        model.common.sensor_array_radius_mm = simulation.model.common.sensor_array_radius_mm;
        model.common.sensors_per_axis = simulation.model.common.sensors_per_axis;
        model.common.sensor_array_size_mm = simulation.model.common.sensor_array_size_mm;
        model.common.sensor_array_origin_mm = simulation.model.common.sensor_array_origin_mm;
        model.common.voxel_size_mm = simulation.model.common.voxel_size_mm;
        model.common.heart_offset_mm = simulation.model.common.heart_offset_mm;
        model.common.sensor_array_motion = simulation.model.common.sensor_array_motion.clone();
        model.common.sensor_array_motion_range_mm =
            simulation.model.common.sensor_array_motion_range_mm;
        model.common.sensor_array_motion_steps = simulation.model.common.sensor_array_motion_steps;
        if let Some(handcrafted) = simulation.model.handcrafted.as_ref() {
            if let Some(model_handcrafted) = model.handcrafted.as_mut() {
                model_handcrafted.heart_size_mm = handcrafted.heart_size_mm;
            }
        }
        if self.config.algorithm.algorithm_type == AlgorithmType::PseudoInverse {
            self.config.algorithm.epochs = 1;
        }
    }

    /// Set't the status of the scenario to "Planning".
    ///
    /// This removes the scenario from the queue and allows
    /// for the parameters to be changed again
    ///
    /// # Errors
    ///
    /// This function will return an error if scenario is not in scheduled
    /// phase.
    #[tracing::instrument(level = "debug")]
    pub fn unschedule(&mut self) -> Result<(), String> {
        debug!("Unscheduling scenario");
        match self.status {
            Status::Scheduled => {
                self.status = Status::Planning;
                Ok(())
            }
            _ => Err(format!(
                "Can only unschedule scenarios that are in the\
            scheduled phase but scenario was in phase {:?}",
                self.get_status_str()
            )),
        }
    }

    /// Sets the scenario status to Running with the given epoch number.
    #[tracing::instrument(level = "debug")]
    pub fn set_simulating(&mut self) {
        debug!("Setting scenario status to simulating");
        self.status = Status::Simulating;
    }

    /// Sets the scenario status to Running with the given epoch number.
    #[tracing::instrument(level = "debug")]
    pub fn set_running(&mut self, epoch: usize) {
        debug!("Setting scenario status to running with epoch {}", epoch);
        self.status = Status::Running(epoch);
        if self.started.is_none() {
            self.started = Some(Utc::now());
        }
        self.last_update = Some(Utc::now());
    }

    /// Sets the scenario status to Done.
    #[tracing::instrument(level = "debug")]
    pub fn set_done(&mut self) {
        debug!("Setting scenario status to done");
        self.status = Status::Done;
        let finished_time = Utc::now();
        self.finished = Some(finished_time);
        if let Some(started_time) = self.started {
            self.duration_s = Some((finished_time - started_time).num_seconds());
        } else {
            warn!("Scenario finished without a recorded start time - duration calculation skipped");
        }
    }

    /// Returns an immutable reference to the scenario status.
    #[must_use]
    pub const fn get_status(&self) -> &Status {
        &self.status
    }

    /// Returns the progress of the scenario as a percentage. The progress will be
    /// 0.0 if the scenario status is not Running. Otherwise it will return the
    /// current epoch divided by the total number of epochs.
    #[must_use]
    #[tracing::instrument(level = "trace")]
    pub fn get_progress(&self) -> f32 {
        trace!("Getting progress for scenario with id {}", self.id);
        #[allow(clippy::cast_precision_loss)]
        match self.status {
            Status::Running(epoch) => epoch as f32 / self.config.algorithm.epochs as f32,
            _ => 0.0,
        }
    }

    #[allow(clippy::cast_possible_truncation)]
    #[must_use]
    #[tracing::instrument(level = "trace")]
    pub fn get_etc(&self) -> String {
        trace!("Getting progress for scenario with id {}", self.id);
        #[allow(clippy::cast_precision_loss)]
        match self.status {
            Status::Running(0) => "ETC: ???".to_string(),
            Status::Running(_) => {
                let now = Utc::now();
                let (Some(started), Some(last_update)) = (self.started, self.last_update) else {
                    return "ETC: ???".to_string();
                };
                let duration = last_update - started;
                let ellapsed_second = duration.num_seconds();
                let progress = self.get_progress();
                let remaining = 1.0 - progress;
                let meanwhile = now - last_update;
                let remaining_seconds_total = (ellapsed_second as f32 / progress * remaining)
                    as i64
                    - meanwhile.num_seconds();
                let remaining_seconds = remaining_seconds_total % 60;
                let remaining_minutes = (remaining_seconds_total / 60) % 60;
                let remaining_hours = (remaining_seconds_total / 3600) % 24;
                let remaining_days = remaining_seconds_total / 86400;

                if remaining_days > 0 {
                    format!("ETC: {remaining_days} days, {remaining_hours} hours")
                } else if remaining_hours > 0 {
                    format!("ETC: {remaining_hours} hours, {remaining_minutes} minutes")
                } else if remaining_minutes > 0 {
                    format!("ETC: {remaining_minutes} minutes, {remaining_seconds} seconds")
                } else {
                    format!("ETC: {remaining_seconds} seconds")
                }
            }
            _ => String::new(),
        }
    }

    /// Creates a new Scenario in Planning status without writing to disk.
    /// Only used in tests.
    #[cfg(test)]
    #[must_use]
    #[tracing::instrument(level = "debug")]
    pub fn new_in_memory() -> Self {
        Self {
            id: format!("test-{}", chrono::Utc::now().format("%Y%m%d%H%M%S%f")),
            status: Status::Planning,
            config: Config::default(),
            summary: None,
            comment: String::new(),
            started: None,
            last_update: None,
            finished: None,
            duration_s: None,
            #[cfg(test)]
            data: None,
            #[cfg(test)]
            results: None,
            #[cfg(test)]
            storage_root: None,
        }
    }

    /// Creates a Planning scenario without saving to disk, for use in tests.
    #[cfg(test)]
    #[must_use]
    #[tracing::instrument(level = "debug")]
    pub fn empty_planning() -> Self {
        Self::new_in_memory()
    }

    /// Forces the scenario into `Scheduled` status without validation.
    /// Only used in tests to set up non-Planning states.
    #[cfg(test)]
    #[tracing::instrument(level = "debug")]
    pub fn force_scheduled(&mut self) {
        self.status = Status::Scheduled;
    }

    #[cfg(test)]
    #[tracing::instrument(level = "debug")]
    pub fn save(&mut self) -> Result<()> {
        let storage = ScenarioStorage::new(
            self.storage_root
                .clone()
                .unwrap_or_else(|| PathBuf::from("./results")),
        );
        storage.save_metadata(self)?;
        self.storage_root = Some(storage.project_root().to_path_buf());
        Ok(())
    }

    #[cfg(test)]
    #[tracing::instrument(level = "debug")]
    pub fn load(path: &Path) -> Result<Self> {
        let project_root = path.parent().unwrap_or_else(|| Path::new("./results"));
        let storage = ScenarioStorage::new(project_root.to_path_buf());
        let mut scenario = storage.load_metadata(path)?;
        scenario.storage_root = Some(project_root.to_path_buf());
        scenario.data = None;
        scenario.results = None;
        Ok(scenario)
    }

    #[cfg(test)]
    #[tracing::instrument(level = "debug")]
    pub fn load_data(&mut self) -> Result<()> {
        let payload = self.load_payload_for_tests()?;
        self.data = Some(payload.data);
        self.results = Some(payload.results);
        Ok(())
    }

    #[cfg(test)]
    #[tracing::instrument(level = "debug")]
    pub fn load_results(&mut self) -> Result<()> {
        let payload = self.load_payload_for_tests()?;
        self.data = Some(payload.data);
        self.results = Some(payload.results);
        Ok(())
    }

    #[cfg(test)]
    #[tracing::instrument(level = "debug")]
    fn load_payload_for_tests(&self) -> Result<ScenarioPayload> {
        let storage = ScenarioStorage::new(
            self.storage_root
                .clone()
                .unwrap_or_else(|| PathBuf::from("./results")),
        );
        storage.load_payload(self.get_id())
    }
}

#[cfg(test)]
#[tracing::instrument(level = "info", skip_all, fields(id = %scenario.id))]
pub fn run_test_scenario(
    scenario: Scenario,
    epoch_tx: &std::sync::mpsc::Sender<usize>,
    summary_tx: &std::sync::mpsc::Sender<Summary>,
) -> Result<()> {
    let storage_root = scenario
        .storage_root
        .clone()
        .unwrap_or_else(|| PathBuf::from("./results"));
    run::run(scenario, ScenarioStorage::new(storage_root), epoch_tx, summary_tx)
}
