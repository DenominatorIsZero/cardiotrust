use anyhow::{Context as AnyhowContext, Result};
use ocl::{Kernel, Program};

use super::GPU;
use crate::core::{
    algorithm::{
        estimation::EstimationsGPU, metrics::MetricsGPU, refinement::derivation::DerivativesGPU,
    },
    config::algorithm::Algorithm,
};

#[allow(clippy::struct_field_names)]
pub struct MetricsKernel {
    mse_step_kernel: Kernel,
    max_reg_step_kernel: Kernel,
    loss_step_kernel: Kernel,
    batch_kernel: Kernel,
}

impl MetricsKernel {
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
        number_of_sensors: i32,
        number_of_steps: i32,
        config: &Algorithm,
    ) -> Result<Self> {
        let context = &gpu.context;
        let queue = &gpu.queue;
        let device = &gpu.device;

        let metrics_src =
            std::fs::read_to_string("src/core/algorithm/gpu/kernels/metrics.cl")
                .context("Failed to read metrics kernel source file")?;
        let atomic_src =
            std::fs::read_to_string("src/core/algorithm/gpu/kernels/atomic.cl")
                .context("Failed to read atomic kernel source file")?;
        let metrics_program = Program::builder()
            .src(format!("{atomic_src}\n{metrics_src}"))
            .build(context)
            .context("Failed to build OpenCL program for metrics kernels")?;

        let max_size = device.max_wg_size()
            .context("Failed to query GPU device maximum work group size")?;
        let work_group_size = max_size.min(number_of_sensors as usize).next_power_of_two();
        let sensors_work_group_size =
            (number_of_sensors as usize).next_multiple_of(work_group_size) as i32;
        let mse_step_kernel = Kernel::builder()
            .program(&metrics_program)
            .name("calculate_mse_step")
            .queue(queue.clone())
            .global_work_size(sensors_work_group_size)
            .local_work_size(work_group_size)
            .arg(&estimations.residuals)
            .arg(&metrics.loss_mse)
            .arg_local::<f32>(work_group_size)
            .arg(&estimations.step)
            .arg(number_of_sensors)
            .build()
            .context("Failed to build MSE step calculation kernel")?;

        let max_reg_step_kernel = Kernel::builder()
            .program(&metrics_program)
            .name("store_max_reg")
            .queue(queue.clone())
            .global_work_size(1)
            .arg(&metrics.loss_maximum_regularization)
            .arg(&derivatives.maximum_regularization_sum)
            .arg(&estimations.step)
            .build()
            .context("Failed to build maximum regularization storage kernel")?;

        let loss_step_kernel = Kernel::builder()
            .program(&metrics_program)
            .name("calculate_final_loss")
            .queue(queue.clone())
            .global_work_size(1)
            .arg(&metrics.loss)
            .arg(&metrics.loss_mse)
            .arg(&metrics.loss_maximum_regularization)
            .arg(&estimations.step)
            .arg(config.maximum_regularization_strength)
            .build()
            .context("Failed to build final loss calculation kernel")?;

        let max_size = device.max_wg_size()
            .context("Failed to query GPU device maximum work group size for batch processing")?;
        let work_group_size = max_size.min(number_of_steps as usize).next_power_of_two();
        let steps_work_group_size =
            (number_of_steps as usize).next_multiple_of(work_group_size) as i32;
        let batch_kernel = Kernel::builder()
            .program(&metrics_program)
            .name("calculate_metrics_batch")
            .queue(queue.clone())
            .global_work_size(steps_work_group_size)
            .local_work_size(work_group_size)
            .arg(&metrics.loss_mse_batch)
            .arg(&metrics.loss_maximum_regularization_batch)
            .arg(&metrics.loss_batch)
            .arg(&metrics.loss_mse)
            .arg(&metrics.loss_maximum_regularization)
            .arg(&metrics.loss)
            .arg_local::<f32>(work_group_size)
            .arg_local::<f32>(work_group_size)
            .arg_local::<f32>(work_group_size)
            .arg(&estimations.epoch)
            .arg(number_of_steps)
            .build()
            .context("Failed to build batch metrics calculation kernel")?;

        Ok(Self {
            mse_step_kernel,
            max_reg_step_kernel,
            loss_step_kernel,
            batch_kernel,
        })
    }

    #[tracing::instrument(level = "trace", skip_all)]
    pub fn execute_step(&self) -> Result<()> {
        // TODO: Optimize prediction by running multiple beats in parallel using async kernel execution.
        // This would allow better GPU utilization by processing independent beats simultaneously.
        // See prediction.rs for implementation details.
        unsafe {
            self.mse_step_kernel.enq()
                .context("Failed to execute MSE step calculation kernel")?;
            self.max_reg_step_kernel.enq()
                .context("Failed to execute maximum regularization storage kernel")?;
            self.loss_step_kernel.enq()
                .context("Failed to execute final loss calculation kernel")?;
        }
        Ok(())
    }
    #[tracing::instrument(level = "trace", skip_all)]
    pub fn execute_batch(&self) -> Result<()> {
        // TODO: Optimize prediction by running multiple beats in parallel using async kernel execution.
        // This would allow better GPU utilization by processing independent beats simultaneously.
        // See prediction.rs for implementation details.
        unsafe {
            self.batch_kernel.enq()
                .context("Failed to execute batch metrics calculation kernel")?;
        }
        Ok(())
    }
}
