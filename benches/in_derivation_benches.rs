use std::time::Duration;

use anyhow::Context;
use cardiotrust::core::{
    algorithm::{
        estimation::{calculate_residuals, prediction::calculate_system_prediction},
        refinement::derivation::{
            calculate_average_delays, calculate_derivatives_coefs_simple,
            calculate_derivatives_gains, calculate_mapped_residuals,
            calculate_maximum_regularization, calculate_smoothness_derivatives,
        },
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
    let mut group = c.benchmark_group("In Derivation");
    bench_residual_mapping(&mut group).expect("Benchmark should succeed");
    bench_maximum_regularization(&mut group).expect("Benchmark should succeed");
    bench_smoothness_derivatives(&mut group).expect("Benchmark should succeed");
    bench_average_delays(&mut group).expect("Benchmark should succeed");
    bench_gains(&mut group).expect("Benchmark should succeed");
    bench_coefs(&mut group).expect("Benchmark should succeed");
    group.finish();
}

fn bench_average_delays(
    group: &mut criterion::BenchmarkGroup<criterion::measurement::WallTime>,
) -> anyhow::Result<()> {
    for voxel_size in VOXEL_SIZES.iter() {
        let config = setup_config(voxel_size);

        // setup inputs
        let (_, model, mut results) =
            setup_inputs(&config).context("Failed to setup benchmark inputs")?;

        // run bench
        let number_of_voxels = model.spatial_description.voxels.count();
        group.throughput(criterion::Throughput::Elements(number_of_voxels as u64));
        group.bench_function(BenchmarkId::new("average_delays", voxel_size), |b| {
            b.iter(|| {
                calculate_average_delays(
                    &mut results.estimations.average_delays,
                    &model.functional_description.ap_params,
                )
                .expect("Calculation to succeed.");
            })
        });
    }
    Ok(())
}

fn bench_smoothness_derivatives(
    group: &mut criterion::BenchmarkGroup<criterion::measurement::WallTime>,
) -> anyhow::Result<()> {
    for voxel_size in VOXEL_SIZES.iter() {
        let config = setup_config(voxel_size);

        // setup inputs
        let (_, model, mut results) =
            setup_inputs(&config).context("Failed to setup benchmark inputs")?;

        // run bench
        let number_of_voxels = model.spatial_description.voxels.count();
        group.throughput(criterion::Throughput::Elements(number_of_voxels as u64));
        group.bench_function(
            BenchmarkId::new("smoothness_derivatives", voxel_size),
            |b| {
                b.iter(|| {
                    calculate_smoothness_derivatives(
                        &mut results.derivatives,
                        &results.estimations,
                        &model.functional_description,
                        &config.algorithm,
                    )
                    .expect("Calculation to succeed.");
                })
            },
        );
    }
    Ok(())
}

fn bench_residual_mapping(
    group: &mut criterion::BenchmarkGroup<criterion::measurement::WallTime>,
) -> anyhow::Result<()> {
    for voxel_size in VOXEL_SIZES.iter() {
        let config = setup_config(voxel_size);

        // setup inputs
        let (_, model, mut results) =
            setup_inputs(&config).context("Failed to setup benchmark inputs")?;

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
    Ok(())
}

fn bench_maximum_regularization(
    group: &mut criterion::BenchmarkGroup<criterion::measurement::WallTime>,
) -> anyhow::Result<()> {
    for voxel_size in VOXEL_SIZES.iter() {
        let config = setup_config(voxel_size);

        // setup inputs
        let (_, model, mut results) =
            setup_inputs(&config).context("Failed to setup benchmark inputs")?;

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
                        config.algorithm.maximum_regularization_threshold,
                    );
                })
            },
        );
    }
    Ok(())
}

fn bench_gains(
    group: &mut criterion::BenchmarkGroup<criterion::measurement::WallTime>,
) -> anyhow::Result<()> {
    for voxel_size in VOXEL_SIZES.iter() {
        let config = setup_config(voxel_size);

        // setup inputs
        let (_, model, mut results) =
            setup_inputs(&config).context("Failed to setup benchmark inputs")?;

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
            config.algorithm.maximum_regularization_threshold,
        );

        // run bench
        let number_of_voxels = model.spatial_description.voxels.count();
        group.throughput(criterion::Throughput::Elements(number_of_voxels as u64));
        group.bench_function(BenchmarkId::new("gains", voxel_size), |b| {
            b.iter(|| {
                calculate_derivatives_gains(
                    &mut results.derivatives.gains,
                    &results.estimations.ap_outputs_now,
                    &results.derivatives.maximum_regularization,
                    &results.derivatives.mapped_residuals,
                    &config.algorithm,
                    results.estimations.measurements.num_sensors(),
                );
            })
        });
    }
    Ok(())
}

fn bench_coefs(
    group: &mut criterion::BenchmarkGroup<criterion::measurement::WallTime>,
) -> anyhow::Result<()> {
    for voxel_size in VOXEL_SIZES.iter() {
        let config = setup_config(voxel_size);

        // setup inputs
        let (_, model, mut results) =
            setup_inputs(&config).context("Failed to setup benchmark inputs")?;

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
            config.algorithm.maximum_regularization_threshold,
        );

        // run bench
        let number_of_voxels = model.spatial_description.voxels.count();
        group.throughput(criterion::Throughput::Elements(number_of_voxels as u64));
        group.bench_function(BenchmarkId::new("coefs", voxel_size), |b| {
            b.iter(|| {
                calculate_derivatives_coefs_simple(
                    &mut results.derivatives,
                    &results.estimations,
                    &model.functional_description,
                    STEP,
                    &config.algorithm,
                )
                .expect("Calculation to succeed.");
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
    calculate_system_prediction(
        &mut results.estimations,
        &model.functional_description,
        BEAT,
        STEP,
    )?;
    calculate_residuals(&mut results.estimations, &data, BEAT, STEP);
    Ok((data, model, results))
}

criterion_group! {name = benches;
config = Criterion::default().measurement_time(Duration::from_secs(30));
targets=run_benches}
criterion_main!(benches);
