use cardiotrust::core::{
    algorithm::{
        estimation::{calculate_residuals, prediction::calculate_system_prediction},
        refinement::derivation::{
            calculate_derivatives_coefs, calculate_derivatives_gains, calculate_mapped_residuals,
            calculate_maximum_regularization,
        },
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
    let mut group = c.benchmark_group("In Derivation");
    bench_residual_mapping(&mut group);
    bench_maximum_regularization(&mut group);
    bench_gains(&mut group);
    bench_coefs(&mut group);
    group.finish();
}

fn bench_residual_mapping(group: &mut criterion::BenchmarkGroup<criterion::measurement::WallTime>) {
    for voxel_size in VOXEL_SIZES.iter() {
        let config = setup_config(voxel_size);

        // setup inputs
        let (_, model, mut results) = setup_inputs(&config);

        // run bench
        let number_of_voxels = model.spatial_description.voxels.count();
        group.throughput(criterion::Throughput::Elements(number_of_voxels as u64));
        group.bench_function(BenchmarkId::new("residual_mapping", voxel_size), |b| {
            b.iter(|| {
                calculate_mapped_residuals(
                    &mut results.derivatives.mapped_residuals,
                    &results.estimations.residuals,
                    &model
                        .functional_description
                        .measurement_matrix
                        .at_beat(BEAT),
                );
            })
        });
    }
}

fn bench_maximum_regularization(
    group: &mut criterion::BenchmarkGroup<criterion::measurement::WallTime>,
) {
    for voxel_size in VOXEL_SIZES.iter() {
        let config = setup_config(voxel_size);

        // setup inputs
        let (_, model, mut results) = setup_inputs(&config);

        // prepare inputs
        calculate_mapped_residuals(
            &mut results.derivatives.mapped_residuals,
            &results.estimations.residuals,
            &model
                .functional_description
                .measurement_matrix
                .at_beat(BEAT),
        );

        // run bench
        let number_of_voxels = model.spatial_description.voxels.count();
        group.throughput(criterion::Throughput::Elements(number_of_voxels as u64));
        group.bench_function(
            BenchmarkId::new("maximum_regularization", voxel_size),
            |b| {
                b.iter(|| {
                    calculate_maximum_regularization(
                        &mut results.derivatives.maximum_regularization,
                        &mut results.derivatives.maximum_regularization_sum,
                        &results.estimations.system_states.at_step(STEP),
                        config.algorithm.regularization_threshold,
                    );
                })
            },
        );
    }
}

fn bench_gains(group: &mut criterion::BenchmarkGroup<criterion::measurement::WallTime>) {
    for voxel_size in VOXEL_SIZES.iter() {
        let config = setup_config(voxel_size);

        // setup inputs
        let (_, model, mut results) = setup_inputs(&config);

        // prepare inputs
        calculate_mapped_residuals(
            &mut results.derivatives.mapped_residuals,
            &results.estimations.residuals,
            &model
                .functional_description
                .measurement_matrix
                .at_beat(BEAT),
        );
        calculate_maximum_regularization(
            &mut results.derivatives.maximum_regularization,
            &mut results.derivatives.maximum_regularization_sum,
            &results.estimations.system_states.at_step(STEP),
            config.algorithm.regularization_threshold,
        );

        // run bench
        let number_of_voxels = model.spatial_description.voxels.count();
        group.throughput(criterion::Throughput::Elements(number_of_voxels as u64));
        group.bench_function(BenchmarkId::new("gains", voxel_size), |b| {
            b.iter(|| {
                calculate_derivatives_gains(
                    &mut results.derivatives.gains,
                    &results.estimations.ap_outputs,
                    &results.derivatives.maximum_regularization,
                    &results.derivatives.mapped_residuals,
                    config.algorithm.regularization_strength,
                    results.estimations.measurements.num_sensors(),
                );
            })
        });
    }
}

fn bench_coefs(group: &mut criterion::BenchmarkGroup<criterion::measurement::WallTime>) {
    for voxel_size in VOXEL_SIZES.iter() {
        let config = setup_config(voxel_size);

        // setup inputs
        let (_, model, mut results) = setup_inputs(&config);

        // prepare inputs
        calculate_mapped_residuals(
            &mut results.derivatives.mapped_residuals,
            &results.estimations.residuals,
            &model
                .functional_description
                .measurement_matrix
                .at_beat(BEAT),
        );
        calculate_maximum_regularization(
            &mut results.derivatives.maximum_regularization,
            &mut results.derivatives.maximum_regularization_sum,
            &results.estimations.system_states.at_step(STEP),
            config.algorithm.regularization_threshold,
        );

        // run bench
        let number_of_voxels = model.spatial_description.voxels.count();
        group.throughput(criterion::Throughput::Elements(number_of_voxels as u64));
        group.bench_function(BenchmarkId::new("coefs", voxel_size), |b| {
            b.iter(|| {
                calculate_derivatives_coefs(
                    &mut results.derivatives.coefs,
                    &mut results.derivatives.coefs_iir,
                    &mut results.derivatives.coefs_fir,
                    &results.estimations.ap_outputs,
                    &results.estimations.system_states,
                    &model.functional_description.ap_params,
                    &results.derivatives.mapped_residuals,
                    STEP,
                    results.estimations.measurements.num_sensors(),
                );
            })
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
    let mut results = Results::new(
        config.algorithm.epochs,
        data.simulation.measurements.num_steps(),
        model.spatial_description.sensors.count(),
        model.spatial_description.voxels.count_states(),
        model.spatial_description.sensors.count_beats(),
        config.algorithm.batch_size,
        config.algorithm.optimizer,
    );
    let estimations = &mut results.estimations;
    let measurement_matrix = model
        .functional_description
        .measurement_matrix
        .at_beat(BEAT);
    calculate_system_prediction(
        &mut estimations.ap_outputs,
        &mut estimations.system_states,
        &mut estimations.measurements.at_beat_mut(BEAT).at_step_mut(STEP),
        &model.functional_description.ap_params,
        &measurement_matrix,
        model.functional_description.control_function_values[STEP],
        &model.functional_description.control_matrix,
        STEP,
    );
    calculate_residuals(
        &mut estimations.residuals,
        &estimations.measurements.at_beat_mut(BEAT).at_step_mut(STEP),
        &data.simulation.measurements.at_beat(BEAT).at_step(STEP),
    );
    (data, model, results)
}

criterion_group! {name = benches;
config = Criterion::default().measurement_time(Duration::from_secs(30));
targets=run_benches}
criterion_main!(benches);
