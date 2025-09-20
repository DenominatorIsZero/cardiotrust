use std::time::Duration;

use anyhow::Context;
use cardiotrust::core::{
    algorithm::{
        refinement::update::{roll_delays, update_delays_sgd, update_gains_sgd},
        run_epoch,
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
    let mut group = c.benchmark_group("In Update");
    bench_gains(&mut group).expect("Benchmark execution should succeed");
    bench_delays(&mut group).expect("Benchmark execution should succeed");
    group.finish();
}

fn bench_gains(group: &mut criterion::BenchmarkGroup<criterion::measurement::WallTime>) -> anyhow::Result<()> {
    for voxel_size in VOXEL_SIZES.iter() {
        let config = setup_config(voxel_size);

        // setup inputs
        let (_, mut results) = setup_inputs(&config)?;

        // run bench
        let number_of_voxels = results
            .model
            .as_ref()
            .unwrap()
            .spatial_description
            .voxels
            .count();
        group.throughput(criterion::Throughput::Elements(number_of_voxels as u64));
        group.bench_function(BenchmarkId::new("gains", voxel_size), |b| {
            b.iter(|| {
                update_gains_sgd(
                    &mut results
                        .model
                        .as_mut()
                        .unwrap()
                        .functional_description
                        .ap_params
                        .gains,
                    &results.derivatives.gains,
                    config.algorithm.learning_rate,
                    2000,
                );
            })
        });
    }
    Ok(())
}

fn bench_delays(group: &mut criterion::BenchmarkGroup<criterion::measurement::WallTime>) -> anyhow::Result<()> {
    for voxel_size in VOXEL_SIZES.iter() {
        let config = setup_config(voxel_size);

        // setup inputs
        let (_, mut results) = setup_inputs(&config)?;

        // run bench
        let number_of_voxels = results
            .model
            .as_ref()
            .unwrap()
            .spatial_description
            .voxels
            .count();
        group.throughput(criterion::Throughput::Elements(number_of_voxels as u64));
        let functional_description = &mut results.model.as_mut().context("Model should be available for mutation")?.functional_description;
        group.bench_function(BenchmarkId::new("delays", voxel_size), |b| {
            b.iter(|| {
                update_delays_sgd(
                    &mut functional_description.ap_params.coefs,
                    &results.derivatives.coefs,
                    config.algorithm.learning_rate,
                    2000,
                    1e-3,
                );
                roll_delays(
                    &mut functional_description.ap_params.coefs,
                    &mut functional_description.ap_params.delays,
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

fn setup_inputs(config: &Config) -> anyhow::Result<(Data, Results)> {
    let simulation_config = &config.simulation;
    let data =
        Data::from_simulation_config(simulation_config)?;
    let model = Model::from_model_config(
        &config.algorithm.model,
        simulation_config.sample_rate_hz,
        simulation_config.duration_s,
    )?;
    let mut results = Results::new(
        config.algorithm.epochs,
        data.simulation.measurements.num_steps(),
        model.spatial_description.sensors.count(),
        model.spatial_description.voxels.count_states(),
        model.spatial_description.sensors.count_beats(),
        0,
        config.algorithm.batch_size,
        config.algorithm.optimizer,
    );
    results.model = Some(model);

    let mut batch_index = 0;
    run_epoch(&mut results, &mut batch_index, &data, &config.algorithm)?;

    Ok((data, results))
}

criterion_group! {name = benches;
config = Criterion::default().measurement_time(Duration::from_secs(30));
targets=run_benches}
criterion_main!(benches);
