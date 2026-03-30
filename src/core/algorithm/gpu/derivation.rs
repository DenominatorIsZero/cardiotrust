use anyhow::{Context, Result};
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
        clippy::cast_sign_loss,
        clippy::cast_possible_truncation,
        clippy::cast_possible_wrap,
        clippy::cast_precision_loss,
        clippy::too_many_lines
    )]
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
    ) -> Result<Self> {
        let context = &gpu.context;
        let queue = &gpu.queue;
        let device = &gpu.device;
        let number_of_voxels = number_of_states / 3;

        let residual_src =
            std::fs::read_to_string("src/core/algorithm/gpu/kernels/calculate_residuals.cl")
                .context(
                "Failed to read residuals kernel source file - ensure GPU kernels are available",
            )?;
        let residual_program = Program::builder()
            .src(residual_src)
            .build(context)
            .context("Failed to compile residuals kernel for GPU device")?;
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
            .context("Failed to build residuals kernel - check GPU device compatibility")?;

        let atomic_src = std::fs::read_to_string("src/core/algorithm/gpu/kernels/atomic.cl")
            .context("Failed to read atomic operations kernel source file")?;
        let mapped_residual_src =
            std::fs::read_to_string("src/core/algorithm/gpu/kernels/mapped_residual.cl")
                .context("Failed to read mapped residual kernel source file")?;
        let mapped_residuals_program = Program::builder()
            .src(format!("{atomic_src}\n{mapped_residual_src}"))
            .build(context)
            .context("Failed to compile mapped residuals kernel for GPU device")?;

        let reset_mapped_residual_kernel = Kernel::builder()
            .program(&mapped_residuals_program)
            .name("reset_mapped_residuals")
            .queue(queue.clone())
            .global_work_size(number_of_states)
            .arg(&derivatives.mapped_residuals)
            .arg(number_of_states)
            .build()
            .context("Failed to build reset mapped residuals kernel")?;

        let max_size = device
            .max_wg_size()
            .context("Failed to query GPU device maximum work group size")?;
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
            .context(
                "Failed to build mapped residuals kernel - check work group size compatibility",
            )?;

        let maximum_regularization_src =
            std::fs::read_to_string("src/core/algorithm/gpu/kernels/maximum_regularization.cl")
                .context("Failed to read maximum regularization kernel source file")?;
        let maximum_regularization_program = Program::builder()
            .src(format!("{atomic_src}\n{maximum_regularization_src}"))
            .build(context)
            .context("Failed to compile maximum regularization kernel for GPU device")?;

        let max_size = device
            .max_wg_size()
            .context("Failed to query GPU device maximum work group size for regularization")?;
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
            .context(
                "Failed to build maximum regularization kernel - check work group configuration",
            )?;

        let derivatives_gains_src = std::fs::read_to_string(
            "src/core/algorithm/gpu/kernels/calculate_derivatives_gains.cl",
        )
        .context("Failed to read derivatives gains kernel source file")?;
        let derivatives_gains_program = Program::builder()
            .src(derivatives_gains_src)
            .build(context)
            .context("Failed to compile derivatives gains kernel for GPU device")?;

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
            .context("Failed to build derivatives gains kernel")?;

        let derivatives_coefs_src = std::fs::read_to_string(
            "src/core/algorithm/gpu/kernels/calculate_derivatives_coefs.cl",
        )
        .context("Failed to read derivatives coefficients kernel source file")?;
        let derivatives_coefs_program = Program::builder()
            .src(derivatives_coefs_src)
            .build(context)
            .context("Failed to compile derivatives coefficients kernel for GPU device")?;

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
            .context("Failed to build FIR derivatives coefficients kernel")?;

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
            .context("Failed to build IIR derivatives coefficients kernel")?;

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
            .context("Failed to build combined coefficients kernel - check local work size compatibility")?;

        Ok(Self {
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
        })
    }

    #[tracing::instrument(level = "trace", skip_all)]
    pub fn execute(&self) -> Result<()> {
        // TODO: Optimize prediction by running multiple beats in parallel using async kernel execution.
        // This would allow better GPU utilization by processing independent beats simultaneously.
        // See prediction.rs for implementation details.
        unsafe {
            self.residual_kernel
                .enq()
                .context("Failed to execute residuals kernel on GPU")?;
            if !(self.freeze_gains && self.freeze_delays) {
                self.reset_mapped_residual_kernel
                    .enq()
                    .context("Failed to execute reset mapped residuals kernel on GPU")?;
                self.mapped_residual_kernel
                    .enq()
                    .context("Failed to execute mapped residuals kernel on GPU")?;
            }
            self.maximum_regularization_kernel
                .enq()
                .context("Failed to execute maximum regularization kernel on GPU")?;
            if !self.freeze_gains {
                self.gains_kernel
                    .enq()
                    .context("Failed to execute gains derivation kernel on GPU")?;
            }
            if !self.freeze_delays {
                self.fir_kernel
                    .enq()
                    .context("Failed to execute FIR coefficients derivation kernel on GPU")?;
                self.iir_kernel
                    .enq()
                    .context("Failed to execute IIR coefficients derivation kernel on GPU")?;
                self.coefs_kernel
                    .enq()
                    .context("Failed to execute combined coefficients derivation kernel on GPU")?;
            }
        }
        Ok(())
    }

    pub const fn set_freeze_delays(&mut self, value: bool) {
        self.freeze_delays = value;
    }
    pub const fn set_freeze_gains(&mut self, value: bool) {
        self.freeze_gains = value;
    }
}

#[cfg(test)]
mod tests;
