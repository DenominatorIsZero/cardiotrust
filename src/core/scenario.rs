pub mod events;
mod persistence;
pub mod plotting;
pub mod results;
pub mod run;
pub mod status;
pub mod summary;
#[cfg(test)]
mod tests;

use std::{
    fs::{self, File},
    io::Write,
    path::Path,
};

use anyhow::{Context, Result};
use chrono::{self, DateTime, Utc};
pub use plotting::calculate_plotting_arrays;
pub use run::run;
use serde::{Deserialize, Serialize};
pub use status::Status;
use toml;
use tracing::{debug, info, trace, warn};

use self::{results::Results, summary::Summary};
use super::config::{algorithm::AlgorithmType, Config};
use crate::core::data::Data;

/// Struct representing a scenario configuration and results.
#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct Scenario {
    id: String,
    status: Status,
    pub config: Config,
    #[serde(skip_serializing, skip_deserializing)]
    pub data: Option<Data>,
    #[serde(skip_serializing, skip_deserializing)]
    pub results: Option<Results>,
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
            data: None,
            results: None,
            summary: None,
            comment: "EMPTY".into(),
            started: None,
            last_update: None,
            finished: None,
            duration_s: None,
        }
    }

    /// Creates a new Scenario with a generated ID and default values.
    ///
    /// The ID is generated from the current date and time. The status is set to
    /// Planning, the config to default, data and results to None, summary to
    /// None, and comment to empty string.
    ///
    /// # Errors
    ///
    /// Returns an error if the new scenario could not be saved to the filesystem.
    #[tracing::instrument(level = "debug")]
    pub fn build(id: Option<String>) -> Result<Self> {
        debug!("Building new scenario");
        let scenario = Self {
            id: id.unwrap_or_else(|| {
                format!("{}", chrono::Utc::now().format("%Y-%m-%d-%H-%M-%S-%f"))
            }),
            status: Status::Planning,
            config: Config::default(),
            data: None,
            results: None,
            summary: None,
            comment: String::new(),
            started: None,
            last_update: None,
            finished: None,
            duration_s: None,
        };
        scenario
            .save()
            .context("Failed to save newly created scenario")?;
        Ok(scenario)
    }

    /// Loads a Scenario from the scenario.toml file in the given path.
    ///
    /// Reads the contents of the scenario.toml file and parses it into a
    /// Scenario struct.
    ///
    /// # Errors
    ///
    /// Returns an error if the scenario.toml file could not be read or parsed.
    #[tracing::instrument(level = "info", skip_all)]
    pub fn load(path: &Path) -> Result<Self> {
        info!("Loading scenario from {}", path.to_string_lossy());
        let scenario_path = path.join("scenario.toml");
        let contents = fs::read_to_string(&scenario_path).with_context(|| {
            format!(
                "Failed to read scenario.toml file: {}",
                scenario_path.display()
            )
        })?;

        let scenario: Self = toml::from_str(&contents).with_context(|| {
            format!(
                "Failed to parse scenario.toml in directory: {}",
                path.display()
            )
        })?;

        Ok(scenario)
    }

    /// Saves the Scenario to a scenario.toml file in the ./results directory.
    ///
    /// Creates the directory path from the scenario ID. Converts the Scenario to a TOML string. Creates the file and writes the TOML string to it.
    /// If the scenario has data, calls `save_data()`. If the scenario has results, calls `save_results()`.
    ///
    /// # Panics
    ///
    /// Panics if scenario could not be parsed into toml string.
    ///
    /// # Errors
    ///
    /// This function will return an error if scenario.toml file could not be created.
    #[tracing::instrument(level = "info", skip(self))]
    pub fn save(&self) -> Result<()> {
        info!("Saving scenario with id {}", self.id);
        let path = Path::new("./results").join(&self.id);
        let toml = toml::to_string(&self).context("Failed to serialize scenario to TOML format")?;
        fs::create_dir_all(&path)?;
        let mut f = File::create(path.join("scenario.toml"))?;
        f.write_all(toml.as_bytes())?;
        if self.data.is_some() {
            self.save_data()?;
        }
        if self.results.is_some() {
            self.save_results()?;
        }
        Ok(())
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

    /// Deletes the results directory for this scenario.
    ///
    /// # Errors
    ///
    /// This function will return an error if the results directory could not be deleted.
    #[tracing::instrument(level = "info", skip_all)]
    pub fn delete(&self) -> Result<(), std::io::Error> {
        info!("Deleting scenario with id {}", self.id);
        let path = Path::new("./results").join(&self.id);
        fs::remove_dir_all(path)?;
        Ok(())
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
            data: None,
            results: None,
            summary: None,
            comment: String::new(),
            started: None,
            last_update: None,
            finished: None,
            duration_s: None,
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
}
