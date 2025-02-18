use ocl::{Kernel, Program};

use super::GPU;
use crate::core::{
    algorithm::refinement::derivation::DerivativesGPU, config::algorithm::Algorithm,
    model::ModelGPU,
};

pub struct UpdateKernel {
    gains_kernel: Kernel,
    coefs_kernel: Kernel,
    freeze_gains: bool,
    freeze_delays: bool,
}

impl UpdateKernel {
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
        derivatives: &DerivativesGPU,
        model: &ModelGPU,
        number_of_states: i32,
        number_of_steps: i32,
        config: &Algorithm,
    ) -> Self {
        let context = &gpu.context;
        let queue = &gpu.queue;
        let number_of_voxels = number_of_states / 3;

        let gains_src =
            std::fs::read_to_string("src/core/algorithm/gpu/kernels/update_gains.cl").unwrap();
        let gains_program = Program::builder().src(gains_src).build(context).unwrap();
        let gains_kernel = Kernel::builder()
            .program(&gains_program)
            .name("update_gains")
            .queue(queue.clone())
            .global_work_size([number_of_states, 78])
            .arg(&model.functional_description.ap_params.gains)
            .arg(&derivatives.gains)
            .arg(config.learning_rate / number_of_steps as f32) // not accounting for batch size at the moment. might want to fix that later
            .arg(number_of_states)
            .build()
            .unwrap();

        let coefs_src =
            std::fs::read_to_string("src/core/algorithm/gpu/kernels/update_coefs.cl").unwrap();
        let coefs_program = Program::builder().src(coefs_src).build(context).unwrap();

        let coefs_kernel = Kernel::builder()
            .program(&coefs_program)
            .name("update_coefs")
            .queue(queue.clone())
            .global_work_size([number_of_voxels, 26])
            .arg(&model.functional_description.ap_params.coefs)
            .arg(&model.functional_description.ap_params.delays)
            .arg(&derivatives.coefs)
            .arg(config.learning_rate / number_of_steps as f32) // not accounting for batch size at the moment. might want to fix that later
            .arg(number_of_states)
            .build()
            .unwrap();

        Self {
            gains_kernel,
            coefs_kernel,
            freeze_gains: config.freeze_gains,
            freeze_delays: config.freeze_delays,
        }
    }

    #[allow(clippy::missing_panics_doc)]
    pub fn execute(&self) {
        // TODO: Optimize prediction by running multiple beats in parallel using async kernel execution.
        // This would allow better GPU utilization by processing independent beats simultaneously.
        // See prediction.rs for implementation details.
        unsafe {
            if !self.freeze_gains {
                self.gains_kernel.enq().unwrap();
            }
            if !self.freeze_delays {
                self.coefs_kernel.enq().unwrap();
            }
        }
    }
    pub fn set_freeze_delays(&mut self, value: bool) {
        self.freeze_delays = value;
    }
    pub fn set_freeze_gains(&mut self, value: bool) {
        self.freeze_gains = value;
    }
}

#[cfg(test)]
mod tests {

    use approx::assert_relative_eq;

