use std::{fs, path::Path};

use anyhow::Context;

use crate::core::scenario::Scenario;

#[test]
fn building_saves_scenario() -> anyhow::Result<()> {
    let path = Path::new("./results/test");
    if path.is_dir() {
        fs::remove_dir_all(path)?;
    }
    let _scenario = Scenario::build(Some("test".to_string()))?;
    assert!(path.is_dir());
    assert!(path.join("scenario.toml").is_file());
    fs::remove_dir_all(path)?;
    Ok(())
}

#[test]
fn loading_scenarios_works() -> anyhow::Result<()> {
    let path = Path::new("./results/test2");
    if path.is_dir() {
        fs::remove_dir_all(path).context("Failed to remove test directory during setup")?;
    }
    let scenario = Scenario::build(Some("test2".to_string()))?;

    let loaded = Scenario::load(path)?;

    assert_eq!(scenario, loaded);

    fs::remove_dir_all(path).context("Failed to remove test directory during cleanup")?;
    Ok(())
}
