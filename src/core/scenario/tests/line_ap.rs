use std::{
    fs::{self, File},
    io::BufWriter,
    path::Path,
    sync::mpsc::channel,
    thread,
};

use ndarray::Array1;
use ndarray_npy::WriteNpyExt;

use super::RUN_IN_TESTS;
use crate::{
    core::{
        algorithm::{metrics::BatchWiseMetric, refinement::Optimizer},
        model::{functional::allpass::from_coef_to_samples, spatial::voxels::VoxelType},
        scenario::{run, tests::SAVE_NPY, Scenario},
    },
    tests::{clean_files, setup_folder},
    vis::plotting::png::line::{line_plot, log_y_plot},
};

const COMMON_PATH: &str = "tests/core/scenario/line_ap/";
const NUM_AP: i32 = 3;

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::too_many_lines
)]
#[test]
fn heavy_no_roll_down_dif_reg() {
    let base_id = "Line AP - No Roll - Down - Difference Regularization".to_string();
    let base_title = "Line AP - No Roll - Down - Difference Regularization";
    let path = Path::new(COMMON_PATH).join("no_roll_down/dif_reg");

    let integer_part = 5.0;
    let voxel_size_mm = 2.5;
    let sample_rate_hz = 2000.0;

    let initial_fractional = 0.25;
    let initial_delay = integer_part + initial_fractional;
    let initial_delay_s = initial_delay / sample_rate_hz;
    let initial_velocity = voxel_size_mm / 1000.0 / initial_delay_s;

    let target_fractional = 0.75;
    let target_delay = integer_part + target_fractional;
    let target_delay_s = target_delay / sample_rate_hz;
    let target_velocity = voxel_size_mm / 1000.0 / target_delay_s;

    let difference_regularization_strength = Array1::logspace(10., -7., -6., 10).to_vec();
    let slow_down_strength = vec![0.0];
    create_and_run(
        target_velocity,
        initial_velocity,
        difference_regularization_strength,
        &slow_down_strength,
        &base_id,
        &path,
        base_title,
    );
}

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::too_many_lines
)]
#[test]

