use std::{
    fs::{self, File},
    io::BufReader,
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::{Context, Result};
use tracing::debug;

use super::{results::Results, Scenario, ScenarioPayload};
use crate::core::data::Data;

#[derive(Debug, Clone)]
pub struct ScenarioStorage {
    root: Arc<PathBuf>,
}

impl ScenarioStorage {
    #[must_use]
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new(project_root: impl Into<PathBuf>) -> Self {
        Self {
            root: Arc::new(project_root.into()),
        }
    }

    #[must_use]
    #[tracing::instrument(level = "trace", skip_all)]
    pub fn project_root(&self) -> &Path {
        self.root.as_ref().as_path()
    }

    #[must_use]
    #[tracing::instrument(level = "trace", skip_all)]
    pub fn scenario_dir(&self, scenario_id: &str) -> PathBuf {
        self.project_root().join(scenario_id)
    }

    #[must_use]
    #[tracing::instrument(level = "trace", skip_all)]
    pub fn metadata_path(&self, scenario_id: &str) -> PathBuf {
        self.scenario_dir(scenario_id).join("scenario.toml")
    }

    #[must_use]
    #[tracing::instrument(level = "trace", skip_all)]
    pub fn data_path(&self, scenario_id: &str) -> PathBuf {
        self.scenario_dir(scenario_id).join("data.bin")
    }

    #[must_use]
    #[tracing::instrument(level = "trace", skip_all)]
    pub fn results_path(&self, scenario_id: &str) -> PathBuf {
        self.scenario_dir(scenario_id).join("results.bin")
    }

    #[must_use]
    #[tracing::instrument(level = "trace", skip_all)]
    pub fn image_path(&self, scenario_id: &str, image_name: &str) -> PathBuf {
        self.scenario_dir(scenario_id)
            .join("img")
            .join(image_name)
            .with_extension("png")
    }

    #[must_use]
    #[tracing::instrument(level = "trace", skip_all)]
    pub fn animation_dir(&self, scenario_id: &str, animation_name: &str) -> PathBuf {
        self.scenario_dir(scenario_id)
            .join("img")
            .join("anim")
            .join(animation_name)
    }

    #[must_use]
    #[tracing::instrument(level = "trace", skip_all)]
    pub fn export_path(&self, scenario_id: &str, file_name: &str) -> PathBuf {
        self.scenario_dir(scenario_id)
            .join("export")
            .join(file_name)
    }

    #[must_use]
    #[tracing::instrument(level = "trace", skip_all)]
    pub fn npy_dir(&self, scenario_id: &str) -> PathBuf {
        self.scenario_dir(scenario_id).join("npy")
    }

    #[tracing::instrument(level = "debug", skip(self, scenario))]
    pub fn save_metadata(&self, scenario: &Scenario) -> Result<()> {
        debug!(
            "Saving scenario metadata for scenario {}",
            scenario.get_id()
        );
        let scenario_dir = self.scenario_dir(scenario.get_id());
        fs::create_dir_all(&scenario_dir)?;
        let toml = toml::to_string(scenario).context("Failed to serialize scenario metadata")?;
        fs::write(self.metadata_path(scenario.get_id()), toml)
            .context("Failed to write scenario metadata")?;
        Ok(())
    }

    #[tracing::instrument(level = "info", skip_all)]
    pub fn load_metadata(&self, path: &Path) -> Result<Scenario> {
        let metadata_path = path.join("scenario.toml");
        let contents = fs::read_to_string(&metadata_path).with_context(|| {
            format!(
                "Failed to read scenario.toml file: {}",
                metadata_path.display()
            )
        })?;

        toml::from_str(&contents).with_context(|| {
            format!(
                "Failed to parse scenario.toml in directory: {}",
                path.display()
            )
        })
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn save_payload(&self, scenario_id: &str, payload: &ScenarioPayload) -> Result<()> {
        let scenario_dir = self.scenario_dir(scenario_id);
        fs::create_dir_all(&scenario_dir)?;

        let mut data_file = File::create(self.data_path(scenario_id))?;
        bincode::serde::encode_into_std_write(
            &payload.data,
            &mut data_file,
            bincode::config::standard(),
        )
        .context("Failed to serialize data to binary format")?;

        let mut results_file = File::create(self.results_path(scenario_id))?;
        bincode::serde::encode_into_std_write(
            &payload.results,
            &mut results_file,
            bincode::config::standard(),
        )
        .context("Failed to serialize results to binary format")?;
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn load_payload(&self, scenario_id: &str) -> Result<ScenarioPayload> {
        let data_path = self.data_path(scenario_id);
        let data_file = File::open(&data_path)
            .with_context(|| format!("Failed to open data file: {}", data_path.display()))?;
        let data: Data = bincode::serde::decode_from_std_read(
            &mut BufReader::new(data_file),
            bincode::config::standard(),
        )
        .context("Failed to deserialize data from binary format")?;

        let results_path = self.results_path(scenario_id);
        let results_file = File::open(&results_path)
            .with_context(|| format!("Failed to open results file: {}", results_path.display()))?;
        let results: Results = bincode::serde::decode_from_std_read(
            &mut BufReader::new(results_file),
            bincode::config::standard(),
        )
        .context("Failed to deserialize results from binary format")?;

        Ok(ScenarioPayload { data, results })
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn save_npy(&self, scenario_id: &str, payload: &ScenarioPayload) -> Result<()> {
        let path = self.npy_dir(scenario_id);
        payload.data.save_npy(&path.join("data"))?;
        payload.results.save_npy(&path.join("results"))?;
        Ok(())
    }

    #[tracing::instrument(level = "info", skip_all)]
    pub fn delete_scenario(&self, scenario_id: &str) -> Result<()> {
        let scenario_dir = self.scenario_dir(scenario_id);
        if scenario_dir.exists() {
            fs::remove_dir_all(&scenario_dir)
                .with_context(|| format!("Failed to delete {}", scenario_dir.display()))?;
        }
        Ok(())
    }
}
