use anyhow::{Context as AnyhowContext, Result};
use ocl::{Kernel, Program};

use super::GPU;
use crate::core::algorithm::estimation::EstimationsGPU;

pub struct HelperKernel {
    step_kernel: Kernel,
    epoch_kernel: Kernel,
}

impl HelperKernel {
    #[allow(
        clippy::cast_sign_loss,
        clippy::cast_possible_truncation,
        clippy::cast_possible_wrap,
        clippy::cast_precision_loss,
        clippy::too_many_lines
    )]
    #[tracing::instrument(level = "trace", skip_all)]
    pub fn new(gpu: &GPU, estimations: &EstimationsGPU) -> Result<Self> {
        let context = &gpu.context;
        let queue = &gpu.queue;

        let helper_src =
            std::fs::read_to_string("src/core/algorithm/gpu/kernels/helper.cl")
                .context("Failed to read helper kernel source file")?;
        let helper_program = Program::builder().src(helper_src).build(context)
            .context("Failed to build OpenCL program for helper kernels")?;
        let step_kernel = Kernel::builder()
            .program(&helper_program)
            .name("increase_int")
            .queue(queue.clone())
            .global_work_size(1)
            .arg(&estimations.step)
            .build()
            .context("Failed to build step increment kernel")?;
        let epoch_kernel = Kernel::builder()
            .program(&helper_program)
            .name("increase_int")
            .queue(queue.clone())
            .global_work_size(1)
            .arg(&estimations.epoch)
            .build()
            .context("Failed to build epoch increment kernel")?;

        Ok(Self {
            step_kernel,
            epoch_kernel,
        })
    }

    #[tracing::instrument(level = "trace", skip_all)]
    pub fn increase_step(&self) -> Result<()> {
        unsafe {
            self.step_kernel.enq()
                .context("Failed to execute step increment kernel")?;
        }
        Ok(())
    }
    #[tracing::instrument(level = "trace", skip_all)]
    pub fn increase_epoch(&self) -> Result<()> {
        unsafe {
            self.epoch_kernel.enq()
                .context("Failed to execute epoch increment kernel")?;
        }
        Ok(())
    }
}
