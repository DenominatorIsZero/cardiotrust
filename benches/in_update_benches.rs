use cardiotrust::core::{
    algorithm::{
        estimation::{calculate_residuals, prediction::calculate_system_prediction},
        refinement::update::{update_delays, update_gains},
        run_epoch,
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

fn run_benches(c: &mut Criterion) {
    let mut group = c.benchmark_group("In System Prediction");
    bench_gains(&mut group);
    bench_delays(&mut group);
    group.finish();
}

fn bench_gains(group: &mut criterion::BenchmarkGroup<criterion::measurement::WallTime>) {
    for voxel_size in VOXEL_SIZES.iter() {
        let config = setup_config(voxel_size);

        // setup inputs
        let (_, mut model, results) = setup_inputs(&config);

        // run bench
        let number_of_voxels = model.spatial_description.voxels.count();
        group.throughput(criterion::Throughput::Elements(number_of_voxels as u64));
        group.bench_function(BenchmarkId::new("gains", voxel_size), |b| {
            b.iter(|| {
                update_gains(
                    &mut model.functional_description.ap_params.gains,
                    &results.derivatives.gains,
                    config.algorithm.learning_rate,
                    2000,
                    config.algorithm.gradient_clamping_threshold,
                );
            })
        });
    }
}

fn bench_delays(group: &mut criterion::BenchmarkGroup<criterion::measurement::WallTime>) {
    for voxel_size in VOXEL_SIZES.iter() {
        let config = setup_config(voxel_size);

        // setup inputs
        let (_, mut model, results) = setup_inputs(&config);

        // run bench
        let number_of_voxels = model.spatial_description.voxels.count();
        group.throughput(criterion::Throughput::Elements(number_of_voxels as u64));
        group.bench_function(BenchmarkId::new("delays", voxel_size), |b| {
            b.iter(|| {
                update_delays(
                    &mut model.functional_description.ap_params.coefs,
                    &mut model.functional_description.ap_params.delays,
                    &results.derivatives.coefs,
                    config.algorithm.learning_rate,
                    2000,
                    config.algorithm.gradient_clamping_threshold,
                );
            })
        });
    }
}

fn setup_config(voxel_size: &f32) -> Config {
    let samplerate_hz = 2000.0 * 2.5 / voxel_size;
    let mut config = Config::default();
    config.simulation.as_mut().unwrap().model.voxel_size_mm = *voxel_size;
    config.simulation.as_mut().unwrap().sample_rate_hz = samplerate_hz;
    config.algorithm.model.voxel_size_mm = *voxel_size;
    config.algorithm.model.apply_system_update = true;
    config.algorithm.calculate_kalman_gain = false;
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
    let mut model = Model::from_model_config(
        &config.algorithm.model,
        simulation_config.sample_rate_hz,
        simulation_config.duration_s,
    )
    .unwrap();
    let mut results = Results::new(
        config.algorithm.epochs,
        data.get_measurements().values.shape()[0],
        model.spatial_description.sensors.count(),
        model.spatial_description.voxels.count_states(),
    );

    run_epoch(
        &mut model.functional_description,
        &mut results,
        &data,
        &config.algorithm,
        TIME_INDEX,
    );

    (data, model, results)
}

criterion_group! {name = benches;
config = Criterion::default().measurement_time(Duration::from_secs(10));
targets=run_benches}
criterion_main!(benches);