fn heavy_no_roll_down_slow_down() {
    let base_id = "Line AP - No Roll - Down - Slow Down".to_string();
    let base_title = "Line AP - No Roll - Down - Slow Down";
    let path = Path::new(COMMON_PATH).join("no_roll_down/slow_down");

    let integer_part = 5.0;
    let voxel_size_mm = 2.5;
    let sample_rate_hz = 2000.0;

    let initial_fractional = 0.25;
    let initial_delay = integer_part + initial_fractional;
    let initial_delay_s = initial_delay / sample_rate_hz;
    let initial_velocity = voxel_size_mm / 1000.0 / initial_delay_s;

    let target_fractional = 0.75;
    let target_delay = integer_part + target_fractional;
    let target_delay_s = target_delay / sample_rate_hz;
    let target_velocity = voxel_size_mm / 1000.0 / target_delay_s;

    let difference_regularization_stregth = vec![6e-7];
    let slow_down_strength = Array1::logspace(10.0, 0.0, 1.0, 10).to_vec();

    create_and_run(
        target_velocity,
        initial_velocity,
        difference_regularization_stregth,
        &slow_down_strength,
        &base_id,
        &path,
        base_title,
    );
}

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::too_many_lines
)]
#[test]
fn heavy_no_roll_up_dif_reg() {
    let base_id = "Line AP - No Roll - Up - Difference Regularization".to_string();
    let base_title = "Line AP - No Roll - Up - Difference Regularization";
    let path = Path::new(COMMON_PATH).join("no_roll_up/dif_reg");

    let integer_part = 5.0;
    let voxel_size_mm = 2.5;
    let sample_rate_hz = 2000.0;

    let initial_fractional = 0.75;
    let initial_delay = integer_part + initial_fractional;
    let initial_delay_s = initial_delay / sample_rate_hz;
    let initial_velocity = voxel_size_mm / 1000.0 / initial_delay_s;

    let target_fractional = 0.25;
    let target_delay = integer_part + target_fractional;
    let target_delay_s = target_delay / sample_rate_hz;
    let target_velocity = voxel_size_mm / 1000.0 / target_delay_s;

    let difference_regularization_stregth = Array1::logspace(10., -7., -5., 10).to_vec();
    let slow_down_strength = vec![0.0];

    create_and_run(
        target_velocity,
        initial_velocity,
        difference_regularization_stregth,
        &slow_down_strength,
        &base_id,
        &path,
        base_title,
    );
}

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::too_many_lines
)]
#[tracing::instrument(level = "trace")]
fn build_scenario(
    target_velocity: f32,
    initial_velocity: f32,
    difference_regularization_strength: f32,
    slow_down_strength: f32,
    base_id: &str,
) -> Scenario {
    let mut scenario = Scenario::build(Some(format!(
        "{base_id} - d_r: {difference_regularization_strength:.2e}, s_d: {slow_down_strength:.2e}"
    )));

    // Set pathological true
    scenario.config.simulation.model.common.pathological = true;
    scenario
        .config
        .simulation
        .model
        .common
        .current_factor_in_pathology = 1.0;
    scenario
        .config
        .simulation
        .model
        .common
        .measurement_covariance_mean = 1e-12;
    // Adjust heart size
    scenario
        .config
        .simulation
        .model
        .handcrafted
        .as_mut()
        .unwrap()
        .heart_size_mm = [2.5, 2.5 * (NUM_AP + 1) as f32, 2.5];
    // Adjust pathology
    scenario
        .config
        .simulation
        .model
        .handcrafted
        .as_mut()
        .unwrap()
        .pathology_x_start_percentage = 0.0;
    scenario
        .config
        .simulation
        .model
        .handcrafted
        .as_mut()
        .unwrap()
        .pathology_x_stop_percentage = 1.0;
    scenario
        .config
        .simulation
        .model
        .handcrafted
        .as_mut()
        .unwrap()
        .pathology_y_start_percentage = 0.0;
    scenario
        .config
        .simulation
        .model
        .handcrafted
        .as_mut()
        .unwrap()
        .pathology_y_stop_percentage = 1.0;
    scenario
        .config
        .simulation
        .model
        .handcrafted
        .as_mut()
        .unwrap()
        .sa_y_center_percentage = 0.8;
    scenario.config.simulation.model.common.heart_offset_mm = [25.0, -250.0, 180.0];
    // Copy settings to algorithm model
    scenario.config.algorithm.model = scenario.config.simulation.model.clone();
    // Adjust propagation velocities
    *scenario
        .config
        .simulation
        .model
        .common
        .propagation_velocities_m_per_s
        .get_mut(&VoxelType::Sinoatrial)
        .unwrap() = target_velocity;
    *scenario
        .config
        .simulation
        .model
        .common
        .propagation_velocities_m_per_s
        .get_mut(&VoxelType::Pathological)
        .unwrap() = target_velocity;
    *scenario
        .config
        .algorithm
        .model
        .common
        .propagation_velocities_m_per_s
        .get_mut(&VoxelType::Sinoatrial)
        .unwrap() = initial_velocity;
    *scenario
        .config
        .algorithm
        .model
        .common
        .propagation_velocities_m_per_s
        .get_mut(&VoxelType::Pathological)
        .unwrap() = initial_velocity;
    // set optimization parameters
    scenario.config.algorithm.epochs = 5_000;
    scenario.config.algorithm.learning_rate = 1e4;
    scenario.config.algorithm.optimizer = Optimizer::Sgd;
    scenario.config.algorithm.freeze_delays = false;
    scenario.config.algorithm.freeze_gains = true;
    scenario.config.algorithm.difference_regularization_strength =
        difference_regularization_strength;
    scenario.config.algorithm.slow_down_stregth = slow_down_strength;
    let number_of_snapshots = 1000;
    scenario.config.algorithm.snapshots_interval =
        scenario.config.algorithm.epochs / number_of_snapshots;

    scenario.schedule().unwrap();
    let _ = scenario.save();
    scenario
}

#[allow(
    clippy::too_many_lines,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)]
