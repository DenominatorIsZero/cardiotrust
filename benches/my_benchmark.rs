use rusty_cde::core::algorithm::run_epoch;
use rusty_cde::core::scenario::results::Results;
#[cfg(not(target_env = "msvc"))]
use tikv_jemallocator::Jemalloc;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

use std::time::Duration;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use rusty_cde::core::algorithm::estimation::{
    add_control_function, calculate_system_prediction, innovate_system_states, predict_measurements,
};
use rusty_cde::core::{
    algorithm::estimation::Estimations, config::Config, data::Data, model::Model,
};

const VOXEL_SIZES: [f32; 1] = [2.5];

fn system_prediction_bench(c: &mut Criterion) {
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
                        &mut estimations.ap_outputs,
                        &mut estimations.system_states,
                        &mut estimations.measurements,
                        &model.functional_description,
                        time_index,
                    )
                })
            },
        );
        group.bench_function(
            BenchmarkId::new("innovate_system_states", voxel_size),
            |b| {
                b.iter(|| {
                    innovate_system_states(
                        &mut estimations.ap_outputs,
                        &model.functional_description,
                        time_index,
                        &mut estimations.system_states,
                    )
                })
            },
        );
        group.bench_function(BenchmarkId::new("add_control_function", voxel_size), |b| {
            b.iter(|| {
                add_control_function(
                    &model.functional_description,
                    time_index,
                    &mut estimations.system_states,
                )
            })
        });
        group.bench_function(BenchmarkId::new("predict_measurements", voxel_size), |b| {
            b.iter(|| {
                predict_measurements(
                    &mut estimations.measurements,
                    time_index,
                    &model.functional_description.measurement_matrix,
                    &mut estimations.system_states,
                )
            })
        });
    }
    group.finish();
}

fn system_prediction_epoch_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("System Prediction Epoch");
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
        let number_of_voxels = model.spatial_description.voxels.count();
        group.throughput(criterion::Throughput::Elements(number_of_voxels as u64));
        group.bench_function(
            BenchmarkId::new("calculate_system_perdiction", voxel_size),
            |b| {
                b.iter(|| {
                    for time_index in 0..estimations.measurements.values.shape()[0] {
                        calculate_system_prediction(
                            &mut estimations.ap_outputs,
                            &mut estimations.system_states,
                            &mut estimations.measurements,
                            &model.functional_description,
                            time_index,
                        )
                    }
                })
            },
        );
        group.bench_function(
            BenchmarkId::new("innovate_system_states", voxel_size),
            |b| {
                b.iter(|| {
                    for time_index in 0..estimations.measurements.values.shape()[0] {
                        innovate_system_states(
                            &mut estimations.ap_outputs,
                            &model.functional_description,
                            time_index,
                            &mut estimations.system_states,
                        )
                    }
                })
            },
        );
        group.bench_function(BenchmarkId::new("add_control_function", voxel_size), |b| {
            b.iter(|| {
                for time_index in 0..estimations.measurements.values.shape()[0] {
                    add_control_function(
                        &model.functional_description,
                        time_index,
                        &mut estimations.system_states,
                    )
                }
            })
        });
        group.bench_function(BenchmarkId::new("predict_measurements", voxel_size), |b| {
            b.iter(|| {
                for time_index in 0..estimations.measurements.values.shape()[0] {
                    predict_measurements(
                        &mut estimations.measurements,
                        time_index,
                        &model.functional_description.measurement_matrix,
                        &mut estimations.system_states,
                    )
                }
            })
        });
    }
    group.finish();
}

fn run_epoch_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("Run Epoch");
    for voxel_size in VOXEL_SIZES.iter() {
        let samplerate_hz = 2000.0 * 2.5 / voxel_size;
        let mut config = Config::default();
        config.simulation.as_mut().unwrap().model.voxel_size_mm = *voxel_size;
        config.simulation.as_mut().unwrap().sample_rate_hz = samplerate_hz;
        config.algorithm.model.voxel_size_mm = *voxel_size;
        let simulation_config = config.simulation.as_ref().unwrap();
        let data = Data::from_simulation_config(simulation_config);
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
        let number_of_voxels = model.spatial_description.voxels.count();
        group.throughput(criterion::Throughput::Elements(number_of_voxels as u64));
        group.bench_function(BenchmarkId::new("run_epoch", voxel_size), |b| {
            b.iter(|| {
                run_epoch(
                    &mut model.functional_description,
                    &mut results,
                    &data,
                    &config.algorithm,
                    0,
                )
            })
        });
    }
    group.finish();
}

criterion_group! {name = sample_benches;
config = Criterion::default().measurement_time(Duration::from_secs(10));
targets=system_prediction_bench}
criterion_group! {name = epoch_benches;
config = Criterion::default().measurement_time(Duration::from_secs(10)).sample_size(10);
targets=system_prediction_epoch_bench, run_epoch_bench}
criterion_main!(sample_benches, epoch_benches);
