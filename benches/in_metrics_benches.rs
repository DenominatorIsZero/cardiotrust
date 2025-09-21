use std::time::Duration;

use anyhow::Context;
use cardiotrust::core::{
    algorithm::{metrics, run_epoch},
    config::Config,
    data::Data,
    model::Model,
    scenario::results::Results,
};
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

const VOXEL_SIZES: [f32; 3] = [2.0, 2.5, 5.0];
const LEARNING_RATE: f32 = 1e-3;
const STEP: usize = 42;

fn run_benches(c: &mut Criterion) {
    let mut group = c.benchmark_group("In Metrics");
    bench_step(&mut group).expect("Benchmark should succeed");
    bench_epoch(&mut group).expect("Benchmark should succeed");
    group.finish();
}

fn bench_step(
    group: &mut criterion::BenchmarkGroup<criterion::measurement::WallTime>,
) -> anyhow::Result<()> {
    for voxel_size in VOXEL_SIZES.iter() {
        let config = setup_config(voxel_size);

        // setup inputs
        let mut results = setup_inputs(&config).context("Failed to setup benchmark inputs")?;

        // run bench
        let number_of_voxels = results
            .model
            .as_ref()
            .context("Model should be available in benchmark setup")?
            .spatial_description
            .voxels
            .count();
        group.throughput(criterion::Throughput::Elements(number_of_voxels as u64));
        group.bench_function(BenchmarkId::new("step", voxel_size), |b| {
            b.iter(|| {
                metrics::calculate_step(
                    &mut results.metrics,
                    &results.estimations,
                    results.derivatives.maximum_regularization_sum,
                    config.algorithm.maximum_regularization_strength,
                    STEP,
                );
            })
        });
    }
    Ok(())
}

fn bench_epoch(
    group: &mut criterion::BenchmarkGroup<criterion::measurement::WallTime>,
) -> anyhow::Result<()> {
    for voxel_size in VOXEL_SIZES.iter() {
        let config = setup_config(voxel_size);

        // setup inputs
        let mut results = setup_inputs(&config).context("Failed to setup benchmark inputs")?;

        // run bench
        let number_of_voxels = results
            .model
            .as_ref()
            .context("Model should be available in benchmark setup")?
            .spatial_description
            .voxels
            .count();
        group.throughput(criterion::Throughput::Elements(number_of_voxels as u64));
        group.bench_function(BenchmarkId::new("epoch", voxel_size), |b| {
            b.iter(|| {
                metrics::calculate_batch(&mut results.metrics, 0).expect("Calculation to succeed.");
            })
        });
    }
    Ok(())
}

fn setup_config(voxel_size: &f32) -> Config {
    let samplerate_hz = 2000.0 * 2.5 / voxel_size;
    let mut config = Config::default();
    config.simulation.model.common.voxel_size_mm = *voxel_size;
    config.simulation.sample_rate_hz = samplerate_hz;
    config.algorithm.model.common.voxel_size_mm = *voxel_size;
    config.algorithm.learning_rate = LEARNING_RATE;
    config.algorithm.freeze_delays = false;
    config.algorithm.freeze_gains = false;
    config.algorithm.batch_size = 0;
    config
}

fn setup_inputs(config: &Config) -> anyhow::Result<Results> {
    let simulation_config = &config.simulation;
    let data = Data::from_simulation_config(simulation_config)
        .context("Failed to create simulation data from config")?;
    let model = Model::from_model_config(
        &config.algorithm.model,
        simulation_config.sample_rate_hz,
        simulation_config.duration_s,
    )
    .context("Failed to create model from config")?;
    let mut results = Results::new(
        config.algorithm.epochs,
        data.simulation.measurements.num_steps(),
        model.spatial_description.sensors.count(),
        model.spatial_description.voxels.count_states(),
        model.spatial_description.sensors.count_beats(),
        config.algorithm.batch_size,
        0,
        config.algorithm.optimizer,
    );
    results.model = Some(model);

    let mut batch_index = 0;
    run_epoch(&mut results, &mut batch_index, &data, &config.algorithm)?;

    Ok(results)
}

criterion_group! {name = benches;
config = Criterion::default().measurement_time(Duration::from_secs(30));
targets=run_benches}
criterion_main!(benches);
