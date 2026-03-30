use anyhow::Context;
use approx::assert_relative_eq;
use ocl::{Buffer, Kernel, MemFlags, Program};

use crate::core::{
    algorithm::{
        estimation::{calculate_residuals, prediction::calculate_system_prediction},
        gpu::{derivation::DerivationKernel, prediction::PredictionKernel, GPU},
        refinement::derivation::{
            calculate_derivatives_coefs_textbook, calculate_derivatives_gains,
            calculate_mapped_residuals, calculate_maximum_regularization,
        },
    },
    config::Config,
    data::Data,
    scenario::results::Results,
};
#[test]
#[ignore = "expensive integration test"]
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::too_many_lines,
    clippy::similar_names
)]
fn test_derivation() -> anyhow::Result<()> {
    let mut config = Config::default();
    config.algorithm.freeze_delays = false;
    let mut results_cpu = Results::get_default();
    let gpu = GPU::new()?;
    let results_gpu = results_cpu.to_gpu(&gpu.queue)?;
    let data = Data::get_default().expect("Failed to create default data for test");
    let actual_measurements = data.simulation.measurements.to_gpu(&gpu.queue)?;
    let number_of_states = data.simulation.system_states.num_states();
    let number_of_sensors = results_cpu
        .model
        .as_ref()
        .context("Model should be available in GPU derivation test")?
        .spatial_description
        .sensors
        .count();

    let prediction_kernel = PredictionKernel::new(
        &gpu,
        &results_gpu.estimations,
        &results_gpu.model,
        results_cpu
            .model
            .as_ref()
            .context("Model not available for derivation prediction test")?
            .spatial_description
            .voxels
            .count_states() as i32,
        results_cpu
            .model
            .as_ref()
            .context("Model not available for derivation prediction test")?
            .spatial_description
            .sensors
            .count() as i32,
        results_cpu.estimations.measurements.num_steps() as i32,
    )?;

    let derivation_kernel = DerivationKernel::new(
        &gpu,
        &results_gpu.estimations,
        &results_gpu.derivatives,
        &actual_measurements,
        &results_gpu.model,
        number_of_states as i32,
        results_cpu
            .model
            .as_ref()
            .context("Model not available for derivation test")?
            .spatial_description
            .sensors
            .count() as i32,
        results_cpu.estimations.measurements.num_steps() as i32,
        &config.algorithm,
    )?;

    let mut results_from_gpu = results_cpu.clone();
    // comparison loop
    for step in 0..results_cpu.estimations.measurements.num_steps() {
        calculate_system_prediction(
            &mut results_cpu.estimations,
            &results_cpu
                .model
                .as_ref()
                .context("Model not available for system prediction test")?
                .functional_description,
            0,
            step,
        )?;
        calculate_residuals(&mut results_cpu.estimations, &data, 0, step);
        calculate_mapped_residuals(
            &mut results_cpu.derivatives.mapped_residuals,
            &results_cpu.estimations.residuals,
            &results_cpu
                .model
                .as_ref()
                .context("Model not available for mapped residuals test")?
                .functional_description
                .measurement_matrix
                .at_beat(0),
        );
        calculate_maximum_regularization(
            &mut results_cpu.derivatives.maximum_regularization,
            &mut results_cpu.derivatives.maximum_regularization_sum,
            &results_cpu.estimations.system_states.at_step(step),
            config.algorithm.maximum_regularization_threshold,
        );
        calculate_derivatives_gains(
            &mut results_cpu.derivatives.gains,
            &results_cpu.estimations.ap_outputs_now,
            &results_cpu.derivatives.maximum_regularization,
            &results_cpu.derivatives.mapped_residuals,
            &config.algorithm,
            number_of_sensors,
        );
        calculate_derivatives_coefs_textbook(
            &mut results_cpu.derivatives,
            &results_cpu.estimations,
            &results_cpu
                .model
                .as_ref()
                .context("Model not available for derivatives coefficients test")?
                .functional_description,
            step,
            &config.algorithm,
        )?;
        results_gpu
            .estimations
            .step
            .write([step as i32].as_slice())
            .enq()
            .context("Failed to write step data to GPU buffer")?;
        prediction_kernel.execute()?;
        derivation_kernel.execute()?;
        results_from_gpu.update_from_gpu(&results_gpu)?;
        assert_relative_eq!(
            results_cpu
                .estimations
                .residuals
                .as_slice()
                .context("Failed to convert CPU residuals to slice for comparison")?,
            results_from_gpu
                .estimations
                .residuals
                .as_slice()
                .context("Failed to convert GPU residuals to slice for comparison")?,
            epsilon = 1e-6
        );
        assert_relative_eq!(
            results_cpu
                .derivatives
                .mapped_residuals
                .as_slice()
                .context("Failed to convert CPU mapped residuals to slice for comparison")?,
            results_from_gpu
                .derivatives
                .mapped_residuals
                .as_slice()
                .context("Failed to convert GPU mapped residuals to slice for comparison")?,
            epsilon = 1e-5
        );
        assert_relative_eq!(
            results_cpu
                .derivatives
                .maximum_regularization
                .as_slice()
                .context("Failed to convert CPU maximum regularization to slice for comparison")?,
            results_from_gpu
                .derivatives
                .maximum_regularization
                .as_slice()
                .context("Failed to convert GPU maximum regularization to slice for comparison")?,
            epsilon = 1e-5
        );
        assert_relative_eq!(
            results_cpu.derivatives.maximum_regularization_sum,
            results_from_gpu.derivatives.maximum_regularization_sum,
            max_relative = 0.01
        );
        assert_relative_eq!(
            results_cpu
                .derivatives
                .gains
                .as_slice()
                .context("Failed to convert CPU gains to slice for comparison")?,
            results_from_gpu
                .derivatives
                .gains
                .as_slice()
                .context("Failed to convert GPU gains to slice for comparison")?,
            epsilon = 1e-5
        );
        assert_relative_eq!(
            results_cpu
                .derivatives
                .coefs_iir
                .as_slice()
                .context("Failed to convert CPU IIR coefficients to slice for comparison")?,
            results_from_gpu
                .derivatives
                .coefs_iir
                .as_slice()
                .context("Failed to convert GPU IIR coefficients to slice for comparison")?,
            epsilon = 1e-6
        );
        assert_relative_eq!(
            &results_cpu
                .derivatives
                .coefs_fir
                .as_slice()
                .context("Failed to convert CPU FIR coefficients to slice for comparison")?[..1000],
            &results_from_gpu
                .derivatives
                .coefs_fir
                .as_slice()
                .context("Failed to convert GPU FIR coefficients to slice for comparison")?[..1000],
            epsilon = 1e-6
        );
        assert_relative_eq!(
            results_cpu
                .derivatives
                .coefs
                .as_slice()
                .context("Failed to convert CPU coefficients to slice for comparison")?,
            results_from_gpu
                .derivatives
                .coefs
                .as_slice()
                .context("Failed to convert GPU coefficients to slice for comparison")?,
            epsilon = 1e-6
        );
    }
    Ok(())
}
#[test]
#[ignore = "expensive integration test"]
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::too_many_lines,
    clippy::similar_names,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::suboptimal_flops
)]
fn test_mapped_residuals_kernel() -> anyhow::Result<()> {
    let gpu = GPU::new()?;

    // Simple test dimensions
    let num_states = 2;
    let num_sensors = 4;

    // Create test data
    let measurement_matrix: Vec<f32> = vec![
        1.0, 0.0, // sensor 0
        0.0, 1.0, // sensor 1
        1.0, 1.0, // sensor 2
        2.0, -1.0, // sensor 3
    ];
    let residuals: Vec<f32> = vec![0.5, -0.3, 0.8, 0.2];
    let beat = vec![0];

    // Create GPU buffers
    let mapped_residuals_buffer = Buffer::builder()
        .queue(gpu.queue.clone())
        .flags(MemFlags::new().write_only())
        .len(num_states)
        .build()
        .context("Failed to create mapped residuals buffer on GPU")?;

    let measurement_matrix_buffer = Buffer::builder()
        .queue(gpu.queue.clone())
        .flags(MemFlags::new().read_only().copy_host_ptr())
        .copy_host_slice(&measurement_matrix)
        .len(measurement_matrix.len())
        .build()
        .context("Failed to create measurement matrix buffer on GPU")?;

    let residuals_buffer = Buffer::builder()
        .queue(gpu.queue.clone())
        .flags(MemFlags::new().read_only().copy_host_ptr())
        .copy_host_slice(&residuals)
        .len(residuals.len())
        .build()
        .context("Failed to create residuals buffer on GPU")?;

    let beat_buffer = Buffer::builder()
        .queue(gpu.queue.clone())
        .flags(MemFlags::new().read_only().copy_host_ptr())
        .len(beat.len())
        .copy_host_slice(&beat)
        .build()
        .context("Failed to create beat buffer on GPU")?;

    // Set up kernel
    let atomic_src = std::fs::read_to_string("src/core/algorithm/gpu/kernels/atomic.cl")
        .context("Failed to read atomic kernel source for test")?;
    let mapped_residual_src =
        std::fs::read_to_string("src/core/algorithm/gpu/kernels/mapped_residual.cl")
            .context("Failed to read mapped residual kernel source for test")?;
    let program = Program::builder()
        .src(format!("{atomic_src}\n{mapped_residual_src}"))
        .build(&gpu.context)
        .context("Failed to build test kernel program")?;

    let max_size = gpu
        .device
        .max_wg_size()
        .context("Failed to query GPU device work group size for test")?;
    let work_group_size = max_size.min(num_sensors as usize);
    let sensors_work_group_size =
        (work_group_size * (num_sensors as f32 / work_group_size as f32).ceil() as usize) as i32;
    let kernel = Kernel::builder()
        .program(&program)
        .name("calculate_mapped_residuals")
        .queue(gpu.queue)
        .global_work_size([num_states, sensors_work_group_size])
        .local_work_size([1, work_group_size])
        .arg(&mapped_residuals_buffer)
        .arg(&measurement_matrix_buffer)
        .arg(&residuals_buffer)
        .arg(&beat_buffer)
        .arg_local::<f32>(work_group_size)
        .arg(num_states)
        .arg(num_sensors)
        .build()
        .context("Failed to build test kernel")?;

    // Execute and verify
    let mut result = vec![0.0f32; num_states as usize];
    mapped_residuals_buffer
        .write(&vec![0.0f32; num_states as usize])
        .enq()
        .context("Failed to write initial data to GPU buffer")?;
    unsafe {
        kernel
            .enq()
            .context("Failed to execute test kernel on GPU")?;
    }
    mapped_residuals_buffer
        .read(&mut result)
        .enq()
        .context("Failed to read test results from GPU buffer")?;

    // Calculate expected result
    let expected = [
        1.0 * 0.5 + 0.0 * -0.3 + 1.0 * 0.8 + 2.0 * 0.2,
        0.0 * 0.5 + 1.0 * -0.3 + 1.0 * 0.8 + -0.2,
    ];

    assert_relative_eq!(&result[..], &expected[..], epsilon = 1e-6);
    Ok(())
}
