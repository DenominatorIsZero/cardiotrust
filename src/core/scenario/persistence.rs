use std::{
    collections::HashMap,
    fs::{self},
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::{Context, Result};
use tracing::debug;

use super::{results::Results, Scenario, ScenarioPayload};
use crate::core::data::Data;

#[derive(Debug, Clone)]
pub enum ScenarioStorage {
    Disk {
        root: Arc<PathBuf>,
    },
    Memory {
        data: Arc<std::sync::Mutex<HashMap<String, Vec<u8>>>>,
    },
}

impl ScenarioStorage {
    #[must_use]
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new_disk(project_root: impl Into<PathBuf>) -> Self {
        Self::Disk {
            root: Arc::new(project_root.into()),
        }
    }

    #[must_use]
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new_memory() -> Self {
        Self::Memory {
            data: Arc::new(std::sync::Mutex::new(HashMap::new())),
        }
    }

    /// Inserts raw bytes into a `Memory` variant storage.
    ///
    /// # Errors
    ///
    /// Returns an error if the storage is not `Memory` or the lock is poisoned.
    #[tracing::instrument(level = "debug", skip(self, value))]
    pub fn put_memory_bytes(&self, key: String, value: Vec<u8>) -> Result<()> {
        match self {
            Self::Memory { data } => {
                data.lock()
                    .map_err(|e| anyhow::anyhow!("Memory storage lock poisoned: {e}"))?
                    .insert(key, value);
                Ok(())
            }
            Self::Disk { .. } => {
                Err(anyhow::anyhow!("put_memory_bytes only works on Memory storage"))
            }
        }
    }

    #[must_use]
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new(project_root: impl Into<PathBuf>) -> Self {
        Self::new_disk(project_root)
    }

    #[must_use]
    #[tracing::instrument(level = "trace", skip_all)]
    pub fn project_root(&self) -> &Path {
        match self {
            Self::Disk { root } => root.as_ref().as_path(),
            Self::Memory { .. } => {
                panic!("project_root() not available on in-memory storage")
            }
        }
    }

    pub(crate) fn memory_key(scenario_id: &str, suffix: &str) -> String {
        format!("{scenario_id}/{suffix}")
    }

    #[tracing::instrument(level = "debug", skip(self, scenario))]
    pub fn save_metadata(&self, scenario: &Scenario) -> Result<()> {
        match self {
            Self::Disk { root } => {
                debug!(
                    "Saving scenario metadata for scenario {}",
                    scenario.get_id()
                );
                let scenario_dir = root.join(scenario.get_id());
                fs::create_dir_all(&scenario_dir)?;
                let toml = toml::to_string(scenario)
                    .context("Failed to serialize scenario metadata")?;
                fs::write(scenario_dir.join("scenario.toml"), toml)
                    .context("Failed to write scenario metadata")?;
            }
            Self::Memory { data } => {
                let key = Self::memory_key(scenario.get_id(), "scenario.toml");
                let toml = toml::to_string(scenario)
                    .context("Failed to serialize scenario metadata")?;
                data.lock()
                    .map_err(|e| anyhow::anyhow!("Memory storage lock poisoned: {e}"))?
                    .insert(key, toml.into_bytes());
            }
        }
        Ok(())
    }

    #[tracing::instrument(level = "info", skip_all)]
    pub fn load_metadata(&self, path: &Path) -> Result<Scenario> {
        match self {
            Self::Disk { .. } => {
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
            Self::Memory { .. } => {
                Err(anyhow::anyhow!(
                    "load_metadata(path) not supported for Memory storage; use load_metadata_by_id"
                ))
            }
        }
    }

    #[tracing::instrument(level = "info", skip_all)]
    pub fn load_metadata_by_id(&self, scenario_id: &str) -> Result<Scenario> {
        match self {
            Self::Disk { root } => self.load_metadata(&root.join(scenario_id)),
            Self::Memory { data } => {
                let key = Self::memory_key(scenario_id, "scenario.toml");
                let bytes = {
                    let guard = data
                        .lock()
                        .map_err(|e| anyhow::anyhow!("Memory storage lock poisoned: {e}"))?;
                    guard
                        .get(&key)
                        .with_context(|| format!("No metadata for scenario: {scenario_id}"))?
                        .clone()
                };
                let contents =
                    String::from_utf8(bytes).context("Metadata is not valid UTF-8")?;
                toml::from_str(&contents).context("Failed to parse scenario metadata")
            }
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn save_payload(&self, scenario_id: &str, payload: &ScenarioPayload) -> Result<()> {
        match self {
            Self::Disk { root } => {
                let scenario_dir = root.join(scenario_id);
                fs::create_dir_all(&scenario_dir)?;

                let serialized = postcard::to_stdvec(&payload.data)
                    .context("Failed to serialize data to binary format")?;
                fs::write(scenario_dir.join("data.bin"), &serialized)
                    .context("Failed to write data file")?;

                let serialized = postcard::to_stdvec(&payload.results)
                    .context("Failed to serialize results to binary format")?;
                fs::write(scenario_dir.join("results.bin"), &serialized)
                    .context("Failed to write results file")?;
            }
            Self::Memory { data } => {
                let data_key = Self::memory_key(scenario_id, "data.bin");
                let results_key = Self::memory_key(scenario_id, "results.bin");
                let mut guard = data
                    .lock()
                    .map_err(|e| anyhow::anyhow!("Memory storage lock poisoned: {e}"))?;

                let serialized = postcard::to_stdvec(&payload.data)
                    .context("Failed to serialize data to binary format")?;
                guard.insert(data_key, serialized);

                let serialized = postcard::to_stdvec(&payload.results)
                    .context("Failed to serialize results to binary format")?;
                guard.insert(results_key, serialized);
            }
        }
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn load_payload(&self, scenario_id: &str) -> Result<ScenarioPayload> {
        match self {
            Self::Disk { root } => {
                let data_path = root.join(scenario_id).join("data.bin");
                let data_bytes = fs::read(&data_path).with_context(|| {
                    format!("Failed to read data file: {}", data_path.display())
                })?;
                let data: Data = postcard::from_bytes(&data_bytes)
                    .context("Failed to deserialize data from binary format")?;

                let results_path = root.join(scenario_id).join("results.bin");
                let results_bytes = fs::read(&results_path).with_context(|| {
                    format!("Failed to read results file: {}", results_path.display())
                })?;
                let results: Results = postcard::from_bytes(&results_bytes)
                    .context("Failed to deserialize results from binary format")?;

                Ok(ScenarioPayload { data, results })
            }
            Self::Memory { data } => {
                let (data_bytes, results_bytes) = {
                    let guard = data
                        .lock()
                        .map_err(|e| anyhow::anyhow!("Memory storage lock poisoned: {e}"))?;
                    let data_key = Self::memory_key(scenario_id, "data.bin");
                    let results_key = Self::memory_key(scenario_id, "results.bin");

                    let data_bytes = guard
                        .get(&data_key)
                        .with_context(|| format!("No data payload for scenario: {scenario_id}"))?
                        .clone();
                    let results_bytes = guard
                        .get(&results_key)
                        .with_context(|| {
                            format!("No results payload for scenario: {scenario_id}")
                        })?
                        .clone();
                    drop(guard);
                    (data_bytes, results_bytes)
                };

                let data: Data = postcard::from_bytes(&data_bytes)
                    .context("Failed to deserialize data from binary format")?;
                let results: Results = postcard::from_bytes(&results_bytes)
                    .context("Failed to deserialize results from binary format")?;

                Ok(ScenarioPayload { data, results })
            }
        }
    }

    #[cfg(feature = "native")]
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn save_npy(&self, scenario_id: &str, payload: &ScenarioPayload) -> Result<()> {
        match self {
            Self::Disk { root } => {
                let path = root.join(scenario_id).join("npy");
                fs::create_dir_all(&path)?;
                payload.data.save_npy(&path.join("data"))?;
                payload.results.save_npy(&path.join("results"))?;
                Ok(())
            }
            Self::Memory { .. } => Err(anyhow::anyhow!(
                "save_npy not supported for in-memory storage"
            )),
        }
    }

    #[tracing::instrument(level = "info", skip_all)]
    pub fn delete_scenario(&self, scenario_id: &str) -> Result<()> {
        match self {
            Self::Disk { root } => {
                let scenario_dir = root.join(scenario_id);
                if scenario_dir.exists() {
                    fs::remove_dir_all(&scenario_dir).with_context(|| {
                        format!("Failed to delete {}", scenario_dir.display())
                    })?;
                }
            }
            Self::Memory { data } => {
                let mut guard = data
                    .lock()
                    .map_err(|e| anyhow::anyhow!("Memory storage lock poisoned: {e}"))?;
                // Remove all keys with this scenario_id prefix
                let prefix = format!("{scenario_id}/");
                guard.retain(|k, _| !k.starts_with(&prefix));
            }
        }
        Ok(())
    }

    // ── Disk-only path helpers ──────────────────────────────────────────

    fn disk_root(&self) -> &Path {
        match self {
            Self::Disk { root } => root.as_ref().as_path(),
            Self::Memory { .. } => {
                panic!("Path-based operations not available on in-memory storage")
            }
        }
    }

    fn disk_scenario_dir(&self, scenario_id: &str) -> PathBuf {
        self.disk_root().join(scenario_id)
    }

    #[must_use]
    #[tracing::instrument(level = "trace", skip_all)]
    pub fn scenario_dir(&self, scenario_id: &str) -> PathBuf {
        self.disk_scenario_dir(scenario_id)
    }

    #[must_use]
    #[tracing::instrument(level = "trace", skip_all)]
    pub fn metadata_path(&self, scenario_id: &str) -> PathBuf {
        self.disk_scenario_dir(scenario_id).join("scenario.toml")
    }

    #[must_use]
    #[tracing::instrument(level = "trace", skip_all)]
    pub fn data_path(&self, scenario_id: &str) -> PathBuf {
        self.disk_scenario_dir(scenario_id).join("data.bin")
    }

    #[must_use]
    #[tracing::instrument(level = "trace", skip_all)]
    pub fn results_path(&self, scenario_id: &str) -> PathBuf {
        self.disk_scenario_dir(scenario_id).join("results.bin")
    }

    #[must_use]
    #[tracing::instrument(level = "trace", skip_all)]
    pub fn image_path(&self, scenario_id: &str, image_name: &str) -> PathBuf {
        self.disk_scenario_dir(scenario_id)
            .join("img")
            .join(image_name)
            .with_extension("png")
    }

    #[must_use]
    #[tracing::instrument(level = "trace", skip_all)]
    pub fn animation_dir(&self, scenario_id: &str, animation_name: &str) -> PathBuf {
        self.disk_scenario_dir(scenario_id)
            .join("img")
            .join("anim")
            .join(animation_name)
    }

    #[must_use]
    #[tracing::instrument(level = "trace", skip_all)]
    pub fn export_path(&self, scenario_id: &str, file_name: &str) -> PathBuf {
        self.disk_scenario_dir(scenario_id)
            .join("export")
            .join(file_name)
    }

    #[must_use]
    #[tracing::instrument(level = "trace", skip_all)]
    pub fn npy_dir(&self, scenario_id: &str) -> PathBuf {
        self.disk_scenario_dir(scenario_id).join("npy")
    }
}
