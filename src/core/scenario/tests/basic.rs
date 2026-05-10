use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::Context;

use crate::{
    core::scenario::{results::Results, Scenario, ScenarioPayload, ScenarioStorage},
    ScenarioBundle, ScenarioList,
};

fn test_project_dir(name: &str) -> PathBuf {
    Path::new("./results").join(name)
}

#[test]
fn building_is_in_memory_only() -> anyhow::Result<()> {
    let path = test_project_dir("test");
    if path.is_dir() {
        fs::remove_dir_all(&path)?;
    }
    let scenario = Scenario::build(Some("test".to_string()));
    assert_eq!(scenario.get_id(), &"test".to_string());
    assert!(!path.exists());
    Ok(())
}

#[test]
fn loading_scenarios_works() -> anyhow::Result<()> {
    let project_dir = test_project_dir("test2-project");
    if project_dir.is_dir() {
        fs::remove_dir_all(&project_dir).context("Failed to remove test directory during setup")?;
    }
    let storage = ScenarioStorage::new(project_dir.clone());
    let scenario = Scenario::build(Some("test2".to_string()));
    let bundle = ScenarioBundle::create(storage.clone(), scenario.clone())?;

    let loaded = storage.load_metadata(&project_dir.join(bundle.scenario.get_id()))?;

    assert_eq!(scenario, loaded);

    fs::remove_dir_all(&project_dir).context("Failed to remove test directory during cleanup")?;
    Ok(())
}

#[test]
fn scenario_list_loads_metadata_without_payload() -> anyhow::Result<()> {
    let project_dir = test_project_dir("metadata-only-project");
    if project_dir.is_dir() {
        fs::remove_dir_all(&project_dir).context("Failed to remove test directory during setup")?;
    }

    let storage = ScenarioStorage::new(project_dir.clone());
    let mut scenario = Scenario::build(Some("done-scenario".to_string()));
    scenario.set_done();
    ScenarioBundle::create(storage, scenario)?;

    let loaded = ScenarioList::load_from(&project_dir)?;

    assert_eq!(loaded.project_root.as_deref(), Some(project_dir.as_path()));
    assert_eq!(loaded.entries.len(), 1);
    assert_eq!(
        loaded.entries[0].scenario.get_id(),
        &"done-scenario".to_string()
    );
    assert_eq!(loaded.entries[0].scenario.get_status_str(), "Done (0s)");

    fs::remove_dir_all(&project_dir).context("Failed to remove test directory during cleanup")?;
    Ok(())
}

#[test]
fn payload_loading_requires_both_parts() -> anyhow::Result<()> {
    let project_dir = test_project_dir("payload-project");
    if project_dir.is_dir() {
        fs::remove_dir_all(&project_dir).context("Failed to remove test directory during setup")?;
    }

    let storage = ScenarioStorage::new(project_dir.clone());
    let scenario = Scenario::build(Some("payload-scenario".to_string()));
    let bundle = ScenarioBundle::create(storage.clone(), scenario)?;
    let payload = ScenarioPayload {
        data: crate::core::data::Data::get_default()?,
        results: Results::get_default(),
    };

    fs::create_dir_all(storage.scenario_dir(bundle.scenario.get_id()))?;
    let serialized = postcard::to_stdvec(&payload.data)
        .context("Failed to serialize test data payload")?;
    std::fs::write(storage.data_path(bundle.scenario.get_id()), &serialized)
        .context("Failed to write test data file")?;

    let err = bundle
        .load_payload()
        .expect_err("payload load should fail when one payload file is missing");

    assert!(
        err.to_string().contains("Failed to open results file")
            || err.to_string().contains("results.bin"),
        "unexpected payload error: {err:#}"
    );

    fs::remove_dir_all(&project_dir).context("Failed to remove test directory during cleanup")?;
    Ok(())
}

#[test]
fn storage_paths_are_isolated_per_project_for_same_scenario_id() -> anyhow::Result<()> {
    let project_a = test_project_dir("isolated-project-a");
    let project_b = test_project_dir("isolated-project-b");
    for dir in [&project_a, &project_b] {
        if dir.is_dir() {
            fs::remove_dir_all(dir).context("Failed to remove test directory during setup")?;
        }
    }

    let storage_a = ScenarioStorage::new(project_a.clone());
    let storage_b = ScenarioStorage::new(project_b.clone());
    let scenario_id = "shared-scenario-id";

    let image_a = storage_a.image_path(scenario_id, "activation-time");
    let image_b = storage_b.image_path(scenario_id, "activation-time");
    let anim_a = storage_a.animation_dir(scenario_id, "phase-map");
    let anim_b = storage_b.animation_dir(scenario_id, "phase-map");
    let export_a = storage_a.export_path(scenario_id, "animation.png");
    let export_b = storage_b.export_path(scenario_id, "animation.png");
    let npy_a = storage_a.npy_dir(scenario_id);
    let npy_b = storage_b.npy_dir(scenario_id);

    assert_ne!(image_a, image_b);
    assert_ne!(anim_a, anim_b);
    assert_ne!(export_a, export_b);
    assert_ne!(npy_a, npy_b);
    assert!(image_a.starts_with(&project_a));
    assert!(image_b.starts_with(&project_b));
    assert!(anim_a.starts_with(&project_a));
    assert!(anim_b.starts_with(&project_b));
    assert!(export_a.starts_with(&project_a));
    assert!(export_b.starts_with(&project_b));
    assert!(npy_a.starts_with(&project_a));
    assert!(npy_b.starts_with(&project_b));

    Ok(())
}
