use std::time::Duration;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rusty_cde::core::algorithm::estimation::{
    calculate_system_prediction, par_calculate_system_prediction,
};
use rusty_cde::core::{
    algorithm::estimation::Estimations, config::Config, data::Data, model::Model,
};

fn criterion_benchmark(c: &mut Criterion) {
    let config = Config::default();
    let simulation_config = config.simulation.as_ref().unwrap();
    let data = Data::from_simulation_config(simulation_config);
    let model = Model::from_model_config(
        &config.algorithm.model,
        simulation_config.sample_rate_hz,
        simulation_config.duration_s,
    )
    .unwrap();
    let mut estimations = Estimations::empty(
        model.spatial_description.voxels.count_states(),
        model.spatial_description.sensors.count(),
        data.get_measurements().values.shape()[0],
    );
    let time_index = 200;
    let mut group = c.benchmark_group("System Prediction");
    group.bench_function("Normal", |b| {
        b.iter(|| {
            calculate_system_prediction(
                black_box(&mut estimations.ap_outputs),
                black_box(&mut estimations.system_states),
                black_box(&mut estimations.measurements),
                black_box(&model.functional_description),
                black_box(time_index),
            )
        })
    });
    group.bench_function("Threaded", |b| {
        b.iter(|| {
            par_calculate_system_prediction(
                black_box(&mut estimations.ap_outputs),
                black_box(&mut estimations.system_states),
                black_box(&mut estimations.measurements),
                black_box(&model.functional_description),
                black_box(time_index),
            )
        })
    });
    group.finish();
}

criterion_group! {name = benches;
config = Criterion::default().measurement_time(Duration::from_secs(10));
targets=criterion_benchmark}
criterion_main!(benches);
