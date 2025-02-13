use ocl::{Buffer, Kernel, Program};

use super::{
    derivation::DerivationKernel, helper::HelperKernel, metrics::MetricsKernel,
    prediction::PredictionKernel, reset::ResetKernel, update::UpdateKernel, GPU,
};
use crate::core::{
    algorithm::{
        estimation::EstimationsGPU, metrics::MetricsGPU, refinement::derivation::DerivativesGPU,
    },
    config::algorithm::Algorithm,
    model::ModelGPU,
    scenario::results::ResultsGPU,
};

pub struct EpochKernel {
    reset_kernel: ResetKernel,
    prediction_kernel: PredictionKernel,
    derivation_kernel: DerivationKernel,
    update_kernel: UpdateKernel,
    metrics_kernel: MetricsKernel,
    helper_kernel: HelperKernel,
    number_of_steps: i32,
}

impl EpochKernel {
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
        results: &ResultsGPU,
        actual_measurements: &Buffer<f32>,
        config: &Algorithm,
        number_of_states: i32,
        number_of_sensors: i32,
        number_of_steps: i32,
    ) -> Self {
        let reset_kernel = ResetKernel::new(
            gpu,
            &results.estimations,
            &results.derivatives,
            number_of_states,
            number_of_steps,
        );
        let prediction_kernel = PredictionKernel::new(
            gpu,
            &results.estimations,
            &results.model,
            number_of_states,
            number_of_sensors,
            number_of_steps,
        );
        let derivation_kernel = DerivationKernel::new(
            gpu,
            &results.estimations,
            &results.derivatives,
            actual_measurements,
            &results.model,
            number_of_states,
            number_of_sensors,
            number_of_steps,
            config,
        );
        let update_kernel = UpdateKernel::new(
            gpu,
            &results.derivatives,
            &results.model,
            number_of_states,
            number_of_steps,
            config,
        );
        let metrics_kernel = MetricsKernel::new(
            gpu,
            &results.estimations,
            &results.derivatives,
            &results.metrics,
            number_of_sensors,
            number_of_steps,
            config,
        );
        let helper_kernel = HelperKernel::new(gpu, &results.estimations);
        Self {
            reset_kernel,
            prediction_kernel,
            derivation_kernel,
            update_kernel,
            metrics_kernel,
            helper_kernel,
            number_of_steps,
        }
    }

    #[allow(clippy::missing_panics_doc)]
    pub fn execute(&self) {
        // TODO: Optimize prediction by running multiple beats in parallel using async kernel execution.
        // This would allow better GPU utilization by processing independent beats simultaneously.
        // See prediction.rs for implementation details.

        // reset
        self.reset_kernel.execute();

        // prediction
        // TODO: Add support for multiple beats.

        for _ in 0..self.number_of_steps {
            self.prediction_kernel.execute();
            self.metrics_kernel.execute_step();
            self.derivation_kernel.execute();
            self.helper_kernel.increase_step();
        }
        self.update_kernel.execute();
        self.metrics_kernel.execute_step();
        self.helper_kernel.increase_epoch();
    }
}

mod tests {

    use approx::assert_relative_eq;
    use ocl::{Buffer, Kernel, MemFlags, Program};

    use crate::core::{
        algorithm::{
            estimation::{calculate_residuals, prediction::calculate_system_prediction},
            gpu::{derivation::DerivationKernel, prediction::PredictionKernel, GPU},
            refinement::derivation::{
                calculate_derivatives_coefs_textbook, calculate_derivatives_gains,
                calculate_mapped_residuals, calculate_maximum_regularization,
            },
            run_epoch,
        },
        config::Config,
        data::Data,
        scenario::results::Results,
    };

    use super::EpochKernel;
    #[test]
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_possible_wrap,
        clippy::too_many_lines,
        clippy::similar_names
    )]
    fn test_epoch() {
        let config = Config::default();
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
        let epoch_kernel = EpochKernel::new(
            &gpu,
            &results_gpu,
            &actual_measurements,
            &config.algorithm,
            number_of_states as i32,
            number_of_sensors as i32,
            results_cpu.estimations.measurements.num_steps() as i32,
        );
        let mut results_from_gpu = results_cpu.clone();

        let mut batch_index = 0;
        for _ in 0..config.algorithm.epochs {
            run_epoch(&mut results_cpu, &mut batch_index, &data, &config.algorithm);
            epoch_kernel.execute();
            results_from_gpu.update_from_gpu(&results_gpu);
            // Model Parameters
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
                epsilon = 1e-3
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
}
