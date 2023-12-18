use rusty_cde::core::algorithm::estimation::EstimationsFlat;
use rusty_cde::core::algorithm::run_epoch;
use rusty_cde::core::scenario::results::Results;
#[cfg(not(target_env = "msvc"))]
use tikv_jemallocator::Jemalloc;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

use std::time::Duration;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use rusty_cde::core::algorithm::estimation::prediction::{
    add_control_function, calculate_system_prediction_flat, innovate_system_states_flat_v1,
    predict_measurements,
};
use rusty_cde::core::{config::Config, data::Data, model::Model};

const VOXEL_SIZES: [f32; 1] = [2.5];

fn system_prediction_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("System Prediction");
    for voxel_size in VOXEL_SIZES.iter() {
        let samplerate_hz = 2000.0 * 2.5 / voxel_size;
        // flat
        let mut config_flat = Config::default();
        config_flat.simulation.as_mut().unwrap().model.voxel_size_mm = *voxel_size;
        config_flat.simulation.as_mut().unwrap().sample_rate_hz = samplerate_hz;
        config_flat.algorithm.model.voxel_size_mm = *voxel_size;
        let simulation_config_flat = config_flat.simulation.as_ref().unwrap();
        let data_flat = Data::from_simulation_config(simulation_config_flat)
            .expect("Model parameters to be valid.");
        let model_flat = Model::from_model_config(
            &config_flat.algorithm.model,
            simulation_config_flat.sample_rate_hz,
            simulation_config_flat.duration_s,
        )
        .unwrap();
        let mut estimations_flat = EstimationsFlat::empty(
            model_flat.spatial_description.voxels.count_states(),
            model_flat.spatial_description.sensors.count(),
            data_flat.get_measurements().values.shape()[0],
        );
        let time_index = 200;
        let number_of_voxels = model_flat.spatial_description.voxels.count();
        group.throughput(criterion::Throughput::Elements(number_of_voxels as u64));
        group.bench_function(
            BenchmarkId::new("calculate_system_perdiction_flat", voxel_size),
            |b| {
                b.iter(|| {
                    calculate_system_prediction_flat(
                        &mut estimations_flat.ap_outputs,
                        &mut estimations_flat.system_states,
                        &mut estimations_flat.measurements,
                        &model_flat.functional_description,
                        time_index,
                    )
                })
            },
        );
        group.bench_function(
            BenchmarkId::new("innovate_system_states_flat_v1", voxel_size),
            |b| {
                b.iter(|| {
                    innovate_system_states_flat_v1(
                        &mut estimations_flat.ap_outputs,
                        &model_flat.functional_description.ap_params_flat,
                        time_index,
                        &mut estimations_flat.system_states,
                    )
                })
            },
        );
        group.bench_function(BenchmarkId::new("add_control_function", voxel_size), |b| {
            b.iter(|| {
                add_control_function(
                    &model_flat.functional_description,
                    time_index,
                    &mut estimations_flat.system_states,
                )
            })
        });
        group.bench_function(BenchmarkId::new("predict_measurements", voxel_size), |b| {
            b.iter(|| {
                predict_measurements(
                    &mut estimations_flat.measurements,
                    time_index,
                    &model_flat.functional_description.measurement_matrix,
                    &mut estimations_flat.system_states,
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
        let mut config_flat = Config::default();
        config_flat.simulation.as_mut().unwrap().model.voxel_size_mm = *voxel_size;
        config_flat.simulation.as_mut().unwrap().sample_rate_hz = samplerate_hz;
        config_flat.algorithm.model.voxel_size_mm = *voxel_size;
        let simulation_config_flat = config_flat.simulation.as_ref().unwrap();
        let data_flat = Data::from_simulation_config(simulation_config_flat)
            .expect("Model parameters to be valid.");
        let model_flat = Model::from_model_config(
            &config_flat.algorithm.model,
            simulation_config_flat.sample_rate_hz,
            simulation_config_flat.duration_s,
        )
        .unwrap();
        let mut estimations_flat = EstimationsFlat::empty(
            model_flat.spatial_description.voxels.count_states(),
            model_flat.spatial_description.sensors.count(),
            data_flat.get_measurements().values.shape()[0],
        );
        let number_of_voxels = model_flat.spatial_description.voxels.count();
        group.throughput(criterion::Throughput::Elements(number_of_voxels as u64));
        group.bench_function(
            BenchmarkId::new("innovate_system_states_flat_v1", voxel_size),
            |b| {
                b.iter(|| {
                    for time_index in 0..estimations_flat.measurements.values.shape()[0] {
                        innovate_system_states_flat_v1(
                            &mut estimations_flat.ap_outputs,
                            &model_flat.functional_description.ap_params_flat,
                            time_index,
                            &mut estimations_flat.system_states,
                        )
                    }
                })
            },
        );
        group.bench_function(BenchmarkId::new("add_control_function", voxel_size), |b| {
            b.iter(|| {
                for time_index in 0..estimations_flat.measurements.values.shape()[0] {
                    add_control_function(
                        &model_flat.functional_description,
                        time_index,
                        &mut estimations_flat.system_states,
                    )
                }
            })
        });
        group.bench_function(BenchmarkId::new("predict_measurements", voxel_size), |b| {
            b.iter(|| {
                for time_index in 0..estimations_flat.measurements.values.shape()[0] {
                    predict_measurements(
                        &mut estimations_flat.measurements,
                        time_index,
                        &model_flat.functional_description.measurement_matrix,
                        &mut estimations_flat.system_states,
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
        // normal
        let mut config_normal = Config::default();
        config_normal
            .simulation
            .as_mut()
            .unwrap()
            .model
            .voxel_size_mm = *voxel_size;
        config_normal.simulation.as_mut().unwrap().sample_rate_hz = samplerate_hz;
        config_normal.algorithm.model.voxel_size_mm = *voxel_size;
        let simulation_config_normal = config_normal.simulation.as_ref().unwrap();
        let data_normal = Data::from_simulation_config(simulation_config_normal)
            .expect("Model parameters to be valid.");
        let mut model_normal = Model::from_model_config(
            &config_normal.algorithm.model,
            simulation_config_normal.sample_rate_hz,
            simulation_config_normal.duration_s,
        )
        .unwrap();
        let mut results_normal = Results::new(
            config_normal.algorithm.epochs,
            data_normal.get_measurements().values.shape()[0],
            model_normal.spatial_description.sensors.count(),
            model_normal.spatial_description.voxels.count_states(),
        );
        let mut config_flat = Config::default();
        config_flat.simulation.as_mut().unwrap().model.voxel_size_mm = *voxel_size;
        config_flat.simulation.as_mut().unwrap().sample_rate_hz = samplerate_hz;
        config_flat.algorithm.model.voxel_size_mm = *voxel_size;
        let simulation_config_flat = config_flat.simulation.as_ref().unwrap();
        let data_flat = Data::from_simulation_config(simulation_config_flat)
            .expect("Model parameters to be valid.");
        let mut model_flat = Model::from_model_config(
            &config_flat.algorithm.model,
            simulation_config_flat.sample_rate_hz,
            simulation_config_flat.duration_s,
        )
        .unwrap();
        let mut results_flat = Results::new(
            config_flat.algorithm.epochs,
            data_flat.get_measurements().values.shape()[0],
            model_flat.spatial_description.sensors.count(),
            model_flat.spatial_description.voxels.count_states(),
        );
        // flat
        let number_of_voxels = model_normal.spatial_description.voxels.count();
        group.throughput(criterion::Throughput::Elements(number_of_voxels as u64));
        group.bench_function(BenchmarkId::new("run_epoch_normal", voxel_size), |b| {
            b.iter(|| {
                run_epoch(
                    &mut model_normal.functional_description,
                    &mut results_normal,
                    &data_normal,
                    &config_normal.algorithm,
                    0,
                )
            })
        });
        group.bench_function(BenchmarkId::new("run_epoch_flat", voxel_size), |b| {
            b.iter(|| {
                run_epoch(
                    &mut model_flat.functional_description,
                    &mut results_flat,
                    &data_flat,
                    &config_flat.algorithm,
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
