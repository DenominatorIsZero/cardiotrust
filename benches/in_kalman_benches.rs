use std::time::Duration;

use cardiotrust::core::{
    algorithm::estimation::{
        calculate_k, calculate_kalman_gain, calculate_s_inv, estimate_state_covariance,
        predict_state_covariance, update_kalman_gain_and_check_convergence,
    },
    config::Config,
    data::Data,
    model::Model,
    scenario::results::Results,
};
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

const VOXEL_SIZES: [f32; 3] = [2.0, 2.5, 5.0];
const LEARNING_RATE: f32 = 1e-3;

fn run_benches(c: &mut Criterion) {
    let mut group = c.benchmark_group("In Kalman");
    bench_kalman(&mut group);
    bench_calculation(&mut group);
    bench_predict_state_covariance(&mut group);
    bench_calculate_s_inv(&mut group);
    bench_calculate_k(&mut group);
    bench_estimate_state_covariance(&mut group);
    group.finish();
}

fn bench_kalman(group: &mut criterion::BenchmarkGroup<criterion::measurement::WallTime>) {
    for voxel_size in VOXEL_SIZES.iter() {
        let config = setup_config(voxel_size);

        // setup inputs
        let (_, mut model, mut results) = setup_inputs(&config);

        // run bench
        let number_of_voxels = model.spatial_description.voxels.count();
        group.throughput(criterion::Throughput::Elements(number_of_voxels as u64));
        group.bench_function(BenchmarkId::new("update_and_check", voxel_size), |b| {
            b.iter(|| {
                update_kalman_gain_and_check_convergence(
                    &mut model.functional_description,
                    &mut results.estimations,
                    0,
                );
                results.estimations.kalman_gain_converged = false;
            })
        });
    }
}

fn bench_calculation(group: &mut criterion::BenchmarkGroup<criterion::measurement::WallTime>) {
    for voxel_size in VOXEL_SIZES.iter() {
        let config = setup_config(voxel_size);

        // setup inputs
        let (_, mut model, mut results) = setup_inputs(&config);

        // run bench
        let number_of_voxels = model.spatial_description.voxels.count();
        group.throughput(criterion::Throughput::Elements(number_of_voxels as u64));
        group.bench_function(BenchmarkId::new("calculate", voxel_size), |b| {
            b.iter(|| {
                calculate_kalman_gain(
                    &mut model.functional_description,
                    &mut results.estimations,
                    0,
                );
            })
        });
    }
}

fn bench_predict_state_covariance(
    group: &mut criterion::BenchmarkGroup<criterion::measurement::WallTime>,
) {
    for voxel_size in VOXEL_SIZES.iter() {
        let config = setup_config(voxel_size);

        // setup inputs
        let (_, model, mut results) = setup_inputs(&config);

        // run bench
        let number_of_voxels = model.spatial_description.voxels.count();
        group.throughput(criterion::Throughput::Elements(number_of_voxels as u64));
        group.bench_function(BenchmarkId::new("predict_covariance", voxel_size), |b| {
            b.iter(|| {
                predict_state_covariance(
                    &mut results.estimations.state_covariance_pred,
                    &results.estimations.state_covariance_est,
                    &model.functional_description.ap_params,
                    &model.functional_description.process_covariance,
                );
            })
        });
    }
}

fn bench_calculate_s_inv(group: &mut criterion::BenchmarkGroup<criterion::measurement::WallTime>) {
    for voxel_size in VOXEL_SIZES.iter() {
        let config = setup_config(voxel_size);

        // setup inputs
        let (_, model, mut results) = setup_inputs(&config);

        // prepare for bench
        predict_state_covariance(
            &mut results.estimations.state_covariance_pred,
            &results.estimations.state_covariance_est,
            &model.functional_description.ap_params,
            &model.functional_description.process_covariance,
        );

        // run bench
        let number_of_voxels = model.spatial_description.voxels.count();
        let measurement_matrix = model.functional_description.measurement_matrix.at_beat(0);
        group.throughput(criterion::Throughput::Elements(number_of_voxels as u64));
        group.bench_function(BenchmarkId::new("s_inv", voxel_size), |b| {
            b.iter(|| {
                calculate_s_inv(
                    &mut results.estimations.innovation_covariance,
                    &mut results.estimations.state_covariance_pred,
                    &model.functional_description.measurement_covariance,
                    &model.functional_description.ap_params,
                    &measurement_matrix,
                );
            })
        });
    }
}

