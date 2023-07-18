pub mod results;
pub mod summary;

use std::path::Path;
use std::sync::mpsc::Sender;

use std::{fs, fs::File, io::Write};

use chrono;

use serde::{Deserialize, Serialize};
use toml;

use self::results::Results;
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
}

impl Scenario {
    pub fn build(id: Option<String>) -> Scenario {
        println!("Creating new scenario!");
        let scenario = Scenario {
            id: match id {
                Some(id) => id,
                None => format!("{}", chrono::Utc::now().format("%Y-%m-%d-%H-%M-%S-%f")),
            },
            status: Status::Planning,
            config: Config::default(),
            data: None,
            results: None,
            summary: None,
        };
        scenario
            .save()
            .expect("Could not save newly created scenario.");
        scenario
    }

    pub fn load(path: &Path) -> Scenario {
        let contents = fs::read_to_string(path.join("scenario.toml")).expect(&format!(
            "Could not read scenario.toml file in directory '{}'",
            path.to_string_lossy()
        ));

        let scenario: Scenario = toml::from_str(&contents).expect(&format!(
            "Could not parse data found in scenario.toml in directory '{}'",
            path.to_string_lossy()
        ));

        scenario
    }

    pub fn save(&self) -> Result<(), std::io::Error> {
        let path = Path::new("./results").join(&self.id);
        let toml = toml::to_string(&self).unwrap();
        fs::create_dir_all(&path)?;
        let mut f = File::create(&path.join("scenario.toml"))?;
        f.write_all(toml.as_bytes())?;
        self.save_data();
        self.save_results();
        Ok(())
    }

    pub fn get_id(&self) -> &String {
        &self.id
    }

    pub fn get_status_str(&self) -> &str {
        match self.status {
            Status::Planning => "Planning",
            Status::Done => "Done",
            Status::Running(_) => "Running",
            Status::Aborted => "Aborted",
            Status::Scheduled => "Scheduled",
        }
    }

    pub fn get_config_mut(&mut self) -> &mut Config {
        &mut self.config
    }

    pub fn schedule(&mut self) -> Result<(), String> {
        match self.status {
            Status::Planning => {
                self.status = Status::Scheduled;
                self.unify_configs();
                return Ok(());
            }
            _ => {
                return Err(format!(
                    "Can only schedule scenarios that are in the planning\
             phase but scenario was in phase {:?}",
                    self.get_status_str()
                ))
            }
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
    pub fn unschedule(&mut self) -> Result<(), String> {
        match self.status {
            Status::Scheduled => {
                self.status = Status::Planning;
                return Ok(());
            }
            _ => {
                return Err(format!(
                    "Can only unschedule scenarios that are in the\
            scheduled phase but scenario was in phase {:?}",
                    self.get_status_str()
                ))
            }
        }
    }

    pub fn set_running(&mut self, epoch: usize) {
        self.status = Status::Running(epoch);
    }

    pub fn set_done(&mut self) {
        self.status = Status::Done;
    }

    pub fn delete(&self) -> Result<(), std::io::Error> {
        let path = Path::new("./results").join(&self.id);
        fs::remove_dir_all(path)?;
        Ok(())
    }

    pub fn get_status(&self) -> &Status {
        &self.status
    }

    pub fn get_progress(&self) -> f32 {
        match self.status {
            Status::Running(epoch) => epoch as f32 / self.config.algorithm.epochs as f32,
            _ => 0.0,
        }
    }

    fn save_data(&self) {
        todo!()
    }

    fn save_results(&self) {
        todo!()
    }
}

pub fn run_scenario(mut scenario: Scenario, epoch_tx: Sender<usize>, summary_tx: Sender<Summary>) {
    let simulation = match &scenario.config.simulation {
        Some(simulation) => simulation,
        None => todo!("Non-simulation case not yet implemented."),
    };
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

    let mut summary = Summary::new();

    for epoch_index in 0..scenario.config.algorithm.epochs {
        algorithm::run_epoch(
            &mut model.functional_description,
            &mut results.estimations,
            &mut results.derivatives,
            &mut results.metrics,
            &data,
            scenario.config.algorithm.learning_rate,
            scenario.config.algorithm.model.apply_system_update,
            epoch_index,
        );
        scenario.status = Status::Running(epoch_index);

        summary.loss = results.metrics.loss_epoch.values[epoch_index];
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

        epoch_tx.send(epoch_index).unwrap();
        summary_tx.send(summary.clone()).unwrap();
    }
    scenario.results = Some(results);
    scenario.data = Some(data);
    scenario.save().expect("Could not save scenario");
    scenario.status = Status::Done;
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
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
