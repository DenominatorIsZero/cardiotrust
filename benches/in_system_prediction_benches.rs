use cardiotrust::core::{
    algorithm::estimation::prediction::{
        add_control_function, innovate_system_states_v1, predict_measurements,
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
const STEP: usize = 42;
const BEAT: usize = 0;

fn run_benches(c: &mut Criterion) {
    let mut group = c.benchmark_group("In System Prediction");
    bench_innovate(&mut group);
    bench_control_function(&mut group);
    bench_measurements(&mut group);
    group.finish();
}

fn bench_innovate(group: &mut criterion::BenchmarkGroup<criterion::measurement::WallTime>) {
    for voxel_size in VOXEL_SIZES.iter() {
        let config = setup_config(voxel_size);

        // setup inputs
        let (_, model, mut results) = setup_inputs(&config);

        // run bench
        let number_of_voxels = model.spatial_description.voxels.count();
        group.throughput(criterion::Throughput::Elements(number_of_voxels as u64));
        group.bench_function(
            BenchmarkId::new("innovate_system_states", voxel_size),
            |b| {
                b.iter(|| {
                    innovate_system_states_v1(
                        &mut results.estimations.ap_outputs,
                        &model.functional_description.ap_params,
                        STEP,
                        &mut results.estimations.system_states,
                    );
                })
            },
        );
    }
}

fn bench_control_function(group: &mut criterion::BenchmarkGroup<criterion::measurement::WallTime>) {
    for voxel_size in VOXEL_SIZES.iter() {
        let config = setup_config(voxel_size);

        // setup inputs
        let (_, model, mut results) = setup_inputs(&config);

        // run bench
        let number_of_voxels = model.spatial_description.voxels.count();
        let mut system_states = results.estimations.system_states.at_step_mut(STEP);
        let control_function_value = model.functional_description.control_function_values[STEP];
        let control_matrix = &model.functional_description.control_matrix;
        group.throughput(criterion::Throughput::Elements(number_of_voxels as u64));
        group.bench_function(BenchmarkId::new("add_control_funciton", voxel_size), |b| {
            b.iter(|| {
                add_control_function(&mut system_states, control_function_value, control_matrix);
            })
        });
    }
}

fn bench_measurements(group: &mut criterion::BenchmarkGroup<criterion::measurement::WallTime>) {
    for voxel_size in VOXEL_SIZES.iter() {
        let config = setup_config(voxel_size);

        // setup inputs
        let (_, model, mut results) = setup_inputs(&config);

        // run bench
        let number_of_voxels = model.spatial_description.voxels.count();
        let mut measurements_at_beat = results.estimations.measurements.at_beat_mut(BEAT);
        let mut measurements = measurements_at_beat.at_step_mut(STEP);
        let measurement_matrix = model
            .functional_description
            .measurement_matrix
            .at_beat(BEAT);
        let system_states = results.estimations.system_states.at_step_mut(STEP);
        group.throughput(criterion::Throughput::Elements(number_of_voxels as u64));
        group.bench_function(BenchmarkId::new("predict_measurements", voxel_size), |b| {
            b.iter(|| predict_measurements(&mut measurements, &measurement_matrix, &system_states));
        });
    }
}

fn setup_config(voxel_size: &f32) -> Config {
    let samplerate_hz = 2000.0 * 2.5 / voxel_size;
    let mut config = Config::default();
    config.simulation.model.common.voxel_size_mm = *voxel_size;
    config.simulation.sample_rate_hz = samplerate_hz;
    config.algorithm.model.common.voxel_size_mm = *voxel_size;
    config.algorithm.model.common.apply_system_update = true;
    config.algorithm.update_kalman_gain = false;
    config.algorithm.learning_rate = LEARNING_RATE;
    config.algorithm.freeze_delays = false;
    config.algorithm.freeze_gains = false;
    config.algorithm.batch_size = 0;
    config
}

fn setup_inputs(config: &Config) -> (Data, Model, Results) {
    let simulation_config = &config.simulation;
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
        data.simulation.measurements.num_steps(),
        model.spatial_description.sensors.count(),
        model.spatial_description.voxels.count_states(),
        model.spatial_description.sensors.count_beats(),
        config.algorithm.optimizer,
    );
    (data, model, results)
}

criterion_group! {name = benches;
config = Criterion::default().measurement_time(Duration::from_secs(30));
targets=run_benches}
criterion_main!(benches);
