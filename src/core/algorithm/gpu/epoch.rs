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
