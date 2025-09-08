use std::time::Duration;

use cardiotrust::core::{
    algorithm::run_epoch, config::Config, data::Data, model::Model, scenario::results::Results,
};
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

const VOXEL_SIZES: [f32; 3] = [2.0, 2.5, 5.0];
const LEARNING_RATE: f32 = 1e-3;

fn run_benches(c: &mut Criterion) {
    let mut group = c.benchmark_group("Run Epoch");
    epoch(&mut group);
    group.finish();
}

fn epoch(group: &mut criterion::BenchmarkGroup<criterion::measurement::WallTime>) {
    for voxel_size in VOXEL_SIZES.iter() {
        let config = setup_config(voxel_size);

        // setup inputs
        let (data, mut results) = setup_inputs(&config);

        // run bench
        let number_of_voxels = results
            .model
            .as_ref()
            .unwrap()
            .spatial_description
            .voxels
            .count();
        let mut batch_index = 0;
        group.throughput(criterion::Throughput::Elements(number_of_voxels as u64));
        group.bench_function(BenchmarkId::new("without_update", voxel_size), |b| {
            b.iter(|| run_epoch(&mut results, &mut batch_index, &data, &config.algorithm))
        });
    }
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

fn setup_inputs(config: &Config) -> (Data, Results) {
    let simulation_config = &config.simulation;
    let data =
        Data::from_simulation_config(simulation_config).expect("Model parameters to be valid.");
    let model = Model::from_model_config(
        &config.algorithm.model,
        simulation_config.sample_rate_hz,
        simulation_config.duration_s,
    )
    .unwrap();
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
    (data, results)
}

criterion_group! {name = epoch_benches;
config = Criterion::default().measurement_time(Duration::from_secs(10)).sample_size(20);
targets=run_benches}
criterion_main!(epoch_benches);
