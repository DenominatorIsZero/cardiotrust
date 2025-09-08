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
        metrics: &MetricsGPU,
        number_of_states: i32,
        number_of_sensors: i32,
        number_of_steps: i32,
    ) -> Self {
        let context = &gpu.context;
        let queue = &gpu.queue;
        let number_of_voxels = number_of_states / 3;

        let reset_src = std::fs::read_to_string("src/core/algorithm/gpu/kernels/reset.cl").unwrap();
        let reset_program = Program::builder().src(reset_src).build(context).unwrap();
        let system_states_kernel = Kernel::builder()
            .program(&reset_program)
            .name("reset_float")
            .queue(queue.clone())
            .global_work_size(number_of_steps * number_of_states)
            .arg(&estimations.system_states)
            .build()
            .unwrap();
        let measurements_kernel = Kernel::builder()
            .program(&reset_program)
            .name("reset_float")
            .queue(queue.clone())
            .global_work_size(number_of_steps * number_of_sensors)
            .arg(&estimations.measurements)
            .build()
            .unwrap();
        let mse_kernel = Kernel::builder()
            .program(&reset_program)
            .name("reset_float")
            .queue(queue.clone())
            .global_work_size(number_of_steps)
            .arg(&metrics.loss_mse)
            .build()
            .unwrap();
        let ap_outputs_kernel = Kernel::builder()
            .program(&reset_program)
            .name("reset_float")
            .queue(queue.clone())
            .global_work_size(number_of_states * 78)
            .arg(&estimations.ap_outputs_now)
            .build()
            .unwrap();
        let gains_kernel = Kernel::builder()
            .program(&reset_program)
            .name("reset_float")
            .queue(queue.clone())
            .global_work_size(number_of_states * 78)
            .arg(&derivatives.gains)
            .build()
            .unwrap();
        let coefs_kernel = Kernel::builder()
            .program(&reset_program)
            .name("reset_float")
            .queue(queue.clone())
            .global_work_size(number_of_voxels * 26)
            .arg(&derivatives.coefs)
            .build()
            .unwrap();
        let iir_kernel = Kernel::builder()
            .program(&reset_program)
            .name("reset_float")
            .queue(queue.clone())
            .global_work_size(number_of_states * 78)
            .arg(&derivatives.coefs_iir)
            .build()
            .unwrap();
        let fir_kernel = Kernel::builder()
            .program(&reset_program)
            .name("reset_float")
            .queue(queue.clone())
            .global_work_size(number_of_states * 78)
            .arg(&derivatives.coefs_fir)
            .build()
            .unwrap();
        let maximum_regularization_sum_kernel = Kernel::builder()
            .program(&reset_program)
            .name("reset_float")
            .queue(queue.clone())
            .global_work_size(1)
            .arg(&derivatives.maximum_regularization_sum)
            .build()
            .unwrap();
        let step_kernel = Kernel::builder()
            .program(&reset_program)
            .name("reset_int")
            .queue(queue.clone())
            .global_work_size(1)
            .arg(&estimations.step)
            .build()
            .unwrap();

        Self {
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
        }
    }

    #[allow(clippy::missing_panics_doc)]
    #[tracing::instrument(level = "trace", skip_all)]
    pub fn execute(&self) {
        // TODO: Optimize prediction by running multiple beats in parallel using async kernel execution.
        // This would allow better GPU utilization by processing independent beats simultaneously.
        // See prediction.rs for implementation details.
        unsafe {
            self.system_states_kernel.enq().unwrap();
            self.measurements_kernel.enq().unwrap();
            self.mse_kernel.enq().unwrap();
            self.ap_outputs_kernel.enq().unwrap();
            self.gains_kernel.enq().unwrap();
            self.coefs_kernel.enq().unwrap();
            self.iir_kernel.enq().unwrap();
            self.fir_kernel.enq().unwrap();
            self.maximum_regularization_sum_kernel.enq().unwrap();
            self.step_kernel.enq().unwrap();
        }
    }
}
