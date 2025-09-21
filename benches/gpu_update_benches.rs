use std::time::Duration;

use anyhow::Context;
use cardiotrust::core::{
    algorithm::{
        estimation::prediction::calculate_system_prediction,
        gpu::{
            derivation::DerivationKernel, prediction::PredictionKernel, update::UpdateKernel, GPU,
        },
        refinement::{
            derivation::calculate_step_derivatives,
            update::{roll_delays, update_delays_sgd, update_gains_sgd},
        },
    },
    config::Config,
    data::Data,
    model::Model,
    scenario::results::{Results, ResultsGPU},
};
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

const VOXEL_SIZES: [f32; 4] = [1.0, 2.0, 2.5, 5.0];

fn run_benches(c: &mut Criterion) {
    let mut group = c.benchmark_group("GPU Update");
    prectiction_benches(&mut group).expect("Benchmark execution should succeed");
    group.finish();
}

fn prectiction_benches(group: &mut criterion::BenchmarkGroup<criterion::measurement::WallTime>) -> anyhow::Result<()> {
    for voxel_size in VOXEL_SIZES.iter() {
        let config = setup_config(voxel_size);
        let (
            data,
            mut results,
            _gpu,
            results_gpu,
            prediction_kernel,
            derivation_kernel,
            update_kernel,
        ) = setup_inputs(&config)?;

        let number_of_voxels = results
            .model
            .as_ref()
            .context("Model should be available in benchmark")?
            .spatial_description
            .voxels
            .count();
        group.throughput(criterion::Throughput::Elements(number_of_voxels as u64));
        for step in 0..data.simulation.measurements.num_steps() {
            calculate_system_prediction(
                &mut results.estimations,
                &results.model.as_ref().context("Model should be available for prediction")?.functional_description,
                0,
                step,
            );
            calculate_step_derivatives(
                &mut results.derivatives,
                &results.estimations,
                &results.model.as_ref().context("Model should be available for prediction")?.functional_description,
                &config.algorithm,
                step,
                0,
                results.estimations.measurements.num_sensors(),
            );
        }
        let batch_size = results.estimations.measurements.num_steps();
        group.bench_function(BenchmarkId::new("cpu", voxel_size), |b| {
            b.iter(|| {
                update_gains_sgd(
                    &mut results
                        .model
                        .as_mut()
                        .expect("Model should be available for parameter updates")
                        .functional_description
                        .ap_params
                        .gains,
                    &results.derivatives.gains,
                    config.algorithm.learning_rate,
                    batch_size,
                );
                update_delays_sgd(
                    &mut results
                        .model
                        .as_mut()
                        .expect("Model should be available for parameter updates")
                        .functional_description
                        .ap_params
                        .coefs,
                    &results.derivatives.coefs,
                    config.algorithm.learning_rate,
                    batch_size,
                    0.0f32,
                );
                let model = results.model.as_mut().expect("Model should be available for mutation");
                roll_delays(
                    &mut model.functional_description.ap_params.coefs,
                    &mut model.functional_description.ap_params.delays,
                );
            })
        });
        for step in 0..data.simulation.measurements.num_steps() {
            results_gpu
                .estimations
                .step
                .write([step as i32].as_slice())
                .enq()
                .context("Failed to enqueue GPU operation in benchmark setup")?;
            prediction_kernel.execute();
            derivation_kernel.execute();
        }
        group.bench_function(BenchmarkId::new("gpu", voxel_size), |b| {
            b.iter(|| {
                update_kernel.execute();
            })
        });
    }
    Ok(())
}

fn setup_config(voxel_size: &f32) -> Config {
    let samplerate_hz = 2000.0 * 2.5 / voxel_size;
    let mut config = Config::default();
    config.simulation.model.common.voxel_size_mm = *voxel_size;
    config.simulation.model.common.pathological = true;
    config.simulation.sample_rate_hz = samplerate_hz;
    config.algorithm.model.common.voxel_size_mm = *voxel_size;
    config.algorithm.learning_rate = 1e3;
    config.algorithm.freeze_delays = false;
    config.algorithm.freeze_gains = false;
    config.algorithm.batch_size = 0;
    config
}

fn setup_inputs(
    config: &Config,
) -> anyhow::Result<(
    Data,
    Results,
    GPU,
    ResultsGPU,
    PredictionKernel,
    DerivationKernel,
    UpdateKernel,
)> {
    let simulation_config = &config.simulation;
    let data = Data::from_simulation_config(simulation_config)?;
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
        config.algorithm.batch_size,
        0,
        config.algorithm.optimizer,
    );
    results.model = Some(model);
    let gpu = GPU::new()?;
    let results_gpu = results.to_gpu(&gpu.queue)?;
    let prediction_kernel = PredictionKernel::new(
        &gpu,
        &results_gpu.estimations,
        &results_gpu.model,
        results
            .model
            .as_ref()
            .context("Model should be available for prediction kernel creation")?
            .spatial_description
            .voxels
            .count_states() as i32,
        results
            .model
            .as_ref()
            .context("Model should be available for prediction kernel creation")?
            .spatial_description
            .sensors
            .count() as i32,
        results.estimations.measurements.num_steps() as i32,
    );
    let actual_measurements = data.simulation.measurements.to_gpu(&gpu.queue)?;
    let derivation_kernel = DerivationKernel::new(
        &gpu,
        &results_gpu.estimations,
        &results_gpu.derivatives,
        &actual_measurements,
        &results_gpu.model,
        results
            .model
            .as_ref()
            .context("Model should be available for derivation kernel creation")?
            .spatial_description
            .voxels
            .count_states() as i32,
        results
            .model
            .as_ref()
            .context("Model should be available for derivation kernel creation")?
            .spatial_description
            .sensors
            .count() as i32,
        results.estimations.measurements.num_steps() as i32,
        &config.algorithm,
    );
    let update_kernel = UpdateKernel::new(
        &gpu,
        &results_gpu.derivatives,
        &results_gpu.model,
        results
            .model
            .as_ref()
            .context("Model should be available for update kernel creation")?
            .spatial_description
            .voxels
            .count_states() as i32,
        results.estimations.measurements.num_steps() as i32,
        &config.algorithm,
    )?;

    Ok((
        data,
        results,
        gpu,
        results_gpu,
        prediction_kernel?,
        derivation_kernel?,
        update_kernel,
    ))
}

criterion_group! {name = gpu_benches;
config = Criterion::default().measurement_time(Duration::from_secs(10)).sample_size(10);
targets=run_benches}
criterion_main!(gpu_benches);
