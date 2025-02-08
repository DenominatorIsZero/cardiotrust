#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{
        algorithm::estimation::{prediction::innovate_system_states_v1, Estimations},
        config::algorithm::Algorithm,
        data::Data,
        model::functional::FunctionalDescription,
        scenario::results::Results,
    };
    use crate::{
        core::{
            config::{
                model::{SensorArrayGeometry, SensorArrayMotion},
                simulation::Simulation as SimulationConfig,
            },
            model::Model,
        },
        vis::plotting::{
            gif::states::states_spherical_plot_over_time,
            png::{line::standard_y_plot, states::states_spherical_plot},
            PlotSlice, StateSphericalPlotMode,
        },
    };
    use approx::assert_relative_eq;
    use ndarray::{Array2, Array3, Dim};
    use ocl::{Buffer, Context, Device, Kernel, Platform, Program, Queue};
    #[test]
    #[allow(clippy::too_many_lines)]
    fn test_innovate_system_states() {
        let mut simulation_config = SimulationConfig::default();
        simulation_config.model.common.pathological = true;
        simulation_config.model.common.sensor_array_geometry = SensorArrayGeometry::Cube;
        simulation_config.model.common.sensor_array_motion = SensorArrayMotion::Static;
        let data = Data::from_simulation_config(&simulation_config)
            .expect("Model parameters to be valid.");

        let mut algorithm_config = Algorithm {
            learning_rate: 1.0,
            epochs: 3,
            ..Default::default()
        };
        algorithm_config.model.common.apply_system_update = true;
        algorithm_config.model.common.sensor_array_geometry = SensorArrayGeometry::Cube;
        algorithm_config.model.common.sensor_array_motion = SensorArrayMotion::Static;

        let mut model = Model::from_model_config(
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
            .src(format!("{}\n{}", atomic_src, prediction_src))
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
            .queue(queue.clone())
            .global_work_size([number_of_states, number_of_offsets])
            .arg_named("ap_outputs_now", &ap_outputs_now_buf)
            .arg_named("ap_outputs_last", &ap_outputs_last_buf)
            .arg_named("system_states", &system_states_buf)
            .arg_named("ap_coefs", &ap_coefs_buf)
            .arg_named("ap_delays", &ap_delays_buf)
            .arg_named("ap_gains", &ap_gains_buf)
            .arg_named("output_state_indices", &output_state_indices_buf)
            .arg_named("step", 0 as i32)
            .arg_named("num_states", number_of_states as i32)
            .arg_named("num_offsets", number_of_offsets as i32)
            .build()
            .unwrap();

        // comparison loop
        for step in 0..results.estimations.system_states.num_steps() {
            innovate_system_states_v1(&mut results.estimations, &functional_description, step);
            kernel.set_arg("step", step as i32);
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
