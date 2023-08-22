pub mod results;
pub mod summary;

use std::path::Path;
use std::sync::mpsc::Sender;

use std::{fs, fs::File, io::Write};

use chrono;

use ciborium::{from_reader, into_writer};
use serde::{Deserialize, Serialize};
use toml;

use self::results::{Results, Snapshot};
use self::summary::Summary;

use super::algorithm;
use super::model::Model;
use super::{config::Config, data::Data};

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct Scenario {
    id: String,
    status: Status,
    config: Config,
    #[serde(skip_serializing, skip_deserializing)]
    pub data: Option<Data>,
    #[serde(skip_serializing, skip_deserializing)]
    pub results: Option<Results>,
    pub summary: Option<Summary>,
    #[serde(default)]
    pub comment: String,
}

impl Scenario {
    #[must_use]
    pub fn build(id: Option<String>) -> Self {
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

    /// .
    ///
    /// # Panics
    ///
    /// Panics if scenario.toml could not be read in scenario directory.
    /// Panics if scenario.toml data could not be parsed into scenario struct.
    #[must_use]
    pub fn load(path: &Path) -> Self {
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

    /// # Panics
    ///
    /// Panics if scenario could not be parsed into toml string.
    ///
    /// # Errors
    ///
    /// This function will return an error if scenario.toml file could not be created.
    ///
    /// This function will return an error if scenario.toml file could not be written to.
    pub fn save(&self) -> Result<(), std::io::Error> {
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

    #[must_use]
    pub const fn get_id(&self) -> &String {
        &self.id
    }

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

    #[must_use]
    pub fn get_config_mut(&mut self) -> &mut Config {
        &mut self.config
    }

    #[must_use]
    pub const fn get_config(&self) -> &Config {
        &self.config
    }

    ///
    /// # Errors
    ///
    /// This function will return an error if scenario is not in plannig
    /// phase.
    pub fn schedule(&mut self) -> Result<(), String> {
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

    fn unify_configs(&mut self) {
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
    }

    /// Set't the status of the scenario to "Planning".
    ///
    /// This removes the scenario from the queue and allows
    /// for the parameters to be changed again
    ///
    /// TODO: Look into types as states
    ///
    /// # Errors
    ///
    /// This function will return an error if scenario is not in scheduled
    /// phase.
    pub fn unschedule(&mut self) -> Result<(), String> {
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

    pub fn set_running(&mut self, epoch: usize) {
        self.status = Status::Running(epoch);
    }

    pub fn set_done(&mut self) {
        self.status = Status::Done;
    }

    /// # Errors
    ///
    /// This function will return an error if directories could not
    /// be deleted.
    pub fn delete(&self) -> Result<(), std::io::Error> {
        let path = Path::new("./results").join(&self.id);
        fs::remove_dir_all(path)?;
        Ok(())
    }

    #[must_use]
    pub const fn get_status(&self) -> &Status {
        &self.status
    }

    #[must_use]
    pub fn get_progress(&self) -> f32 {
        #[allow(clippy::cast_precision_loss)]
        match self.status {
            Status::Running(epoch) => epoch as f32 / self.config.algorithm.epochs as f32,
            _ => 0.0,
        }
    }

    fn save_data(&self) -> Result<(), std::io::Error> {
        let path = Path::new("./results").join(&self.id);
        fs::create_dir_all(&path)?;
        let f = File::create(path.join("data.bin"))?;
        into_writer(self.data.as_ref().unwrap(), f).unwrap();
        Ok(())
    }

    fn save_results(&self) -> Result<(), std::io::Error> {
        let path = Path::new("./results").join(&self.id);
        fs::create_dir_all(&path)?;
        let f = File::create(path.join("results.bin"))?;
        into_writer(self.results.as_ref().unwrap(), f).unwrap();
        Ok(())
    }

    /// # Panics
    ///
    /// Panics if the data.bin file can not be parsed into the data struct.
    pub fn load_data(&mut self) {
        if self.data.is_some() {
            return;
        }
        let file_path = Path::new("./results").join(&self.id).join("data.bin");
        if file_path.is_file() {
            self.data = Some(from_reader(File::open(file_path).unwrap()).unwrap());
        }
    }

    /// # Panics
    ///
    /// Panics if the results.bin file can not be parsed into the results struct.
    pub fn load_results(&mut self) {
        println!("Loading results");
        if self.results.is_some() {
            return;
        }
        let file_path = Path::new("./results").join(&self.id).join("results.bin");
        if file_path.is_file() {
            println!("Found File");
            self.results = Some(from_reader(File::open(file_path).unwrap()).unwrap());
        }
        println!("Loaded results.");
    }
}

/// .
///
/// # Panics
///
/// Panics if .
pub fn run(mut scenario: Scenario, epoch_tx: &Sender<usize>, summary_tx: &Sender<Summary>) {
    let Some(simulation) = &scenario.config.simulation else { todo!("Non-simulation case not yet implemented.") };
    let data = Data::from_simulation_config(simulation);
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

    for epoch_index in 0..scenario.config.algorithm.epochs {
        algorithm::run_epoch(
            &mut model.functional_description,
            &mut results,
            &data,
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
    }
    results.model = Some(model);
    scenario.results = Some(results);
    scenario.data = Some(data);
    scenario.summary = Some(summary);
    scenario.status = Status::Done;
    scenario.save().expect("Could not save scenario");
}

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
