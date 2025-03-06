use ocl::{Kernel, Program};

use super::GPU;
use crate::core::algorithm::estimation::EstimationsGPU;

pub struct HelperKernel {
    step_kernel: Kernel,
    epoch_kernel: Kernel,
}

impl HelperKernel {
    #[allow(
        clippy::missing_panics_doc,
        clippy::cast_sign_loss,
        clippy::cast_possible_truncation,
        clippy::cast_possible_wrap,
        clippy::cast_precision_loss,
        clippy::too_many_lines
    )]
    #[must_use]
    pub fn new(gpu: &GPU, estimations: &EstimationsGPU) -> Self {
        let context = &gpu.context;
        let queue = &gpu.queue;

        let helper_src =
            std::fs::read_to_string("src/core/algorithm/gpu/kernels/helper.cl").unwrap();
        let helper_program = Program::builder().src(helper_src).build(context).unwrap();
        let step_kernel = Kernel::builder()
            .program(&helper_program)
            .name("increase_int")
            .queue(queue.clone())
            .global_work_size(1)
            .arg(&estimations.step)
            .build()
            .unwrap();
        let epoch_kernel = Kernel::builder()
            .program(&helper_program)
            .name("increase_int")
            .queue(queue.clone())
            .global_work_size(1)
            .arg(&estimations.epoch)
            .build()
            .unwrap();

        Self {
            step_kernel,
            epoch_kernel,
        }
    }

    #[allow(clippy::missing_panics_doc)]
    pub fn increase_step(&self) {
        unsafe {
            self.step_kernel.enq().unwrap();
        }
    }
    #[allow(clippy::missing_panics_doc)]
    pub fn increase_epoch(&self) {
        unsafe {
            self.epoch_kernel.enq().unwrap();
        }
    }
}
