pub mod results;
pub mod summary;


use bincode;
use chrono;
use ndarray_stats::QuantileExt;
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::Write,
    path::Path,
    sync::mpsc::Sender,
};
use toml;
use tracing::{debug, info, trace};

use self::{
    results::{Results, Snapshot},
    summary::Summary,
};
use super::{
    algorithm::{self, calculate_pseudo_inverse},
    config::{algorithm::AlgorithmType, Config},
    data::Data,
    model::Model,
};

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
    pub const fn get_status_str(&self) -> &str {
        match self.status {
            Status::Planning => "Planning",
            Status::Done => "Done",
            Status::Running(_) => "Running",
            Status::Aborted => "Aborted",
            Status::Scheduled => "Scheduled",
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
        match &self.config.simulation {
            Some(simulation) => {
                model.sensors_per_axis = simulation.model.sensors_per_axis;
                model.sensor_array_size_mm = simulation.model.sensor_array_size_mm;
                model.sensor_array_origin_mm = simulation.model.sensor_array_origin_mm;
                model.voxel_size_mm = simulation.model.voxel_size_mm;
                model.heart_size_mm = simulation.model.heart_size_mm;
                model.heart_origin_mm = simulation.model.heart_origin_mm;
            }
            None => todo!(),
        };
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
    pub fn set_running(&mut self, epoch: usize) {
        debug!("Setting scenario status to running with epoch {}", epoch);
        self.status = Status::Running(epoch);
    }

    /// Sets the scenario status to Done.
    #[tracing::instrument(level = "debug")]
    pub fn set_done(&mut self) {
        debug!("Setting scenario status to done");
        self.status = Status::Done;
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
        let f = File::create(path.join("data.bin"))?;
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
        let f = File::create(path.join("results.bin"))?;
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
            self.data = Some(bincode::deserialize_from(File::open(file_path).unwrap()).unwrap());
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
            self.results = Some(bincode::deserialize_from(File::open(file_path).unwrap()).unwrap());
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
    let Some(simulation) = &scenario.config.simulation else {
        panic!("Non-simulation case not yet implemented.")
    };

    let data = Data::from_simulation_config(simulation).expect("Model parametrs to be valid.");
    let mut model = Model::from_model_config(
        &scenario.config.algorithm.model,
        simulation.sample_rate_hz,
        simulation.duration_s,
    )
    .unwrap();

    let mut results = Results::new(
        scenario.config.algorithm.epochs,
        model
            .functional_description
            .control_function_values
            .values
            .shape()[0],
        model.spatial_description.sensors.count(),
        model.spatial_description.voxels.count_states(),
    );

    let mut summary = Summary::default();

    match scenario.config.algorithm.algorithm_type {
        AlgorithmType::ModelBased => {
            run_model_based(
                &mut scenario,
                &mut model,
                &mut results,
                &data,
                &mut summary,
                epoch_tx,
                summary_tx,
            );
        }
        AlgorithmType::PseudoInverse => {
            run_pseudo_inverse(&scenario, &model, &mut results, &data, &mut summary);
        }
        AlgorithmType::Loreta => panic!("Algorithm type not implemented"),
    }

    results.metrics.calculate_final(
        &results.estimations,
        data.get_voxel_types(),
        &model.spatial_description.voxels.numbers,
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

    results.model = Some(model);
    scenario.results = Some(results);
    scenario.data = Some(data);
    scenario.summary = Some(summary.clone());
    scenario.status = Status::Done;
    scenario.save().expect("Could not save scenario");
    epoch_tx.send(scenario.config.algorithm.epochs - 1).unwrap();
    summary_tx.send(summary).unwrap();
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
    summary.loss = results.metrics.loss_epoch.values[0];
    summary.loss_mse = results.metrics.loss_mse_epoch.values[0];
    summary.loss_maximum_regularization =
        results.metrics.loss_maximum_regularization_epoch.values[0];
    summary.delta_states_mean = results.metrics.delta_states_mean_epoch.values[0];
    summary.delta_states_max = results.metrics.delta_states_max_epoch.values[0];
    summary.delta_measurements_mean = results.metrics.delta_measurements_mean_epoch.values[0];
    summary.delta_measurements_max = results.metrics.delta_measurements_max_epoch.values[0];
}

/// Runs the model-based algorithm on the given scenario, model, and data.
/// Calculates model parameters over epochs and calculates summary metrics.
/// Reduces learning rate at intervals. Saves snapshots at intervals.
/// Sends epoch and summary updates over channels.
/// Exits early if loss becomes non-finite.
#[tracing::instrument(level = "info", skip_all)]
fn run_model_based(
    scenario: &mut Scenario,
    model: &mut Model,
    results: &mut Results,
    data: &Data,
    summary: &mut Summary,
    epoch_tx: &Sender<usize>,
    summary_tx: &Sender<Summary>,
) {
    info!("Running model-based algorithm");
    let original_learning_rate = scenario.config.algorithm.learning_rate;
    for epoch_index in 0..scenario.config.algorithm.epochs {
        if scenario.config.algorithm.learning_rate_reduction_interval != 0
            && (epoch_index % scenario.config.algorithm.learning_rate_reduction_interval == 0)
            && epoch_index != 0
        {
            scenario.config.algorithm.learning_rate *=
                scenario.config.algorithm.learning_rate_reduction_factor;
        }
        algorithm::run_epoch(
            &mut model.functional_description,
            results,
            data,
            &scenario.config.algorithm,
            epoch_index,
        );
        scenario.status = Status::Running(epoch_index);

        summary.loss = results.metrics.loss_epoch.values[epoch_index];
        summary.loss_mse = results.metrics.loss_mse_epoch.values[epoch_index];
        summary.loss_maximum_regularization =
            results.metrics.loss_maximum_regularization_epoch.values[epoch_index];
        summary.delta_states_mean = results.metrics.delta_states_mean_epoch.values[epoch_index];
        summary.delta_states_max = results.metrics.delta_states_max_epoch.values[epoch_index];
        summary.delta_measurements_mean =
            results.metrics.delta_measurements_mean_epoch.values[epoch_index];
        summary.delta_measurements_max =
            results.metrics.delta_measurements_max_epoch.values[epoch_index];
        summary.delta_gains_mean = results.metrics.delta_gains_mean_epoch.values[epoch_index];
        summary.delta_gains_max = results.metrics.delta_gains_max_epoch.values[epoch_index];
        summary.delta_delays_mean = results.metrics.delta_delays_mean_epoch.values[epoch_index];
        summary.delta_delays_max = results.metrics.delta_delays_max_epoch.values[epoch_index];

        if scenario.config.algorithm.snapshots_interval != 0
            && epoch_index % scenario.config.algorithm.snapshots_interval == 0
        {
            results.snapshots.push(Snapshot::new(
                &results.estimations,
                &model.functional_description,
            ));
        }

        epoch_tx.send(epoch_index).unwrap();
        summary_tx.send(summary.clone()).unwrap();
        // Check if algorithm diverged. If so return early
        if !summary.loss.is_normal() {
            break;
        }
    }
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
    Running(usize),
    Aborted,
    Scheduled,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn building_saves_scenario() {
        let path = Path::new("./results/test");
        if path.is_dir() {
            fs::remove_dir_all(path).unwrap();
        }
        let _scenario = Scenario::build(Some("test".to_string()));
        assert!(path.is_dir());
        assert!(path.join("scenario.toml").is_file());
        fs::remove_dir_all(path).unwrap();
    }

    #[test]
    fn loading_scenarios_works() {
        let path = Path::new("./results/test2");
        if path.is_dir() {
            fs::remove_dir_all(path).unwrap();
        }
        let scenario = Scenario::build(Some("test2".to_string()));

        let loaded = Scenario::load(path);

        assert_eq!(scenario, loaded);

        fs::remove_dir_all(path).unwrap();
    }
}
