use std::time::Duration;

use anyhow::Context;
use cardiotrust::core::{
    algorithm::{
        estimation::{calculate_residuals, prediction::calculate_system_prediction},
        metrics,
        refinement::derivation::calculate_step_derivatives,
    },
    config::Config,
    data::Data,
    model::Model,
    scenario::results::Results,
};
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

const VOXEL_SIZES: [f32; 1] = [2.5];
const LEARNING_RATE: f32 = 1e-3;
const STEP: usize = 42;
const BEAT: usize = 0;

fn run_benches(c: &mut Criterion) {
    let mut group = c.benchmark_group("In Epoch");
    bench_resetting(&mut group).expect("Benchmark should succeed");
    bench_system_prediction(&mut group).expect("Benchmark should succeed");
    bench_residuals(&mut group).expect("Benchmark should succeed");
    bench_derivation(&mut group).expect("Benchmark should succeed");
    bench_metrics(&mut group).expect("Benchmark should succeed");
    bench_update_parameters(&mut group).expect("Benchmark should succeed");
    group.finish();
}

fn resetting(results: &mut Results) {
    results.estimations.reset();
    results.derivatives.reset();
}

fn bench_resetting(group: &mut criterion::BenchmarkGroup<criterion::measurement::WallTime>) -> anyhow::Result<()> {
    for voxel_size in VOXEL_SIZES.iter() {
        let config = setup_config(voxel_size);

        // setup inputs
        let (_, model, mut results) = setup_inputs(&config).context("Failed to setup benchmark inputs")?;

        // run bench
        let number_of_voxels = model.spatial_description.voxels.count();
        group.throughput(criterion::Throughput::Elements(number_of_voxels as u64));
        group.bench_function(BenchmarkId::new("resetting", voxel_size), |b| {
            b.iter(|| {
                resetting(&mut results);
            })
        });
    }
    Ok(())
}

fn bench_system_prediction(
    group: &mut criterion::BenchmarkGroup<criterion::measurement::WallTime>,
) -> anyhow::Result<()> {
    for voxel_size in VOXEL_SIZES.iter() {
        let config = setup_config(voxel_size);

        // setup inputs
        let (_, model, mut results) = setup_inputs(&config).context("Failed to setup benchmark inputs")?;

        // run bench
        let number_of_voxels = model.spatial_description.voxels.count();
        group.throughput(criterion::Throughput::Elements(number_of_voxels as u64));
        group.bench_function(BenchmarkId::new("system_prediction", voxel_size), |b| {
            b.iter(|| {
                calculate_system_prediction(
                    &mut results.estimations,
                    &model.functional_description,
                    BEAT,
                    STEP,
                )
            })
        });
    }
    Ok(())
}

fn bench_residuals(group: &mut criterion::BenchmarkGroup<criterion::measurement::WallTime>) -> anyhow::Result<()> {
    for voxel_size in VOXEL_SIZES.iter() {
        let config = setup_config(voxel_size);

        // setup inputs
        let (data, model, mut results) = setup_inputs(&config).context("Failed to setup benchmark inputs")?;

        // run bench
        let number_of_voxels = model.spatial_description.voxels.count();
        group.throughput(criterion::Throughput::Elements(number_of_voxels as u64));
        group.bench_function(BenchmarkId::new("residuals", voxel_size), |b| {
            b.iter(|| {
                calculate_residuals(&mut results.estimations, &data, BEAT, STEP);
            })
        });
    }
    Ok(())
}

fn bench_derivation(group: &mut criterion::BenchmarkGroup<criterion::measurement::WallTime>) -> anyhow::Result<()> {
    for voxel_size in VOXEL_SIZES.iter() {
        let config = setup_config(voxel_size);

        // setup inputs
        let (_, model, mut results) = setup_inputs(&config).context("Failed to setup benchmark inputs")?;

        // run bench
        let number_of_voxels = model.spatial_description.voxels.count();
        group.throughput(criterion::Throughput::Elements(number_of_voxels as u64));
        group.bench_function(BenchmarkId::new("derivation", voxel_size), |b| {
            b.iter(|| {
                calculate_step_derivatives(
                    &mut results.derivatives,
                    &results.estimations,
                    &model.functional_description,
                    &config.algorithm,
                    STEP,
                    BEAT,
                    results.estimations.measurements.num_sensors(),
                );
            })
        });
    }
    Ok(())
}

fn bench_metrics(group: &mut criterion::BenchmarkGroup<criterion::measurement::WallTime>) -> anyhow::Result<()> {
    for voxel_size in VOXEL_SIZES.iter() {
        let config = setup_config(voxel_size);

        // setup inputs
        let (_, model, mut results) = setup_inputs(&config).context("Failed to setup benchmark inputs")?;

        // run bench
        let number_of_voxels = model.spatial_description.voxels.count();
        group.throughput(criterion::Throughput::Elements(number_of_voxels as u64));
        group.bench_function(BenchmarkId::new("metrics", voxel_size), |b| {
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

fn bench_update_parameters(
    group: &mut criterion::BenchmarkGroup<criterion::measurement::WallTime>,
) -> anyhow::Result<()> {
    for voxel_size in VOXEL_SIZES.iter() {
        let config = setup_config(voxel_size);

        // setup inputs
        let (_, mut model, mut results) = setup_inputs(&config).context("Failed to setup benchmark inputs")?;

        // run bench
        let number_of_voxels = model.spatial_description.voxels.count();
        group.throughput(criterion::Throughput::Elements(number_of_voxels as u64));
        group.bench_function(BenchmarkId::new("update_parameters", voxel_size), |b| {
            b.iter(|| {
                model.functional_description.ap_params.update(
                    &mut results.derivatives,
                    &config.algorithm,
                    results.estimations.system_states.num_steps(),
                    model.spatial_description.sensors.count_beats(),
                );
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

fn setup_inputs(config: &Config) -> anyhow::Result<(Data, Model, Results)> {
    let simulation_config = &config.simulation;
    let data = Data::from_simulation_config(simulation_config)
        .context("Failed to create simulation data from config")?;
    let model = Model::from_model_config(
        &config.algorithm.model,
        simulation_config.sample_rate_hz,
        simulation_config.duration_s,
    )
    .context("Failed to create model from config")?;
    let results = Results::new(
        config.algorithm.epochs,
        data.simulation.measurements.num_steps(),
        model.spatial_description.sensors.count(),
        model.spatial_description.voxels.count_states(),
        model.spatial_description.sensors.count_beats(),
        config.algorithm.batch_size,
        0,
        config.algorithm.optimizer,
    );
    Ok((data, model, results))
}

criterion_group! {name = benches;
config = Criterion::default().measurement_time(Duration::from_secs(30)).sample_size(20);
targets=run_benches}
criterion_main!(benches);
