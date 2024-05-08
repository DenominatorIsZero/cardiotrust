use cardiotrust::core::{
    algorithm::{
        calculate_deltas, constrain_system_states,
        estimation::{
            calculate_residuals, calculate_system_update, prediction::calculate_system_prediction,
            update_kalman_gain_and_check_convergence,
        },
    },
    config::Config,
    data::Data,
    model::Model,
    scenario::results::Results,
};
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use std::time::Duration;

const VOXEL_SIZES: [f32; 3] = [2.0, 2.5, 5.0];
const LEARNING_RATE: f32 = 1e-3;
const TIME_INDEX: usize = 42;
const BEAT_INDEX: usize = 0;

fn run_benches(c: &mut Criterion) {
    let mut group = c.benchmark_group("In Epoch");
    bench_resetting(&mut group);
    bench_system_prediction(&mut group);
    bench_residuals(&mut group);
    bench_constrain(&mut group);
    bench_kalman(&mut group);
    bench_system_update(&mut group);
    bench_derivation(&mut group);
    bench_deltas(&mut group);
    bench_metrics(&mut group);
    bench_update_parameters(&mut group);
    group.finish();
}

fn resetting(results: &mut Results) {
    results.estimations.reset();
    results.derivatives.reset();
}

fn bench_resetting(group: &mut criterion::BenchmarkGroup<criterion::measurement::WallTime>) {
    for voxel_size in VOXEL_SIZES.iter() {
        let config = setup_config(voxel_size);

        // setup inputs
        let (_, model, mut results) = setup_inputs(&config);

        // run bench
        let number_of_voxels = model.spatial_description.voxels.count();
        group.throughput(criterion::Throughput::Elements(number_of_voxels as u64));
        group.bench_function(BenchmarkId::new("resetting", voxel_size), |b| {
            b.iter(|| {
                resetting(&mut results);
            })
        });
    }
}

fn bench_system_prediction(
    group: &mut criterion::BenchmarkGroup<criterion::measurement::WallTime>,
) {
    for voxel_size in VOXEL_SIZES.iter() {
        let config = setup_config(voxel_size);

        // setup inputs
        let (_, model, mut results) = setup_inputs(&config);

        // run bench
        let number_of_voxels = model.spatial_description.voxels.count();
        group.throughput(criterion::Throughput::Elements(number_of_voxels as u64));
        group.bench_function(BenchmarkId::new("system_prediction", voxel_size), |b| {
            b.iter(|| {
                calculate_system_prediction(
                    &mut results.estimations.ap_outputs,
                    &mut results.estimations.system_states,
                    &mut results.estimations.measurements,
                    &model.functional_description,
                    TIME_INDEX,
                    BEAT_INDEX,
                )
            })
        });
    }
}

fn bench_residuals(group: &mut criterion::BenchmarkGroup<criterion::measurement::WallTime>) {
    for voxel_size in VOXEL_SIZES.iter() {
        let config = setup_config(voxel_size);

        // setup inputs
        let (data, model, mut results) = setup_inputs(&config);

        // run bench
        let number_of_voxels = model.spatial_description.voxels.count();
        group.throughput(criterion::Throughput::Elements(number_of_voxels as u64));
        group.bench_function(BenchmarkId::new("residuals", voxel_size), |b| {
            b.iter(|| {
                calculate_residuals(
                    &mut results.estimations.residuals,
                    &results.estimations.measurements,
                    data.get_measurements(),
                    TIME_INDEX,
                    BEAT_INDEX,
                );
            })
        });
    }
}

fn bench_constrain(group: &mut criterion::BenchmarkGroup<criterion::measurement::WallTime>) {
    for voxel_size in VOXEL_SIZES.iter() {
        let config = setup_config(voxel_size);

        // setup inputs
        let (_, model, mut results) = setup_inputs(&config);

        // run bench
        let number_of_voxels = model.spatial_description.voxels.count();
        group.throughput(criterion::Throughput::Elements(number_of_voxels as u64));
        group.bench_function(BenchmarkId::new("constrain", voxel_size), |b| {
            b.iter(|| {
                constrain_system_states(
                    &mut results.estimations.system_states,
                    TIME_INDEX,
                    config.algorithm.state_clamping_threshold,
                );
            })
        });
    }
}

fn bench_derivation(group: &mut criterion::BenchmarkGroup<criterion::measurement::WallTime>) {
    for voxel_size in VOXEL_SIZES.iter() {
        let config = setup_config(voxel_size);

        // setup inputs
        let (_, model, mut results) = setup_inputs(&config);

        // run bench
        let number_of_voxels = model.spatial_description.voxels.count();
        group.throughput(criterion::Throughput::Elements(number_of_voxels as u64));
        group.bench_function(BenchmarkId::new("derivation", voxel_size), |b| {
            b.iter(|| {
                results.derivatives.calculate(
                    &model.functional_description,
                    &results.estimations,
                    &config.algorithm,
                    TIME_INDEX,
                    BEAT_INDEX,
                );
            })
        });
    }
}

