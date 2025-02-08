use ocl::{Context, Kernel, Program, Queue};

use crate::core::{algorithm::estimation::EstimationsGPU, model::ModelGPU};

use super::GPU;

pub struct PredictionKernel {
    innovate_kernel: Kernel,
    add_control_kernel: Kernel,
    predict_measurements_kernel: Kernel,
}

impl PredictionKernel {
    #[allow(clippy::missing_panics_doc)]
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
            .name("innovate")
            .queue(queue.clone())
            .global_work_size([number_of_states, 78])
            .arg_named("ap_outputs_now", &estimations.ap_outputs_now)
            .arg_named("ap_outputs_last", &estimations.ap_outputs_last)
            .arg_named("system_states", &estimations.system_states)
            .arg_named("ap_coefs", &model.functional_description.ap_params.coefs)
            .arg_named("ap_delays", &model.functional_description.ap_params.delays)
            .arg_named("ap_gains", &model.functional_description.ap_params.gains)
            .arg_named(
                "output_state_indices",
                &model.functional_description.ap_params.output_state_indices,
            )
            .arg_named("step", &estimations.step)
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
            .name("add_control")
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

        let predict_measurements_src =
            std::fs::read_to_string("src/core/algorithm/gpu/kernels/predict_measurements.cl")
                .unwrap();
        let predict_measurements_program = Program::builder()
            .src(predict_measurements_src)
            .build(context)
            .unwrap();
        let max_work_group_size = device.max_wg_size().unwrap();
        let work_group_size = max_work_group_size.min(256);
        let predict_measurements_kernel = Kernel::builder()
            .program(&predict_measurements_program)
            .queue(queue.clone())
            .global_work_size(number_of_sensors)
            .local_work_size(work_group_size)
            .arg(&estimations.measurements)
            .arg(&model.functional_description.measurement_matrix)
            .arg(&estimations.system_states)
            .arg(&estimations.beat)
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
    pub fn enq(&self) {
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
    use ndarray::Array2;
    use ocl::{Buffer, Context, Device, Kernel, Platform, Program, Queue};

    use crate::core::{
        algorithm::estimation::prediction::innovate_system_states_v1,
        config::{
            algorithm::Algorithm,
            model::{SensorArrayGeometry, SensorArrayMotion},
            simulation::Simulation as SimulationConfig,
        },
        data::Data,
        model::Model,
        scenario::results::Results,
    };
    #[test]
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_possible_wrap,
        clippy::too_many_lines
    )]
    fn test_innovate_system_states() {
        let mut simulation_config = SimulationConfig::default();
        simulation_config.model.common.pathological = true;
        simulation_config.model.common.sensor_array_geometry = SensorArrayGeometry::Cube;
        simulation_config.model.common.sensor_array_motion = SensorArrayMotion::Static;
        let _data = Data::from_simulation_config(&simulation_config)
            .expect("Model parameters to be valid.");

        let mut algorithm_config = Algorithm {
            learning_rate: 1.0,
            epochs: 3,
            ..Default::default()
        };
        algorithm_config.model.common.apply_system_update = true;
        algorithm_config.model.common.sensor_array_geometry = SensorArrayGeometry::Cube;
        algorithm_config.model.common.sensor_array_motion = SensorArrayMotion::Static;

        let model = Model::from_model_config(
            &algorithm_config.model,
            simulation_config.sample_rate_hz,
            simulation_config.duration_s,
        )
        .expect("Model parameters to be valid.");

        let mut results = Results::new(
            algorithm_config.epochs,
            model.functional_description.control_function_values.shape()[0],
            model.spatial_description.sensors.count(),
            model.spatial_description.voxels.count_states(),
            simulation_config
                .model
                .common
                .sensor_array_motion_steps
                .iter()
                .product(),
            algorithm_config.batch_size,
            algorithm_config.optimizer,
        );

        let functional_description = &model.functional_description;

        // setup GPU
        let platform = Platform::default();
        let device = Device::first(platform).unwrap();
        let context = Context::builder()
            .platform(platform)
            .devices(device)
            .build()
            .unwrap();
        let queue = Queue::new(&context, device, None).unwrap();

        // Load kernel sources and create program (also before loop)
        let atomic_src =
            std::fs::read_to_string("src/core/algorithm/gpu/kernels/atomic.cl").unwrap();
        let prediction_src =
            std::fs::read_to_string("src/core/algorithm/gpu/kernels/innovate.cl").unwrap();
        let program = Program::builder()
            .src(format!("{atomic_src}\n{prediction_src}"))
            .build(&context)
            .unwrap();

        // Create individual buffers
        let ap_outputs_now_buf = Buffer::builder()
            .queue(queue.clone())
            .len(results.estimations.ap_outputs_now.len())
            .copy_host_slice(results.estimations.ap_outputs_now.as_slice().unwrap())
            .build()
            .unwrap();

        let ap_outputs_last_buf = Buffer::builder()
            .queue(queue.clone())
            .len(results.estimations.ap_outputs_last.len())
            .copy_host_slice(results.estimations.ap_outputs_last.as_slice().unwrap())
            .build()
            .unwrap();

        let system_states_buf = Buffer::builder()
            .queue(queue.clone())
            .len(results.estimations.system_states.len())
            .copy_host_slice(results.estimations.system_states.as_slice().unwrap())
            .build()
            .unwrap();

        let ap_coefs_buf = Buffer::builder()
            .queue(queue.clone())
            .len(functional_description.ap_params.coefs.len())
            .copy_host_slice(functional_description.ap_params.coefs.as_slice().unwrap())
            .build()
            .unwrap();

        let ap_delays_buf = Buffer::builder()
            .queue(queue.clone())
            .len(functional_description.ap_params.delays.len())
            .copy_host_slice(functional_description.ap_params.delays.as_slice().unwrap())
            .build()
            .unwrap();

        let ap_gains_buf = Buffer::builder()
            .queue(queue.clone())
            .len(functional_description.ap_params.gains.len())
            .copy_host_slice(functional_description.ap_params.gains.as_slice().unwrap())
            .build()
            .unwrap();

        let output_state_indices_buf = Buffer::builder()
            .queue(queue.clone())
            .len(functional_description.ap_params.output_state_indices.len())
            .copy_host_slice(
                functional_description
                    .ap_params
                    .output_state_indices
                    .mapv(|opt| opt.map_or(-1i32, |val| val as i32))
                    .as_slice()
                    .unwrap(),
            )
            .build()
            .unwrap();

        let number_of_states = results.estimations.system_states.num_states();
        let number_of_offsets = results.estimations.ap_outputs_now.shape()[1];
        let kernel = Kernel::builder()
            .program(&program)
            .name("innovate_system_states")
            .queue(queue)
            .global_work_size([number_of_states, number_of_offsets])
            .arg_named("ap_outputs_now", &ap_outputs_now_buf)
            .arg_named("ap_outputs_last", &ap_outputs_last_buf)
            .arg_named("system_states", &system_states_buf)
            .arg_named("ap_coefs", &ap_coefs_buf)
            .arg_named("ap_delays", &ap_delays_buf)
            .arg_named("ap_gains", &ap_gains_buf)
            .arg_named("output_state_indices", &output_state_indices_buf)
            .arg_named("step", 0)
            .arg_named("num_states", number_of_states as i32)
            .arg_named("num_offsets", number_of_offsets as i32)
            .build()
            .unwrap();

        // comparison loop
        for step in 0..results.estimations.system_states.num_steps() {
            innovate_system_states_v1(&mut results.estimations, functional_description, step);
            kernel.set_arg("step", step as i32).unwrap();
            unsafe {
                kernel.enq().unwrap();
            }
        }

        // Read GPU results
        let mut gpu_system_states = vec![0.0f32; results.estimations.system_states.len()];
        system_states_buf
            .read(&mut gpu_system_states)
            .enq()
            .unwrap();
        let gpu_system_states =
            Array2::from_shape_vec(results.estimations.system_states.dim(), gpu_system_states)
                .unwrap();

        assert_relative_eq!(
            results.estimations.system_states.as_slice().unwrap(),
            gpu_system_states.as_slice().unwrap(),
            epsilon = 1e-6
        );
    }
}
