use ocl::{Buffer, Kernel, Program};

use super::GPU;
use crate::core::{
    algorithm::{estimation::EstimationsGPU, refinement::derivation::DerivativesGPU},
    config::algorithm::Algorithm,
    model::ModelGPU,
};

pub struct DerivationKernel {
    residual_kernel: Kernel,
    reset_mapped_residual_kernel: Kernel,
    mapped_residual_kernel: Kernel,
    maximum_regularization_kernel: Kernel,
    gains_kernel: Kernel,
    fir_kernel: Kernel,
    iir_kernel: Kernel,
    coefs_kernel: Kernel,
    freeze_gains: bool,
    freeze_delays: bool,
}

impl DerivationKernel {
    #[allow(
        clippy::missing_panics_doc,
        clippy::cast_sign_loss,
        clippy::cast_possible_truncation,
        clippy::cast_possible_wrap,
        clippy::cast_precision_loss,
        clippy::too_many_lines
    )]
    #[must_use]
    #[tracing::instrument(level = "trace", skip_all)]
    pub fn new(
        gpu: &GPU,
        estimations: &EstimationsGPU,
        derivatives: &DerivativesGPU,
        actual_measurements: &Buffer<f32>,
        model: &ModelGPU,
        number_of_states: i32,
        number_of_sensors: i32,
        number_of_steps: i32,
        config: &Algorithm,
    ) -> Self {
        let context = &gpu.context;
        let queue = &gpu.queue;
        let device = &gpu.device;
        let number_of_voxels = number_of_states / 3;

        let residual_src =
            std::fs::read_to_string("src/core/algorithm/gpu/kernels/calculate_residuals.cl")
                .unwrap();
        let residual_program = Program::builder().src(residual_src).build(context).unwrap();
        let residual_kernel = Kernel::builder()
            .program(&residual_program)
            .name("calculate_residuals")
            .queue(queue.clone())
            .global_work_size(number_of_sensors)
            .arg(&estimations.residuals)
            .arg(&estimations.measurements)
            .arg(actual_measurements)
            .arg(&estimations.step)
            .arg(&estimations.beat)
            .arg(number_of_sensors)
            .arg(number_of_steps)
            .build()
            .unwrap();

        let atomic_src =
            std::fs::read_to_string("src/core/algorithm/gpu/kernels/atomic.cl").unwrap();
        let mapped_residual_src =
            std::fs::read_to_string("src/core/algorithm/gpu/kernels/mapped_residual.cl").unwrap();
        let mapped_residuals_program = Program::builder()
            .src(format!("{atomic_src}\n{mapped_residual_src}"))
            .build(context)
            .unwrap();

        let reset_mapped_residual_kernel = Kernel::builder()
            .program(&mapped_residuals_program)
            .name("reset_mapped_residuals")
            .queue(queue.clone())
            .global_work_size(number_of_states)
            .arg(&derivatives.mapped_residuals)
            .arg(number_of_states)
            .build()
            .unwrap();

        let max_size = device.max_wg_size().unwrap();
        let work_group_size = max_size.min(number_of_sensors as usize).next_power_of_two();
        let sensors_work_group_size =
            (number_of_sensors as usize).next_multiple_of(work_group_size) as i32;
        let mapped_residual_kernel = Kernel::builder()
            .program(&mapped_residuals_program)
            .name("calculate_mapped_residuals")
            .queue(queue.clone())
            .global_work_size([number_of_states, sensors_work_group_size])
            .local_work_size([1, work_group_size])
            .arg(&derivatives.mapped_residuals)
            .arg(&model.functional_description.measurement_matrix)
            .arg(&estimations.residuals)
            .arg(&estimations.beat)
            .arg_local::<f32>(work_group_size)
            .arg(number_of_states)
            .arg(number_of_sensors)
            .build()
            .unwrap();

        let maximum_regularization_src =
            std::fs::read_to_string("src/core/algorithm/gpu/kernels/maximum_regularization.cl")
                .unwrap();
        let maximum_regularization_program = Program::builder()
            .src(format!("{atomic_src}\n{maximum_regularization_src}"))
            .build(context)
            .unwrap();

        let max_size = device.max_wg_size().unwrap();
        let work_group_size = max_size.min(number_of_voxels as usize).next_power_of_two();
        let voxel_work_group_size =
            (number_of_voxels as usize).next_multiple_of(work_group_size) as i32;

        let maximum_regularization_kernel = Kernel::builder()
            .program(&maximum_regularization_program)
            .name("calculate_maximum_regularization")
            .queue(queue.clone())
            .global_work_size(voxel_work_group_size)
            .local_work_size(work_group_size)
            .arg(&derivatives.maximum_regularization)
            .arg(&derivatives.maximum_regularization_sum)
            .arg(&estimations.system_states)
            .arg_local::<f32>(work_group_size)
            .arg(&estimations.step)
            .arg(config.maximum_regularization_threshold)
            .arg(number_of_voxels)
            .build()
            .unwrap();

        let derivatives_gains_src = std::fs::read_to_string(
            "src/core/algorithm/gpu/kernels/calculate_derivatives_gains.cl",
        )
        .unwrap();
        let derivatives_gains_program = Program::builder()
            .src(derivatives_gains_src)
            .build(context)
            .unwrap();

        let gains_kernel = Kernel::builder()
            .program(&derivatives_gains_program)
            .name("calculate_derivatives_gains")
            .queue(queue.clone())
            .global_work_size([number_of_states, 78])
            .arg(&derivatives.gains)
            .arg(&estimations.ap_outputs_now)
            .arg(&derivatives.maximum_regularization)
            .arg(&derivatives.mapped_residuals)
            .arg(config.mse_strength / number_of_sensors as f32)
            .arg(config.maximum_regularization_strength)
            .arg(number_of_states)
            .build()
            .unwrap();

        let derivatives_coefs_src = std::fs::read_to_string(
            "src/core/algorithm/gpu/kernels/calculate_derivatives_coefs.cl",
        )
        .unwrap();
        let derivatives_coefs_program = Program::builder()
            .src(derivatives_coefs_src)
            .build(context)
            .unwrap();

        let fir_kernel = Kernel::builder()
            .program(&derivatives_coefs_program)
            .name("calculate_derivatives_coefs_fir")
            .queue(queue.clone())
            .global_work_size([number_of_states, 78])
            .arg(&derivatives.coefs_fir)
            .arg(&estimations.system_states)
            .arg(&model.functional_description.ap_params.output_state_indices)
            .arg(&model.functional_description.ap_params.coefs)
            .arg(&model.functional_description.ap_params.delays)
            .arg(&estimations.step)
            .arg(number_of_states)
            .build()
            .unwrap();

        let iir_kernel = Kernel::builder()
            .program(&derivatives_coefs_program)
            .name("calculate_derivatives_coefs_iir")
            .queue(queue.clone())
            .global_work_size([number_of_states, 78])
            .arg(&derivatives.coefs_iir)
            .arg(&estimations.ap_outputs_last)
            .arg(&model.functional_description.ap_params.coefs)
            .arg(&model.functional_description.ap_params.delays)
            .arg(&estimations.step)
            .arg(number_of_states)
            .build()
            .unwrap();

        let coefs_kernel = Kernel::builder()
            .program(&derivatives_coefs_program)
            .name("calculate_derivatives_coefs_combine")
            .queue(queue.clone())
            .global_work_size([number_of_states, 78])
            .local_work_size([3, 3])
            .arg(&derivatives.coefs)
            .arg(&derivatives.coefs_iir)
            .arg(&derivatives.coefs_fir)
            .arg(&model.functional_description.ap_params.gains)
            .arg(&derivatives.mapped_residuals)
            .arg(&model.functional_description.ap_params.coefs)
            .arg(&model.functional_description.ap_params.delays)
            .arg_local::<f32>(9) // 4x4 local memory
            .arg(config.mse_strength / number_of_sensors as f32)
            .arg(number_of_states)
            .build()
            .unwrap();

        Self {
            residual_kernel,
            reset_mapped_residual_kernel,
            mapped_residual_kernel,
            maximum_regularization_kernel,
            gains_kernel,
            fir_kernel,
            iir_kernel,
            coefs_kernel,
            freeze_gains: config.freeze_gains,
            freeze_delays: config.freeze_delays,
        }
    }

    #[allow(clippy::missing_panics_doc)]
    #[tracing::instrument(level = "trace", skip_all)]
    pub fn execute(&self) {
        // TODO: Optimize prediction by running multiple beats in parallel using async kernel execution.
        // This would allow better GPU utilization by processing independent beats simultaneously.
        // See prediction.rs for implementation details.
        unsafe {
            self.residual_kernel.enq().unwrap();
            if !(self.freeze_gains && self.freeze_delays) {
                self.reset_mapped_residual_kernel.enq().unwrap();
                self.mapped_residual_kernel.enq().unwrap();
            }
            self.maximum_regularization_kernel.enq().unwrap();
            if !self.freeze_gains {
                self.gains_kernel.enq().unwrap();
            }
            if !self.freeze_delays {
                self.fir_kernel.enq().unwrap();
                self.iir_kernel.enq().unwrap();
                self.coefs_kernel.enq().unwrap();
            }
        }
    }

    pub const fn set_freeze_delays(&mut self, value: bool) {
        self.freeze_delays = value;
    }
    pub const fn set_freeze_gains(&mut self, value: bool) {
        self.freeze_gains = value;
    }
}