#[tracing::instrument(level = "trace")]
fn plot_results(path: &Path, base_title: &str, scenarios: &Vec<Scenario>) {
    setup_folder(path);
    let files = vec![
        path.join("loss.png"),
        path.join("params.png"),
        path.join("params_error.png"),
        path.join("delays.png"),
        path.join("delays_error.png"),
    ];
    clean_files(&files);

    let first_scenario = scenarios.first().unwrap();

    println!(
        "{:?}",
        first_scenario
            .results
            .as_ref()
            .unwrap()
            .model
            .as_ref()
            .unwrap()
            .functional_description
            .ap_params
            .coefs
    );

    let x_epochs = Array1::range(0.0, first_scenario.config.algorithm.epochs as f32, 1.0);
    let num_snapshots = first_scenario.results.as_ref().unwrap().snapshots.len() as f32;
    let x_snapshots = Array1::range(0.0, num_snapshots, 1.0);
    let mut losses_owned: Vec<BatchWiseMetric> = Vec::new();
    let mut params_owned: Vec<Array1<f32>> = Vec::new();
    let mut params_error_owned: Vec<Array1<f32>> = Vec::new();
    let mut delays_owned: Vec<Array1<f32>> = Vec::new();
    let mut delays_error_owned: Vec<Array1<f32>> = Vec::new();
    let mut labels_owned: Vec<String> = Vec::new();

    for scenario in scenarios {
        losses_owned.push(
            scenario
                .results
                .as_ref()
                .unwrap()
                .metrics
                .loss_mse_batch
                .clone(),
        );
        labels_owned.push(format!(
            "d_r {:.2e}, s_d {:.2e}",
            scenario.config.algorithm.difference_regularization_strength,
            scenario.config.algorithm.slow_down_stregth
        ));
    }

    let losses = losses_owned
        .iter()
        .map(std::ops::Deref::deref)
        .collect::<Vec<&Array1<f32>>>();
    let labels: Vec<&str> = labels_owned
        .iter()
        .map(std::string::String::as_str)
        .collect();

    if SAVE_NPY {
        let path = path.join("npy");
        fs::create_dir_all(&path).unwrap();
        let writer = BufWriter::new(File::create(path.join("x_epochs.npy")).unwrap());
        x_epochs.write_npy(writer).unwrap();
        let writer = BufWriter::new(File::create(path.join("x_snapshots.npy")).unwrap());
        x_snapshots.write_npy(writer).unwrap();
        for (label, loss) in labels.iter().zip(losses.iter()) {
            let writer =
                BufWriter::new(File::create(path.join(format!("loss_{label}.npy"))).unwrap());
            loss.write_npy(writer).unwrap();
        }
    }

    log_y_plot(
        Some(&x_epochs),
        losses,
        Some(files[0].as_path()),
        Some(format!("{base_title} - Loss").as_str()),
        Some("Loss MSE"),
        Some("Epoch"),
        Some(&labels),
        None,
    )
    .unwrap();

    let mut labels_owned: Vec<String> = Vec::new();

    let mut min_loss_n = 0;
    let mut min_loss = 1e9;

    for (n, scenario) in scenarios.iter().enumerate() {
        if scenario.summary.as_ref().unwrap().loss_mse < min_loss {
            min_loss = scenario.summary.as_ref().unwrap().loss_mse;
            min_loss_n = n;
        }
    }

    let scenario = &scenarios[min_loss_n];

    for ap in 0..NUM_AP as usize {
        let mut ap_param = Array1::<f32>::zeros(num_snapshots as usize);
        let mut delays = Array1::<f32>::zeros(num_snapshots as usize);
        let mut delays_error = Array1::<f32>::zeros(num_snapshots as usize);
        let mut ap_param_error = Array1::<f32>::zeros(num_snapshots as usize);
        let target_param = scenario
            .data
            .as_ref()
            .unwrap()
            .simulation
            .model
            .functional_description
            .ap_params
            .coefs[(ap, 15)];
        let target_delay = from_coef_to_samples(
            scenario
                .data
                .as_ref()
                .unwrap()
                .simulation
                .model
                .functional_description
                .ap_params
                .coefs[(ap, 15)],
        ) + scenario
            .data
            .as_ref()
            .unwrap()
            .simulation
            .model
            .functional_description
            .ap_params
            .delays[(ap, 15)] as f32;

        for (i, snapshot) in scenario
            .results
            .as_ref()
            .unwrap()
            .snapshots
            .iter()
            .enumerate()
        {
            delays[i] =
                from_coef_to_samples(snapshot.functional_description.ap_params.coefs[(ap, 15)])
                    + snapshot.functional_description.ap_params.delays[(ap, 15)] as f32;
            delays_error[i] = target_delay - delays[i];
            ap_param[i] = snapshot.functional_description.ap_params.coefs[(ap, 15)];
            ap_param_error[i] = target_param - ap_param[i];
        }
        params_owned.push(ap_param);
        params_error_owned.push(ap_param_error);
        delays_owned.push(delays);
        delays_error_owned.push(delays_error);
        labels_owned.push(format!("AP {ap}"));
    }

    let params = params_owned.iter().collect::<Vec<&Array1<f32>>>();
    let params_error = params_error_owned.iter().collect::<Vec<&Array1<f32>>>();
    let delays = delays_owned.iter().collect::<Vec<&Array1<f32>>>();
    let delays_error = delays_error_owned.iter().collect::<Vec<&Array1<f32>>>();
    let labels: Vec<&str> = labels_owned
        .iter()
        .map(std::string::String::as_str)
        .collect();

    if SAVE_NPY {
        let path = path.join("npy");
        for (i, delay) in delays.iter().enumerate() {
            let writer = BufWriter::new(
                File::create(path.join(format!(
                    "delay_{i}_d_r{:.2e}, s_d {:.2e}.npy",
                    scenario.config.algorithm.difference_regularization_strength,
                    scenario.config.algorithm.slow_down_stregth
                )))
                .unwrap(),
            );
            delay.write_npy(writer).unwrap();
        }
    }

    line_plot(
        Some(&x_snapshots),
        params,
        Some(files[1].as_path()),
        Some(format!("{base_title} - AP Coef").as_str()),
        Some("AP Coef (Estimated)"),
        Some("Snapshot"),
        Some(&labels),
        None,
    )
    .unwrap();

    line_plot(
        Some(&x_snapshots),
        params_error,
        Some(files[2].as_path()),
        Some(format!("{base_title} - AP Coef Error").as_str()),
        Some("AP Coef (Target - Estimated)"),
        Some("Snapshot"),
        Some(&labels),
        None,
    )
    .unwrap();

    line_plot(
        Some(&x_snapshots),
        delays,
        Some(files[3].as_path()),
        Some(format!("{base_title} - AP Delay").as_str()),
        Some("AP Delay (Estimated)"),
        Some("Snapshot"),
        Some(&labels),
        None,
    )
    .unwrap();

    line_plot(
        Some(&x_snapshots),
        delays_error,
        Some(files[4].as_path()),
        Some(format!("{base_title} - AP Delay Error").as_str()),
        Some("AP Delay (Target - Estimated)"),
        Some("Snapshot"),
        Some(&labels),
        None,
    )
    .unwrap();
}

