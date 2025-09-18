use std::{fs, path::Path};

use crate::core::scenario::Scenario;

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
fn loading_scenarios_works() -> anyhow::Result<()> {
    let path = Path::new("./results/test2");
    if path.is_dir() {
        fs::remove_dir_all(path).unwrap();
    }
    let scenario = Scenario::build(Some("test2".to_string()));

    let loaded = Scenario::load(path)?;

    assert_eq!(scenario, loaded);

    fs::remove_dir_all(path).unwrap();
    Ok(())
}