fn bench_calculate_k(group: &mut criterion::BenchmarkGroup<criterion::measurement::WallTime>) {
    for voxel_size in VOXEL_SIZES.iter() {
        let config = setup_config(voxel_size);

        // setup inputs
        let (_, mut model, mut results) = setup_inputs(&config);

        let measurement_matrix = model.functional_description.measurement_matrix.at_beat(0);
        // prepare for bench
        predict_state_covariance(
            &mut results.estimations.state_covariance_pred,
            &results.estimations.state_covariance_est,
            &model.functional_description.ap_params,
            &model.functional_description.process_covariance,
        );
        calculate_s_inv(
            &mut results.estimations.innovation_covariance,
            &mut results.estimations.state_covariance_pred,
            &model.functional_description.measurement_covariance,
            &model.functional_description.ap_params,
            &measurement_matrix,
        );

        // run bench
        let number_of_voxels = model.spatial_description.voxels.count();
        group.throughput(criterion::Throughput::Elements(number_of_voxels as u64));
        group.bench_function(BenchmarkId::new("calculate_k", voxel_size), |b| {
            b.iter(|| {
                calculate_k(
                    &mut model.functional_description.kalman_gain,
                    &results.estimations.state_covariance_pred,
                    &results.estimations.innovation_covariance,
                    &model.functional_description.ap_params,
                    &measurement_matrix,
                );
            })
        });
    }
}
fn bench_estimate_state_covariance(
    group: &mut criterion::BenchmarkGroup<criterion::measurement::WallTime>,
) {
    for voxel_size in VOXEL_SIZES.iter() {
        let config = setup_config(voxel_size);

        // setup inputs
        let (_, mut model, mut results) = setup_inputs(&config);
        let measurement_matrix = model.functional_description.measurement_matrix.at_beat(0);

        // prepare for bench
        predict_state_covariance(
            &mut results.estimations.state_covariance_pred,
            &results.estimations.state_covariance_est,
            &model.functional_description.ap_params,
            &model.functional_description.process_covariance,
        );
        calculate_s_inv(
            &mut results.estimations.innovation_covariance,
            &mut results.estimations.state_covariance_pred,
            &model.functional_description.measurement_covariance,
            &model.functional_description.ap_params,
            &measurement_matrix,
        );
        calculate_k(
            &mut model.functional_description.kalman_gain,
            &results.estimations.state_covariance_pred,
            &results.estimations.innovation_covariance,
            &model.functional_description.ap_params,
            &measurement_matrix,
        );

        // run bench
        let number_of_voxels = model.spatial_description.voxels.count();
        group.throughput(criterion::Throughput::Elements(number_of_voxels as u64));
        group.bench_function(BenchmarkId::new("estimate_covariance", voxel_size), |b| {
            b.iter(|| {
                estimate_state_covariance(
                    &mut results.estimations.state_covariance_est,
                    &results.estimations.state_covariance_pred,
                    &model.functional_description.ap_params,
                    &measurement_matrix,
                    &model.functional_description.kalman_gain,
                );
            })
        });
    }
}

fn setup_config(voxel_size: &f32) -> Config {
    let samplerate_hz = 2000.0 * 2.5 / voxel_size;
    let mut config = Config::default();
    config.simulation.model.common.voxel_size_mm = *voxel_size;
    config.simulation.sample_rate_hz = samplerate_hz;
    config.algorithm.model.common.voxel_size_mm = *voxel_size;
    config.algorithm.model.common.apply_system_update = true;
    config.algorithm.update_kalman_gain = false;
    config.algorithm.learning_rate = LEARNING_RATE;
    config.algorithm.freeze_delays = false;
    config.algorithm.freeze_gains = false;
    config.algorithm.batch_size = 0;
    config
}

fn setup_inputs(config: &Config) -> (Data, Model, Results) {
    let simulation_config = &config.simulation;
    let data =
        Data::from_simulation_config(simulation_config).expect("Model parameters to be valid.");
    let model = Model::from_model_config(
        &config.algorithm.model,
        simulation_config.sample_rate_hz,
        simulation_config.duration_s,
    )
    .unwrap();
    let results = Results::new(
        config.algorithm.epochs,
        data.simulation.measurements.num_steps(),
        model.spatial_description.sensors.count(),
        model.spatial_description.voxels.count_states(),
        model.spatial_description.sensors.count_beats(),
        config.algorithm.batch_size,
        config.algorithm.optimizer,
    );
    (data, model, results)
}

criterion_group! {name = benches;
config = Criterion::default().measurement_time(Duration::from_secs(30)).sample_size(20);
targets=run_benches}
criterion_main!(benches);