#[tracing::instrument(level = "trace")]
fn create_and_run(
    target_velocity: f32,
    initial_velocity: f32,
    difference_regularization_strengths: Vec<f32>,
    slow_down_strengths: &Vec<f32>,
    base_id: &str,
    img_path: &Path,
    base_title: &str,
) {
    let mut join_handles = Vec::new();
    let mut scenarios = Vec::new();

    for difference_regularization_strength in difference_regularization_strengths {
        for slow_down_strength in slow_down_strengths {
            let id = format!(
            "{base_id} - d_r: {difference_regularization_strength:.2e}, s_d: {slow_down_strength:.2e}",
        );
            let path = Path::new("results").join(id);
            println!("Looking for scenario {path:?}");
            let scenario = if path.is_dir() {
                println!("Found scenario. Loading it!");
                let mut scenario = Scenario::load(path.as_path());
                scenario.load_data();
                scenario.load_results();
                scenario
            } else {
                println!("Didn't find scenario. Building it!");
                let scenario = build_scenario(
                    target_velocity,
                    initial_velocity,
                    difference_regularization_strength,
                    *slow_down_strength,
                    base_id,
                );
                if RUN_IN_TESTS {
                    let send_scenario = scenario.clone();
                    let (epoch_tx, _) = channel();
                    let (summary_tx, _) = channel();
                    let handle = thread::spawn(move || run(send_scenario, &epoch_tx, &summary_tx));
                    println!("handle {handle:?}");
                    join_handles.push(handle);
                }
                scenario
            };
            scenarios.push(scenario);
        }
    }

    if RUN_IN_TESTS {
        for handle in join_handles {
            handle.join().unwrap();
        }
        for scenario in &mut scenarios {
            let path = Path::new("results").join(scenario.id.clone());
            *scenario = Scenario::load(path.as_path());
            scenario.load_data();
            scenario.load_results();
        }
    }

    plot_results(img_path, base_title, &scenarios);
}
