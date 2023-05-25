use std::error::Error;
use std::path::Path;
use std::{fs, fs::File, io::Write};

use chrono;
use serde::{Deserialize, Serialize};
use toml;

use super::{config::Config, data::Data, results::Results};

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct Scenario {
    id: String,
    status: Status,
    config: Config,
    data: Option<Data>,
    results: Option<Results>,
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

    pub fn delete(&self) -> Result<(), std::io::Error> {
        let path = Path::new("./results").join(&self.id);
        fs::remove_dir_all(path)?;
        Ok(())
    }

    pub(crate) fn get_status(&self) -> &Status {
        &self.status
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum Status {
    Planning,
    Done,
    Running(f32),
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
