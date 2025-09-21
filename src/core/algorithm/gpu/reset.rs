use anyhow::{Context, Result};
use ocl::{Kernel, Program};

use super::GPU;
use crate::core::algorithm::{
    estimation::EstimationsGPU, metrics::MetricsGPU, refinement::derivation::DerivativesGPU,
};

#[allow(clippy::struct_field_names)]
pub struct ResetKernel {
    system_states_kernel: Kernel,
    measurements_kernel: Kernel,
    mse_kernel: Kernel,
    ap_outputs_kernel: Kernel,
    gains_kernel: Kernel,
    coefs_kernel: Kernel,
    iir_kernel: Kernel,
    fir_kernel: Kernel,
    maximum_regularization_sum_kernel: Kernel,
    step_kernel: Kernel,
}

impl ResetKernel {
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
        metrics: &MetricsGPU,
        number_of_states: i32,
        number_of_sensors: i32,
        number_of_steps: i32,
    ) -> Result<Self> {
        let context = &gpu.context;
        let queue = &gpu.queue;
        let number_of_voxels = number_of_states / 3;

        let reset_src = std::fs::read_to_string("src/core/algorithm/gpu/kernels/reset.cl")
            .context("Failed to read reset kernel source file")?;
        let reset_program = Program::builder()
            .src(reset_src)
            .build(context)
            .context("Failed to build OpenCL program for reset kernels")?;
        let system_states_kernel = Kernel::builder()
            .program(&reset_program)
            .name("reset_float")
            .queue(queue.clone())
            .global_work_size(number_of_steps * number_of_states)
            .arg(&estimations.system_states)
            .build()
            .context("Failed to build system states reset kernel")?;
        let measurements_kernel = Kernel::builder()
            .program(&reset_program)
            .name("reset_float")
            .queue(queue.clone())
            .global_work_size(number_of_steps * number_of_sensors)
            .arg(&estimations.measurements)
            .build()
            .context("Failed to build measurements reset kernel")?;
        let mse_kernel = Kernel::builder()
            .program(&reset_program)
            .name("reset_float")
            .queue(queue.clone())
            .global_work_size(number_of_steps)
            .arg(&metrics.loss_mse)
            .build()
            .context("Failed to build MSE reset kernel")?;
        let ap_outputs_kernel = Kernel::builder()
            .program(&reset_program)
            .name("reset_float")
            .queue(queue.clone())
            .global_work_size(number_of_states * 78)
            .arg(&estimations.ap_outputs_now)
            .build()
            .context("Failed to build AP outputs reset kernel")?;
        let gains_kernel = Kernel::builder()
            .program(&reset_program)
            .name("reset_float")
            .queue(queue.clone())
            .global_work_size(number_of_states * 78)
            .arg(&derivatives.gains)
            .build()
            .context("Failed to build gains reset kernel")?;
        let coefs_kernel = Kernel::builder()
            .program(&reset_program)
            .name("reset_float")
            .queue(queue.clone())
            .global_work_size(number_of_voxels * 26)
            .arg(&derivatives.coefs)
            .build()
            .context("Failed to build coefficients reset kernel")?;
        let iir_kernel = Kernel::builder()
            .program(&reset_program)
            .name("reset_float")
            .queue(queue.clone())
            .global_work_size(number_of_states * 78)
            .arg(&derivatives.coefs_iir)
            .build()
            .context("Failed to build IIR coefficients reset kernel")?;
        let fir_kernel = Kernel::builder()
            .program(&reset_program)
            .name("reset_float")
            .queue(queue.clone())
            .global_work_size(number_of_states * 78)
            .arg(&derivatives.coefs_fir)
            .build()
            .context("Failed to build FIR coefficients reset kernel")?;
        let maximum_regularization_sum_kernel = Kernel::builder()
            .program(&reset_program)
            .name("reset_float")
            .queue(queue.clone())
            .global_work_size(1)
            .arg(&derivatives.maximum_regularization_sum)
            .build()
            .context("Failed to build maximum regularization sum reset kernel")?;
        let step_kernel = Kernel::builder()
            .program(&reset_program)
            .name("reset_int")
            .queue(queue.clone())
            .global_work_size(1)
            .arg(&estimations.step)
            .build()
            .context("Failed to build step reset kernel")?;

        Ok(Self {
            system_states_kernel,
            measurements_kernel,
            mse_kernel,
            ap_outputs_kernel,
            gains_kernel,
            coefs_kernel,
            iir_kernel,
            fir_kernel,
            maximum_regularization_sum_kernel,
            step_kernel,
        })
    }

    #[tracing::instrument(level = "trace", skip_all)]
    pub fn execute(&self) -> Result<()> {
        // TODO: Optimize prediction by running multiple beats in parallel using async kernel execution.
        // This would allow better GPU utilization by processing independent beats simultaneously.
        // See prediction.rs for implementation details.
        unsafe {
            self.system_states_kernel
                .enq()
                .context("Failed to execute system states reset kernel")?;
            self.measurements_kernel
                .enq()
                .context("Failed to execute measurements reset kernel")?;
            self.mse_kernel
                .enq()
                .context("Failed to execute MSE reset kernel")?;
            self.ap_outputs_kernel
                .enq()
                .context("Failed to execute AP outputs reset kernel")?;
            self.gains_kernel
                .enq()
                .context("Failed to execute gains reset kernel")?;
            self.coefs_kernel
                .enq()
                .context("Failed to execute coefficients reset kernel")?;
            self.iir_kernel
                .enq()
                .context("Failed to execute IIR coefficients reset kernel")?;
            self.fir_kernel
                .enq()
                .context("Failed to execute FIR coefficients reset kernel")?;
            self.maximum_regularization_sum_kernel
                .enq()
                .context("Failed to execute maximum regularization sum reset kernel")?;
            self.step_kernel
                .enq()
                .context("Failed to execute step reset kernel")?;
        }
        Ok(())
    }
}
