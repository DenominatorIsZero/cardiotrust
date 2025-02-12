use ocl::{Buffer, Context, Kernel, Program, Queue};

use crate::core::{
    algorithm::{
        estimation::EstimationsGPU, metrics::MetricsGPU, refinement::derivation::DerivativesGPU,
    },
    config::algorithm::Algorithm,
    data::shapes::Measurements,
    model::ModelGPU,
};

use super::GPU;

pub struct MetricsKernel {
    mse_step_kernel: Kernel,
    max_reg_step_kernel: Kernel,
    loss_step_kernel: Kernel,
    batch_kernel: Kernel,
}

impl MetricsKernel {
    #[allow(
        clippy::missing_panics_doc,
        clippy::cast_sign_loss,
        clippy::cast_possible_truncation,
        clippy::cast_possible_wrap,
        clippy::cast_precision_loss,
        clippy::too_many_lines
    )]
    #[must_use]
    pub fn new(
        gpu: &GPU,
        estimations: &EstimationsGPU,
        derivatives: &DerivativesGPU,
        metrics: &MetricsGPU,
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

        let metrics_src =
            std::fs::read_to_string("src/core/algorithm/gpu/kernels/metrics.cl").unwrap();
        let metrics_program = Program::builder().src(metrics_src).build(context).unwrap();

        let max_size = device.max_wg_size().unwrap();
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
            .unwrap();

        let max_reg_step_kernel = Kernel::builder()
            .program(&metrics_program)
            .name("store_max_reg")
            .queue(queue.clone())
            .global_work_size(1)
            .arg(&metrics.loss_maximum_regularization)
            .arg(&derivatives.maximum_regularization_sum)
            .arg(&estimations.step)
            .build()
            .unwrap();

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
            .unwrap();

        let max_size = device.max_wg_size().unwrap();
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
            .unwrap();

        Self {
            mse_step_kernel,
            max_reg_step_kernel,
            loss_step_kernel,
            batch_kernel,
        }
    }

    #[allow(clippy::missing_panics_doc)]
    pub fn execute_step(&self) {
        // TODO: Optimize prediction by running multiple beats in parallel using async kernel execution.
        // This would allow better GPU utilization by processing independent beats simultaneously.
        // See prediction.rs for implementation details.
        unsafe {
            self.mse_step_kernel.enq().unwrap();
            self.max_reg_step_kernel.enq().unwrap();
            self.loss_step_kernel.enq().unwrap();
        }
    }
    #[allow(clippy::missing_panics_doc)]
    pub fn execute_batch(&self) {
        // TODO: Optimize prediction by running multiple beats in parallel using async kernel execution.
        // This would allow better GPU utilization by processing independent beats simultaneously.
        // See prediction.rs for implementation details.
        unsafe {
            self.batch_kernel.enq().unwrap();
        }
    }
}
