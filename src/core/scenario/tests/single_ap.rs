use std::{fs, path::Path, sync::mpsc::channel, thread};

use crate::core::scenario::{run, Scenario};

#[test]
fn optimize_single_ap() {
    let id = "single_ap_basic".to_string();
    let path = Path::new("./results").join(&id);
    if path.is_dir() {
        fs::remove_dir_all(&path).unwrap();
    }
    let mut scenario = Scenario::build(Some(id));
    scenario.config.simulation.model.common.pathological = true;
    scenario.config.algorithm.epochs = 100;
    assert!(path.is_dir());
    assert!(path.join("scenario.toml").is_file());
    scenario.schedule().unwrap();
    let send_scenario = scenario.clone();
    let (epoch_tx, _epoch_rx) = channel();
    let (summary_tx, _summary_rx) = channel();
    let handle = thread::spawn(move || run(send_scenario, &epoch_tx, &summary_tx));
    handle.join().unwrap();
    assert!(path.join("results.bin").is_file());
}