fn bench_kalman(group: &mut criterion::BenchmarkGroup<criterion::measurement::WallTime>) {
    for voxel_size in VOXEL_SIZES.iter() {
        let config = setup_config(voxel_size);

        // setup inputs
        let (_, mut model, mut results) = setup_inputs(&config);

        // run bench
        let number_of_voxels = model.spatial_description.voxels.count();
        group.throughput(criterion::Throughput::Elements(number_of_voxels as u64));
        group.bench_function(
            BenchmarkId::new("update_kalman_gain_and_check_congergence", voxel_size),
            |b| {
                b.iter(|| {
                    update_kalman_gain_and_check_convergence(
                        &mut results.estimations,
                        &mut model.functional_description,
                        BEAT_INDEX,
                    );
                    results.estimations.kalman_gain_converged = false;
                })
            },
        );
    }
}

fn bench_system_update(group: &mut criterion::BenchmarkGroup<criterion::measurement::WallTime>) {
    for voxel_size in VOXEL_SIZES.iter() {
        let config = setup_config(voxel_size);

        // setup inputs
        let (_, mut model, mut results) = setup_inputs(&config);

        // run bench
        let number_of_voxels = model.spatial_description.voxels.count();
        group.throughput(criterion::Throughput::Elements(number_of_voxels as u64));
        group.bench_function(BenchmarkId::new("system_update", voxel_size), |b| {
            b.iter(|| {
                calculate_system_update(
                    &mut results.estimations,
                    TIME_INDEX,
                    &mut model.functional_description,
                    &config.algorithm,
                );
            })
        });
    }
}

fn bench_deltas(group: &mut criterion::BenchmarkGroup<criterion::measurement::WallTime>) {
    for voxel_size in VOXEL_SIZES.iter() {
        let config = setup_config(voxel_size);

        // setup inputs
        let (data, model, mut results) = setup_inputs(&config);

        // run bench
        let number_of_voxels = model.spatial_description.voxels.count();
        group.throughput(criterion::Throughput::Elements(number_of_voxels as u64));
        group.bench_function(BenchmarkId::new("deltas", voxel_size), |b| {
            b.iter(|| {
                calculate_deltas(
                    &mut results.estimations,
                    &model.functional_description,
                    &data,
                    TIME_INDEX,
                    BEAT_INDEX,
                );
            })
        });
    }
}

fn bench_metrics(group: &mut criterion::BenchmarkGroup<criterion::measurement::WallTime>) {
    for voxel_size in VOXEL_SIZES.iter() {
        let config = setup_config(voxel_size);

        // setup inputs
        let (_, model, mut results) = setup_inputs(&config);

        // run bench
        let number_of_voxels = model.spatial_description.voxels.count();
        group.throughput(criterion::Throughput::Elements(number_of_voxels as u64));
        group.bench_function(BenchmarkId::new("metrics", voxel_size), |b| {
            b.iter(|| {
                results.metrics.calculate_step(
                    &results.estimations,
                    &results.derivatives,
                    config.algorithm.regularization_strength,
                    TIME_INDEX,
                );
            })
        });
    }
}

fn bench_update_parameters(
    group: &mut criterion::BenchmarkGroup<criterion::measurement::WallTime>,
) {
    for voxel_size in VOXEL_SIZES.iter() {
        let config = setup_config(voxel_size);

        // setup inputs
        let (_, mut model, results) = setup_inputs(&config);

        // run bench
        let number_of_voxels = model.spatial_description.voxels.count();
        group.throughput(criterion::Throughput::Elements(number_of_voxels as u64));
        group.bench_function(BenchmarkId::new("update_parameters", voxel_size), |b| {
            b.iter(|| {
                model.functional_description.ap_params.update(
                    &results.derivatives,
                    &config.algorithm,
                    results.estimations.system_states.values.shape()[0],
                    model.spatial_description.sensors.count_beats(),
                );
            })
        });
    }
}

fn setup_config(voxel_size: &f32) -> Config {
    let samplerate_hz = 2000.0 * 2.5 / voxel_size;
    let mut config = Config::default();
    config
        .simulation
        .as_mut()
        .unwrap()
        .model
        .common
        .voxel_size_mm = *voxel_size;
    config.simulation.as_mut().unwrap().sample_rate_hz = samplerate_hz;
    config.algorithm.model.common.voxel_size_mm = *voxel_size;
    config.algorithm.model.common.apply_system_update = true;
    config.algorithm.update_kalman_gain = false;
    config.algorithm.learning_rate = LEARNING_RATE;
    config.algorithm.freeze_delays = false;
    config.algorithm.freeze_gains = false;
    config.algorithm.batch_size = 0;
    config.algorithm.constrain_system_states = true;
    config
}

fn setup_inputs(config: &Config) -> (Data, Model, Results) {
    let simulation_config = config.simulation.as_ref().unwrap();
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
        data.get_measurements().values.shape()[1],
        model.spatial_description.sensors.count(),
        model.spatial_description.voxels.count_states(),
        model.spatial_description.sensors.count_beats(),
    );
    (data, model, results)
}

criterion_group! {name = benches;
config = Criterion::default().measurement_time(Duration::from_secs(30)).sample_size(20);
targets=run_benches}
criterion_main!(benches);
