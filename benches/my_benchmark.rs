#[cfg(not(target_env = "msvc"))]
use tikv_jemallocator::Jemalloc;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

use std::time::Duration;

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use rusty_cde::core::algorithm::estimation::{
    add_control_function, calculate_system_prediction, innovate_system_states, predict_measurements,
};
use rusty_cde::core::{
    algorithm::estimation::Estimations, config::Config, data::Data, model::Model,
};

const VOXEL_SIZES: [f32; 1] = [2.5];

fn system_prediction(c: &mut Criterion) {
    let mut group = c.benchmark_group("System Prediction");
    for voxel_size in VOXEL_SIZES.iter() {
        let samplerate_hz = 2000.0 * 2.5 / voxel_size;
        let mut config = Config::default();
        config.simulation.as_mut().unwrap().model.voxel_size_mm = *voxel_size;
        config.simulation.as_mut().unwrap().sample_rate_hz = samplerate_hz;
        config.algorithm.model.voxel_size_mm = *voxel_size;
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
        let number_of_voxels = model.spatial_description.voxels.count();
        group.throughput(criterion::Throughput::Elements(number_of_voxels as u64));
        group.bench_function(
            BenchmarkId::new("calculate_system_perdiction", voxel_size),
            |b| {
                b.iter(|| {
                    calculate_system_prediction(
                        black_box(&mut estimations.ap_outputs),
                        black_box(&mut estimations.system_states),
                        black_box(&mut estimations.measurements),
                        black_box(&model.functional_description),
                        black_box(time_index),
                    )
                })
            },
        );
        group.bench_function(
            BenchmarkId::new("innovate_system_states", voxel_size),
            |b| {
                b.iter(|| {
                    innovate_system_states(
                        black_box(&mut estimations.ap_outputs),
                        black_box(&model.functional_description),
                        black_box(time_index),
                        black_box(&mut estimations.system_states),
                    )
                })
            },
        );
        group.bench_function(BenchmarkId::new("add_control_function", voxel_size), |b| {
            b.iter(|| {
                add_control_function(
                    black_box(&model.functional_description),
                    black_box(time_index),
                    black_box(&mut estimations.system_states),
                )
            })
        });
        group.bench_function(BenchmarkId::new("predict_measurements", voxel_size), |b| {
            b.iter(|| {
                predict_measurements(
                    black_box(&mut estimations.measurements),
                    black_box(time_index),
                    black_box(&model.functional_description.measurement_matrix),
                    black_box(&mut estimations.system_states),
                )
            })
        });
    }
    group.finish();
}

criterion_group! {name = benches;
config = Criterion::default().measurement_time(Duration::from_secs(10));
targets=system_prediction}
criterion_main!(benches);
