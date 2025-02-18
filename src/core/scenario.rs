pub mod results;
pub mod summary;
#[cfg(test)]
mod tests;

use std::{
    fs::{self, File},
    io::{BufReader, BufWriter, Write},
    path::Path,
    sync::mpsc::Sender,
};

use bincode;
use chrono::{self, DateTime, Utc};
use ndarray_stats::QuantileExt;
use serde::{Deserialize, Serialize};
use toml;
use tracing::{debug, info, trace};

use self::{
    results::{Results, Snapshots},
    summary::Summary,
};
use super::{
    algorithm::{self, calculate_pseudo_inverse},
    config::{algorithm::AlgorithmType, Config},
    data::Data,
    model::Model,
};
use crate::core::algorithm::{metrics, refinement::derivation::calculate_average_delays};

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
    /// # Panics
    ///
    /// Panics if the new scenario could not be saved.
    #[must_use]
    #[tracing::instrument(level = "debug")]
    pub fn build(id: Option<String>) -> Self {
        debug!("Building new scenario");
        let scenario = Self {
            id: id.map_or_else(
                || format!("{}", chrono::Utc::now().format("%Y-%m-%d-%H-%M-%S-%f")),
                |id| id,
            ),
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
            .expect("Could not save newly created scenario.");
        scenario
    }

    /// Loads a Scenario from the scenario.toml file in the given path.
    ///
    /// Reads the contents of the scenario.toml file and parses it into a
    /// Scenario struct.
    ///
    /// # Panics
    ///
    /// Panics if the scenario.toml file could not be read or parsed.
    #[must_use]
    #[tracing::instrument(level = "info", skip_all)]
    pub fn load(path: &Path) -> Self {
        info!("Loading scenario from {}", path.to_string_lossy());
        let contents = fs::read_to_string(path.join("scenario.toml")).unwrap_or_else(|_| {
            panic!(
                "Could not read scenario.toml file in directory '{}'",
                path.to_string_lossy()
            )
        });

        let scenario: Self = toml::from_str(&contents).unwrap_or_else(|_| {
            panic!(
                "Could not parse data found in scenario.toml in directory '{}'",
                path.to_string_lossy()
            )
        });

        scenario
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
    pub fn save(&self) -> Result<(), std::io::Error> {
        info!("Saving scenario with id {}", self.id);
        let path = Path::new("./results").join(&self.id);
        let toml = toml::to_string(&self).unwrap();
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
    pub fn schedule(&mut self) -> Result<(), String> {
        debug!("Scheduling scenario");
        match self.status {
            Status::Planning => {
                self.status = Status::Scheduled;
                self.unify_configs();
                Ok(())
            }
            _ => Err(format!(
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
            model.handcrafted.as_mut().unwrap().heart_size_mm = handcrafted.heart_size_mm;
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
        self.finished = Some(Utc::now());
        self.duration_s = Some((self.finished.unwrap() - self.started.unwrap()).num_seconds());
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
                let duration = self.last_update.unwrap() - self.started.unwrap();
                let ellapsed_second = duration.num_seconds();
                let progress = self.get_progress();
                let remaining = 1.0 - progress;
                let meanwhile = now - self.last_update.unwrap();
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

    /// Saves the scenario data to a file in the results directory.
    ///
    /// # Errors
    ///
    /// This function will return an error if the results directory could not be created or the data file could not be written.
    #[tracing::instrument(level = "debug")]
    fn save_data(&self) -> Result<(), std::io::Error> {
        debug!("Saving scenario data for scenario with id {}", self.id);
        let path = Path::new("./results").join(&self.id);
        fs::create_dir_all(&path)?;
        let f = BufWriter::new(File::create(path.join("data.bin"))?);
        bincode::serialize_into(f, self.data.as_ref().unwrap()).unwrap();
        Ok(())
    }

    /// Saves the scenario results to a file in the results directory.
    ///
    /// # Errors
    ///
    /// This function will return an error if the results directory could not be created or the results file could not be written.
    #[tracing::instrument(level = "debug")]
    fn save_results(&self) -> Result<(), std::io::Error> {
        debug!("Saving scenario results for scenario with id {}", self.id);
        let path = Path::new("./results").join(&self.id);
        fs::create_dir_all(&path)?;
        let f = BufWriter::new(File::create(path.join("results.bin"))?);
        bincode::serialize_into(f, self.results.as_ref().unwrap()).unwrap();
        Ok(())
    }

    /// Loads the scenario data from the data.bin file in the results directory if it exists.
    ///
    /// # Panics
    ///
    /// Panics if the data.bin file can not be parsed into the data struct.
    #[tracing::instrument(level = "debug")]
    pub fn load_data(&mut self) {
        debug!("Loading scenario data for scenario with id {}", self.id);
        if self.data.is_some() {
            return;
        }
        let file_path = Path::new("./results").join(&self.id).join("data.bin");
        if file_path.is_file() {
            self.data = Some(
                bincode::deserialize_from(BufReader::new(File::open(file_path).unwrap())).unwrap(),
            );
        }
    }

    /// Loads the scenario results from the results.bin file in the results directory if it exists.
    ///
    /// # Panics
    ///
    /// Panics if the results.bin file can not be parsed into the results struct.
    #[tracing::instrument(level = "debug")]
    pub fn load_results(&mut self) {
        debug!("Loading scenario results for scenario with id {}", self.id);
        if self.results.is_some() {
            return;
        }
        let file_path = Path::new("./results").join(&self.id).join("results.bin");
        if file_path.is_file() {
            self.results = Some(
                bincode::deserialize_from(BufReader::new(File::open(file_path).unwrap())).unwrap(),
            );
        }
    }

    /// Saves the scenario data and results as .npy files in the results directory.
    ///
    /// # Panics
    ///
    /// Panics if a file or directory cant be created or written to.
    #[tracing::instrument(level = "debug")]
    pub fn save_npy(&self) {
        debug!("Saving scenario data and results as npy");
        let path = Path::new("./results").join(&self.id).join("npy");
        self.data.as_ref().unwrap().save_npy(&path.join("data"));
        self.results
            .as_ref()
            .unwrap()
            .save_npy(&path.join("results"));
    }
}

/// Runs the simulation for the given scenario, model, and data.
///
/// Updates the results and summary structs with the output. Sends the final epoch
/// count and summary via the provided channels. Saves the results to the scenario.
///
/// # Panics
///
/// Panics if simulation is none, an unimplemented algorithm is selected or
/// the parameters do not yield a valid model.
#[tracing::instrument(level = "info", skip_all, fields(id = %scenario.id))]
pub fn run(mut scenario: Scenario, epoch_tx: &Sender<usize>, summary_tx: &Sender<Summary>) {
    debug!("Running scenario with id {}", scenario.id);

    let simulation = &scenario.config.simulation;

    let data = Data::from_simulation_config(simulation).expect("Model parametrs to be valid.");
    let mut model = Model::from_model_config(
        &scenario.config.algorithm.model,
        simulation.sample_rate_hz,
        simulation.duration_s,
    )
    .unwrap();

    // synchronice model and simulation sensor parameters
    model.synchronize_parameters(&data);

    let _ = epoch_tx.send(0);

    let number_of_snapshots = if scenario.config.algorithm.snapshots_interval == 0 {
        0
    } else {
        scenario.config.algorithm.epochs / scenario.config.algorithm.snapshots_interval + 1
    };

    let mut results = Results::new(
        scenario.config.algorithm.epochs,
        model.functional_description.control_function_values.shape()[0],
        model.spatial_description.sensors.count(),
        model.spatial_description.voxels.count_states(),
        model.spatial_description.sensors.count_beats(),
        number_of_snapshots,
        scenario.config.algorithm.batch_size,
        scenario.config.algorithm.optimizer,
    );

    let mut summary = Summary::default();

    match scenario.config.algorithm.algorithm_type {
        AlgorithmType::ModelBased => {
            results.model = Some(model);
            run_model_based(
                &mut scenario,
                &mut results,
                &data,
                &mut summary,
                epoch_tx,
                summary_tx,
            );
        }
        AlgorithmType::PseudoInverse => {
            run_pseudo_inverse(&scenario, &model, &mut results, &data, &mut summary);
            results.model = Some(model);
        }
    }

    calculate_plotting_arrays(&mut results, &data);

    metrics::calculate_final(
        &mut results.metrics,
        &results.estimations,
        &data.simulation.model.spatial_description.voxels.types,
        &results
            .model
            .as_ref()
            .unwrap()
            .spatial_description
            .voxels
            .numbers,
    );

    let optimal_threshold = results
        .metrics
        .dice_score_over_threshold
        .argmax_skipnan()
        .unwrap_or_default();

    #[allow(clippy::cast_precision_loss)]
    {
        summary.threshold = optimal_threshold as f32 / 100.0;
    }
    summary.dice = results.metrics.dice_score_over_threshold[optimal_threshold];
    summary.iou = results.metrics.iou_over_threshold[optimal_threshold];
    summary.recall = results.metrics.recall_over_threshold[optimal_threshold];
    summary.precision = results.metrics.precision_over_threshold[optimal_threshold];

    scenario.results = Some(results);
    scenario.data = Some(data);
    scenario.summary = Some(summary.clone());
    scenario.status = Status::Done;
    scenario.save().expect("Could not save scenario");
    let _ = epoch_tx.send(scenario.config.algorithm.epochs - 1);
    let _ = summary_tx.send(summary);
}

#[tracing::instrument(level = "trace", skip_all)]
pub(crate) fn calculate_plotting_arrays(results: &mut Results, data: &Data) {
    results
        .estimations
        .system_states_spherical
        .calculate(&results.estimations.system_states);
    results
        .estimations
        .system_states_spherical_max
        .calculate(&results.estimations.system_states_spherical);

    results
        .estimations
        .system_states_spherical_max_delta
        .theta
        .assign(
            &(&data.simulation.system_states_spherical_max.theta
                - &results.estimations.system_states_spherical_max.theta),
        );

    results
        .estimations
        .system_states_spherical_max_delta
        .phi
        .assign(
            &(&data.simulation.system_states_spherical_max.phi
                - &results.estimations.system_states_spherical_max.phi),
        );

    results
        .estimations
        .system_states_spherical_max_delta
        .magnitude
        .assign(
            &(&data.simulation.system_states_spherical_max.magnitude
                - &results.estimations.system_states_spherical_max.magnitude),
        );

    results.estimations.activation_times.calculate(
        &results.estimations.system_states_spherical,
        data.simulation.sample_rate_hz,
    );

    results
        .estimations
        .activation_times_delta
        .assign(&(&*data.simulation.activation_times - &*results.estimations.activation_times));

    results
        .model
        .as_mut()
        .unwrap()
        .update_activation_time(&results.estimations.activation_times);
}

/// Runs the pseudo inverse algorithm on the given scenario, model, and data.
/// Calculates the pseudo inverse, runs estimations, and calculates summary metrics.
#[tracing::instrument(level = "info", skip_all)]
fn run_pseudo_inverse(
    scenario: &Scenario,
    model: &Model,
    results: &mut Results,
    data: &Data,
    summary: &mut Summary,
) {
    info!("Running pseudo inverse algorithm");
    calculate_pseudo_inverse(
        &model.functional_description,
        results,
        data,
        &scenario.config.algorithm,
    );
    summary.loss = results.metrics.loss_batch[0];
    summary.loss_mse = results.metrics.loss_mse_batch[0];
    summary.loss_maximum_regularization = results.metrics.loss_maximum_regularization_batch[0];
}

/// Runs the model-based algorithm on the given scenario, model, and data.
/// Calculates model parameters over epochs and calculates summary metrics.
/// Reduces learning rate at intervals. Saves snapshots at intervals.
/// Sends epoch and summary updates over channels.
/// Exits early if loss becomes non-finite.
#[tracing::instrument(level = "info", skip_all)]
fn run_model_based(
    scenario: &mut Scenario,
    results: &mut Results,
    data: &Data,
    summary: &mut Summary,
    epoch_tx: &Sender<usize>,
    summary_tx: &Sender<Summary>,
) {
    info!("Running model-based algorithm");
    let original_learning_rate = scenario.config.algorithm.learning_rate;
    let mut batch_index = 0;
    for epoch_index in 0..scenario.config.algorithm.epochs {
        if epoch_index == 0 {
            scenario.config.algorithm.learning_rate = 0.0;
        } else if epoch_index == 1 {
            scenario.config.algorithm.learning_rate = original_learning_rate;
        }
        if scenario.config.algorithm.learning_rate_reduction_interval != 0
            && (epoch_index % scenario.config.algorithm.learning_rate_reduction_interval == 0)
        {
            scenario.config.algorithm.learning_rate *=
                scenario.config.algorithm.learning_rate_reduction_factor;
        }
        algorithm::run_epoch(results, &mut batch_index, data, &scenario.config.algorithm);
        scenario.status = Status::Running(epoch_index);

        summary.loss = results.metrics.loss_batch[batch_index - 1];
        summary.loss_mse = results.metrics.loss_mse_batch[batch_index - 1];
        summary.loss_maximum_regularization =
            results.metrics.loss_maximum_regularization_batch[batch_index - 1];

        if scenario.config.algorithm.snapshots_interval != 0
            && epoch_index % scenario.config.algorithm.snapshots_interval == 0
        {
            results.snapshots.as_mut().unwrap().push(
                &results.estimations,
                &results
                    .model
                    .as_ref()
                    .unwrap()
                    .functional_description
                    .ap_params,
            );
        }

        let _ = epoch_tx.send(epoch_index);
        let _ = summary_tx.send(summary.clone());
        // Check if algorithm diverged. If so return early
        if !summary.loss.is_normal() {
            break;
        }
    }
    calculate_average_delays(
        &mut results.estimations.average_delays,
        &results
            .model
            .as_ref()
            .unwrap()
            .functional_description
            .ap_params,
    );
    scenario.config.algorithm.learning_rate = original_learning_rate;
}
/// Enumeration of possible scenario execution statuses.
///
/// * `Planning`: Scenario is being planned.
/// * `Done`: Scenario execution finished.
/// * `Running`: Scenario is running the specified epoch.
/// * `Aborted`: Scenario execution was aborted.
/// * `Scheduled`: Scenario execution is scheduled but not yet running.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub enum Status {
    Planning,
    Done,
    Simulating,
    Running(usize),
    Aborted,
    Scheduled,
}
