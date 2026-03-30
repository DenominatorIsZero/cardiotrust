use std::{
    fs::{self, File},
    io::BufReader,
    path::Path,
};

use anyhow::{Context, Result};
use tracing::debug;

use super::Scenario;

impl Scenario {
    /// Saves the scenario data to a file in the results directory.
    ///
    /// # Errors
    ///
    /// This function will return an error if the results directory could not be created or the data file could not be written.
    #[tracing::instrument(level = "debug")]
    pub(super) fn save_data(&self) -> Result<()> {
        debug!("Saving scenario data for scenario with id {}", self.id);
        let path = Path::new("./results").join(&self.id);
        fs::create_dir_all(&path)?;
        let mut f = File::create(path.join("data.bin"))?;
        let data = self
            .data
            .as_ref()
            .context("Data not available for saving")?;
        bincode::serde::encode_into_std_write(data, &mut f, bincode::config::standard())
            .context("Failed to serialize data to binary format")?;
        Ok(())
    }

    /// Saves the scenario results to a file in the results directory.
    ///
    /// # Errors
    ///
    /// This function will return an error if the results directory could not be created or the results file could not be written.
    #[tracing::instrument(level = "debug")]
    pub(super) fn save_results(&self) -> Result<()> {
        debug!("Saving scenario results for scenario with id {}", self.id);
        let path = Path::new("./results").join(&self.id);
        fs::create_dir_all(&path)?;
        let mut f = File::create(path.join("results.bin"))?;
        let results = self
            .results
            .as_ref()
            .context("Results not available for saving")?;
        bincode::serde::encode_into_std_write(results, &mut f, bincode::config::standard())
            .context("Failed to serialize results to binary format")?;
        Ok(())
    }

    /// Loads the scenario data from the data.bin file in the results directory if it exists.
    ///
    /// # Errors
    ///
    /// Returns an error if the data.bin file cannot be read or parsed.
    #[tracing::instrument(level = "debug")]
    pub fn load_data(&mut self) -> Result<()> {
        debug!("Loading scenario data for scenario with id {}", self.id);
        if self.data.is_some() {
            return Ok(());
        }
        let file_path = Path::new("./results").join(&self.id).join("data.bin");
        if file_path.is_file() {
            let file = File::open(&file_path)
                .with_context(|| format!("Failed to open data file: {}", file_path.display()))?;
            self.data = Some(
                bincode::serde::decode_from_std_read(
                    &mut BufReader::new(file),
                    bincode::config::standard(),
                )
                .context("Failed to deserialize data from binary format")?,
            );
        }
        Ok(())
    }

    /// Loads the scenario results from the results.bin file in the results directory if it exists.
    ///
    /// # Errors
    ///
    /// Returns an error if the results.bin file cannot be read or parsed.
    #[tracing::instrument(level = "debug")]
    pub fn load_results(&mut self) -> Result<()> {
        debug!("Loading scenario results for scenario with id {}", self.id);
        if self.results.is_some() {
            return Ok(());
        }
        let file_path = Path::new("./results").join(&self.id).join("results.bin");
        if file_path.is_file() {
            let file = File::open(&file_path)
                .with_context(|| format!("Failed to open results file: {}", file_path.display()))?;
            self.results = Some(
                bincode::serde::decode_from_std_read(
                    &mut BufReader::new(file),
                    bincode::config::standard(),
                )
                .context("Failed to deserialize results from binary format")?,
            );
        }
        Ok(())
    }

    /// Saves the scenario data and results as .npy files in the results directory.
    ///
    /// # Errors
    ///
    /// Returns an error if file or directory creation fails or any save operation fails.
    #[tracing::instrument(level = "debug")]
    pub fn save_npy(&self) -> Result<()> {
        debug!("Saving scenario data and results as npy");
        let path = Path::new("./results").join(&self.id).join("npy");
        self.data
            .as_ref()
            .context("Scenario data not available for NPY export")?
            .save_npy(&path.join("data"))?;
        self.results
            .as_ref()
            .context("Scenario results not available for NPY export")?
            .save_npy(&path.join("results"))?;
        Ok(())
    }
}
