use ocl::{Kernel, Program};

use super::GPU;
use crate::core::{algorithm::estimation::EstimationsGPU, model::ModelGPU};

#[allow(clippy::struct_field_names)]
pub struct PredictionKernel {
    innovate_kernel: Kernel,
    add_control_kernel: Kernel,
    predict_measurements_kernel: Kernel,
}

impl PredictionKernel {
    #[allow(
        clippy::missing_panics_doc,
        clippy::cast_sign_loss,
        clippy::cast_possible_truncation,
        clippy::cast_possible_wrap
    )]
    #[must_use]
    pub fn new(
        gpu: &GPU,
        estimations: &EstimationsGPU,
        model: &ModelGPU,
        number_of_states: i32,
        number_of_sensors: i32,
        number_of_steps: i32,
    ) -> Self {
        let context = &gpu.context;
        let queue = &gpu.queue;
        let device = &gpu.device;

        let atomic_src =
            std::fs::read_to_string("src/core/algorithm/gpu/kernels/atomic.cl").unwrap();
        let innovate_src =
            std::fs::read_to_string("src/core/algorithm/gpu/kernels/innovate.cl").unwrap();
        let innovate_program = Program::builder()
            .src(format!("{atomic_src}\n{innovate_src}"))
            .build(context)
            .unwrap();

        let innovate_kernel = Kernel::builder()
            .program(&innovate_program)
            .name("innovate_system_states")
            .queue(queue.clone())
            .global_work_size([number_of_states, 128])
            .local_work_size([1, 128])
            .arg(&estimations.ap_outputs_now)
            .arg(&estimations.ap_outputs_last)
            .arg(&estimations.system_states)
            .arg(&model.functional_description.ap_params.coefs)
            .arg(&model.functional_description.ap_params.delays)
            .arg(&model.functional_description.ap_params.gains)
            .arg(&model.functional_description.ap_params.output_state_indices)
            .arg(&estimations.step)
            .arg_local::<f32>(128)
            .arg_named("num_states", number_of_states)
            .build()
            .unwrap();

        let add_control_src =
            std::fs::read_to_string("src/core/algorithm/gpu/kernels/add_control.cl").unwrap();
        let add_control_program = Program::builder()
            .src(add_control_src)
            .build(context)
            .unwrap();
        let add_control_kernel = Kernel::builder()
            .program(&add_control_program)
            .name("add_control_function")
            .queue(queue.clone())
            .global_work_size([number_of_states])
            .arg_named("stystem_states", &estimations.system_states)
            .arg_named(
                "control_matrix",
                &model.functional_description.control_matrix,
            )
            .arg_named("step", &estimations.step)
            .arg_named(
                "control_values",
                &model.functional_description.control_function_values,
            )
            .arg_named("num_states", number_of_states)
            .build()
            .unwrap();

        let max_size = device.max_wg_size().unwrap();
        let work_group_size = max_size.min(number_of_states as usize).next_power_of_two();
        let states_work_group_size =
            (number_of_states as usize).next_multiple_of(work_group_size) as i32;

        let predict_measurements_src =
            std::fs::read_to_string("src/core/algorithm/gpu/kernels/predict_measurements_local.cl")
                .unwrap();
        let predict_measurements_program = Program::builder()
            .src(format!("{atomic_src}\n{predict_measurements_src}"))
            .build(context)
            .unwrap();
        let predict_measurements_kernel = Kernel::builder()
            .program(&predict_measurements_program)
            .name("predict_measurements")
            .queue(queue.clone())
            .global_work_size([number_of_sensors, states_work_group_size])
            .local_work_size([1, work_group_size])
            .arg(&estimations.measurements)
            .arg(&model.functional_description.measurement_matrix)
            .arg(&estimations.system_states)
            .arg(&estimations.beat)
            .arg(&estimations.step)
            .arg_local::<f32>(work_group_size)
            .arg_named("num_sensors", number_of_sensors)
            .arg_named("num_states", number_of_states)
            .arg_named("num_steps", number_of_steps)
            .build()
            .unwrap();

        Self {
            innovate_kernel,
            add_control_kernel,
            predict_measurements_kernel,
        }
    }

    #[allow(clippy::missing_panics_doc)]
    pub fn execute(&self) {
        // TODO: Optimize prediction by running multiple beats in parallel using async kernel execution.
        // This would allow better GPU utilization by processing independent beats simultaneously.
        // See prediction.rs for implementation details.
        unsafe {
            self.innovate_kernel.enq().unwrap();
            self.add_control_kernel.enq().unwrap();
            self.predict_measurements_kernel.enq().unwrap();
        }
    }
}

#[cfg(test)]
mod tests {

    use approx::assert_relative_eq;

    use crate::core::{
        algorithm::{
            estimation::prediction::calculate_system_prediction,
            gpu::{prediction::PredictionKernel, GPU},
        },
        scenario::results::Results,
    };
    #[test]
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_possible_wrap,
        clippy::too_many_lines,
        clippy::similar_names
    )]
    fn test_innovate_system_states() {
        let mut results_cpu = Results::get_default();
        let gpu = GPU::new();
        let results_gpu = results_cpu.to_gpu(&gpu.queue);
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

        let mut results_from_gpu = results_cpu.clone();
        // comparison loop
        for step in 0..results_cpu.estimations.measurements.num_steps() {
            calculate_system_prediction(
                &mut results_cpu.estimations,
                &results_cpu.model.as_ref().unwrap().functional_description,
                0,
                step,
            );
            results_gpu
                .estimations
                .step
                .write([step as i32].as_slice())
                .enq()
                .unwrap();
            prediction_kernel.execute();
            results_from_gpu.update_from_gpu(&results_gpu);
            let dif = &*results_cpu.estimations.ap_outputs_now
                - &*results_from_gpu.estimations.ap_outputs_now;
            println!("Diference {:?}", dif.as_slice().unwrap());
            assert_relative_eq!(
                results_cpu.estimations.ap_outputs_now.as_slice().unwrap(),
                results_from_gpu
                    .estimations
                    .ap_outputs_now
                    .as_slice()
                    .unwrap(),
                epsilon = 1e-6
            );
        }
        results_from_gpu.update_from_gpu(&results_gpu);

        assert_relative_eq!(
            results_cpu.estimations.ap_outputs_now.as_slice().unwrap(),
            results_from_gpu
                .estimations
                .ap_outputs_now
                .as_slice()
                .unwrap(),
            epsilon = 1e-6
        );
        assert_relative_eq!(
            results_cpu.estimations.system_states.as_slice().unwrap(),
            results_from_gpu
                .estimations
                .system_states
                .as_slice()
                .unwrap(),
            epsilon = 1e-6
        );

        assert_relative_eq!(
            results_cpu.estimations.measurements.as_slice().unwrap(),
            results_from_gpu
                .estimations
                .measurements
                .as_slice()
                .unwrap(),
            epsilon = 1e-4
        );
    }
}