    use super::UpdateKernel;
    use crate::core::{
        algorithm::{
            estimation::{calculate_residuals, prediction::calculate_system_prediction},
            gpu::{derivation::DerivationKernel, prediction::PredictionKernel, GPU},
            refinement::{
                derivation::{
                    calculate_derivatives_coefs_textbook, calculate_derivatives_gains,
                    calculate_mapped_residuals, calculate_maximum_regularization,
                },
                update::{roll_delays, update_delays_sgd, update_gains_sgd},
            },
        },
        config::Config,
        data::Data,
        scenario::results::Results,
    };
    #[test]
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_possible_wrap,
        clippy::too_many_lines,
        clippy::similar_names
    )]
    fn test_update() {
        let mut config = Config::default();
        config.algorithm.freeze_delays = false;
        let mut results_cpu = Results::get_default();
        let gpu = GPU::new();
        let results_gpu = results_cpu.to_gpu(&gpu.queue);
        let data = Data::get_default();
        let actual_measurements = data.simulation.measurements.to_gpu(&gpu.queue);
        let number_of_states = data.simulation.system_states.num_states();
        let number_of_sensors = results_cpu
            .model
            .as_ref()
            .unwrap()
            .spatial_description
            .sensors
            .count();

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

        let derivation_kernel = DerivationKernel::new(
            &gpu,
            &results_gpu.estimations,
            &results_gpu.derivatives,
            &actual_measurements,
            &results_gpu.model,
            number_of_states as i32,
            results_cpu
                .model
                .as_ref()
                .unwrap()
                .spatial_description
                .sensors
                .count() as i32,
            results_cpu.estimations.measurements.num_steps() as i32,
            &config.algorithm,
        );

        let update_kernel = UpdateKernel::new(
            &gpu,
            &results_gpu.derivatives,
            &results_gpu.model,
            number_of_states as i32,
            results_cpu.estimations.measurements.num_steps() as i32,
            &config.algorithm,
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
            calculate_residuals(&mut results_cpu.estimations, &data, 0, step);
            calculate_mapped_residuals(
                &mut results_cpu.derivatives.mapped_residuals,
                &results_cpu.estimations.residuals,
                &results_cpu
                    .model
                    .as_ref()
                    .unwrap()
                    .functional_description
                    .measurement_matrix
                    .at_beat(0),
            );
            calculate_maximum_regularization(
                &mut results_cpu.derivatives.maximum_regularization,
                &mut results_cpu.derivatives.maximum_regularization_sum,
                &results_cpu.estimations.system_states.at_step(step),
                config.algorithm.maximum_regularization_threshold,
            );
            calculate_derivatives_gains(
                &mut results_cpu.derivatives.gains,
                &results_cpu.estimations.ap_outputs_now,
                &results_cpu.derivatives.maximum_regularization,
                &results_cpu.derivatives.mapped_residuals,
                &config.algorithm,
                number_of_sensors,
            );
            calculate_derivatives_coefs_textbook(
                &mut results_cpu.derivatives,
                &results_cpu.estimations,
                &results_cpu.model.as_ref().unwrap().functional_description,
                step,
                &config.algorithm,
            );
            results_gpu
                .estimations
                .step
                .write([step as i32].as_slice())
                .enq()
                .unwrap();
            prediction_kernel.execute();
            derivation_kernel.execute();
        }
        let batch_size = results_cpu.estimations.measurements.num_steps();
        update_gains_sgd(
            &mut results_cpu
                .model
                .as_mut()
                .unwrap()
                .functional_description
                .ap_params
                .gains,
            &results_cpu.derivatives.gains,
            config.algorithm.learning_rate,
            batch_size,
        );
        update_delays_sgd(
            &mut results_cpu
                .model
                .as_mut()
                .unwrap()
                .functional_description
                .ap_params
                .coefs,
            &results_cpu.derivatives.coefs,
            config.algorithm.learning_rate,
            batch_size,
            0.0f32,
        );
        let model = results_cpu.model.as_mut().unwrap();
        roll_delays(
            &mut model.functional_description.ap_params.coefs,
            &mut model.functional_description.ap_params.delays,
        );
        update_kernel.execute();
        results_from_gpu.update_from_gpu(&results_gpu);
        assert_relative_eq!(
            results_cpu
                .model
                .as_ref()
                .unwrap()
                .functional_description
                .ap_params
                .gains
                .as_slice()
                .unwrap(),
            results_from_gpu
                .model
                .as_ref()
                .unwrap()
                .functional_description
                .ap_params
                .gains
                .as_slice()
                .unwrap(),
            epsilon = 1e-5
        );
        assert_relative_eq!(
            results_cpu
                .model
                .as_ref()
                .unwrap()
                .functional_description
                .ap_params
                .coefs
                .as_slice()
                .unwrap(),
            results_from_gpu
                .model
                .as_ref()
                .unwrap()
                .functional_description
                .ap_params
                .coefs
                .as_slice()
                .unwrap(),
            epsilon = 1e-5
        );
        assert_eq!(
            results_cpu
                .model
                .as_ref()
                .unwrap()
                .functional_description
                .ap_params
                .delays
                .as_slice()
                .unwrap(),
            results_from_gpu
                .model
                .as_ref()
                .unwrap()
                .functional_description
                .ap_params
                .delays
                .as_slice()
                .unwrap(),
        );
    }
}