#[cfg(test)]
mod tests {

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
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_possible_wrap,
        clippy::too_many_lines,
        clippy::similar_names
    )]
    fn test_derivation() {
        let mut config = Config::default();
        config.algorithm.freeze_delays = false;
        let mut results_cpu = Results::get_default();
        let gpu = GPU::new();
        let results_gpu = results_cpu.to_gpu(&gpu.queue);
        let data = Data::get_default().expect("Failed to create default data for test");
        let actual_measurements = data.simulation.measurements.to_gpu(&gpu.queue);
        let number_of_states = data.simulation.system_states.num_states();
        let number_of_sensors = results_cpu
            .model
            .as_ref()
            .unwrap()
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
                .unwrap()
                .spatial_description
                .voxels
                .count_states() as i32,
            results_cpu
                .model
                .as_ref()
                .unwrap()
                .spatial_description
                .sensors
                .count() as i32,
            results_cpu.estimations.measurements.num_steps() as i32,
        );

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
                .unwrap()
                .spatial_description
                .sensors
                .count() as i32,
            results_cpu.estimations.measurements.num_steps() as i32,
            &config.algorithm,
        );

        let mut results_from_gpu = results_cpu.clone();
        // comparison loop
        for step in 0..results_cpu.estimations.measurements.num_steps() {
            calculate_system_prediction(
                &mut results_cpu.estimations,
                &results_cpu.model.as_ref().unwrap().functional_description,
                0,
                step,
            );
            calculate_residuals(&mut results_cpu.estimations, &data, 0, step);
            calculate_mapped_residuals(
                &mut results_cpu.derivatives.mapped_residuals,
                &results_cpu.estimations.residuals,
                &results_cpu
                    .model
                    .as_ref()
                    .unwrap()
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
                &results_cpu.model.as_ref().unwrap().functional_description,
                step,
                &config.algorithm,
            );
            results_gpu
                .estimations
                .step
                .write([step as i32].as_slice())
                .enq()
                .unwrap();
            prediction_kernel.execute();
            derivation_kernel.execute();
            results_from_gpu.update_from_gpu(&results_gpu);
            assert_relative_eq!(
                results_cpu.estimations.residuals.as_slice().unwrap(),
                results_from_gpu.estimations.residuals.as_slice().unwrap(),
                epsilon = 1e-6
            );
            assert_relative_eq!(
                results_cpu.derivatives.mapped_residuals.as_slice().unwrap(),
                results_from_gpu
                    .derivatives
                    .mapped_residuals
                    .as_slice()
                    .unwrap(),
                epsilon = 1e-5
            );
            assert_relative_eq!(
                results_cpu
                    .derivatives
                    .maximum_regularization
                    .as_slice()
                    .unwrap(),
                results_from_gpu
                    .derivatives
                    .maximum_regularization
                    .as_slice()
                    .unwrap(),
                epsilon = 1e-5
            );
            assert_relative_eq!(
                results_cpu.derivatives.maximum_regularization_sum,
                results_from_gpu.derivatives.maximum_regularization_sum,
                max_relative = 0.01
            );
            assert_relative_eq!(
                results_cpu.derivatives.gains.as_slice().unwrap(),
                results_from_gpu.derivatives.gains.as_slice().unwrap(),
                epsilon = 1e-5
            );
            assert_relative_eq!(
                results_cpu.derivatives.coefs_iir.as_slice().unwrap(),
                results_from_gpu.derivatives.coefs_iir.as_slice().unwrap(),
                epsilon = 1e-6
            );
            assert_relative_eq!(
                results_cpu.derivatives.coefs_fir.as_slice().unwrap()[..1000],
                results_from_gpu.derivatives.coefs_fir.as_slice().unwrap()[..1000],
                epsilon = 1e-6
            );
            assert_relative_eq!(
                results_cpu.derivatives.coefs.as_slice().unwrap(),
                results_from_gpu.derivatives.coefs.as_slice().unwrap(),
                epsilon = 1e-6
            );
        }
    }
    #[test]
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_possible_wrap,
        clippy::too_many_lines,
        clippy::similar_names,
        clippy::cast_sign_loss,
        clippy::cast_precision_loss,
        clippy::suboptimal_flops
    )]
    fn test_mapped_residuals_kernel() {
        let gpu = GPU::new();

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
            .unwrap();

        let measurement_matrix_buffer = Buffer::builder()
            .queue(gpu.queue.clone())
            .flags(MemFlags::new().read_only().copy_host_ptr())
            .copy_host_slice(&measurement_matrix)
            .len(measurement_matrix.len())
            .build()
            .unwrap();

        let residuals_buffer = Buffer::builder()
            .queue(gpu.queue.clone())
            .flags(MemFlags::new().read_only().copy_host_ptr())
            .copy_host_slice(&residuals)
            .len(residuals.len())
            .build()
            .unwrap();

        let beat_buffer = Buffer::builder()
            .queue(gpu.queue.clone())
            .flags(MemFlags::new().read_only().copy_host_ptr())
            .len(beat.len())
            .copy_host_slice(&beat)
            .build()
            .unwrap();

        // Set up kernel
        let atomic_src =
            std::fs::read_to_string("src/core/algorithm/gpu/kernels/atomic.cl").unwrap();
        let mapped_residual_src =
            std::fs::read_to_string("src/core/algorithm/gpu/kernels/mapped_residual.cl").unwrap();
        let program = Program::builder()
            .src(format!("{atomic_src}\n{mapped_residual_src}"))
            .build(&gpu.context)
            .unwrap();

        let max_size = gpu.device.max_wg_size().unwrap();
        let work_group_size = max_size.min(num_sensors as usize);
        let sensors_work_group_size = (work_group_size
            * (num_sensors as f32 / work_group_size as f32).ceil() as usize)
            as i32;
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
            .unwrap();

        // Execute and verify
        let mut result = vec![0.0f32; num_states as usize];
        mapped_residuals_buffer
            .write(&vec![0.0f32; num_states as usize])
            .enq()
            .unwrap();
        unsafe {
            kernel.enq().unwrap();
        }
        mapped_residuals_buffer.read(&mut result).enq().unwrap();

        // Calculate expected result
        let expected = [
            1.0 * 0.5 + 0.0 * -0.3 + 1.0 * 0.8 + 2.0 * 0.2,
            0.0 * 0.5 + 1.0 * -0.3 + 1.0 * 0.8 + -0.2,
        ];

        assert_relative_eq!(&result[..], &expected[..], epsilon = 1e-6);
    }
}
