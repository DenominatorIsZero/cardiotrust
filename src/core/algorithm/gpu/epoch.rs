use ocl::Buffer;

use super::{
    derivation::DerivationKernel, helper::HelperKernel, metrics::MetricsKernel,
    prediction::PredictionKernel, reset::ResetKernel, update::UpdateKernel, GPU,
};
use crate::core::{config::algorithm::Algorithm, scenario::results::ResultsGPU};

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
    #[tracing::instrument(level = "trace", skip_all)]
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
            &results.metrics,
            number_of_states,
            number_of_sensors,
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
    #[tracing::instrument(level = "trace", skip_all)]
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
            self.derivation_kernel.execute();
            self.metrics_kernel.execute_step();
            self.helper_kernel.increase_step();
        }
        self.update_kernel.execute();
        self.metrics_kernel.execute_batch();
        self.helper_kernel.increase_epoch();
    }
    pub const fn set_freeze_delays(&mut self, value: bool) {
        self.derivation_kernel.set_freeze_delays(value);
        self.update_kernel.set_freeze_delays(value);
    }
    pub const fn set_freeze_gains(&mut self, value: bool) {
        self.derivation_kernel.set_freeze_gains(value);
        self.update_kernel.set_freeze_gains(value);
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;
    use ndarray_stats::QuantileExt;

    use crate::core::{
        algorithm::{
            gpu::{epoch::EpochKernel, GPU},
            run_epoch,
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
    fn test_epoch() -> anyhow::Result<()> {
        let mut config = Config::default();
        config.algorithm.epochs = 10;
        config.algorithm.freeze_delays = false;
        config.algorithm.learning_rate = 100.0;
        let mut results_cpu = Results::get_default();
        let gpu = GPU::new();
        let results_gpu = results_cpu.to_gpu(&gpu.queue)?;
        let data = Data::get_default().expect("Failed to create default data for test");
        let actual_measurements = data.simulation.measurements.to_gpu(&gpu.queue)?;
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
        for epoch in 0..config.algorithm.epochs {
            println!("Epoch: {epoch}");
            run_epoch(&mut results_cpu, &mut batch_index, &data, &config.algorithm);
            epoch_kernel.execute();
            results_from_gpu.update_from_gpu(&results_gpu)?;
            // Model Parameters
            let delta_states = &*results_cpu.estimations.system_states
                - &*results_from_gpu.estimations.system_states;
            println!(
                "States: delta_max {}, delta_min {}",
                delta_states.max_skipnan(),
                delta_states.min_skipnan()
            );
            let delta_measurements = &*results_cpu.estimations.measurements
                - &*results_from_gpu.estimations.measurements;
            println!(
                "Measurements: delta_max {}, delta_min {}",
                delta_measurements.max_skipnan(),
                delta_measurements.min_skipnan()
            );
            let delta_residuals =
                &*results_cpu.estimations.residuals - &*results_from_gpu.estimations.residuals;
            println!(
                "Residuals: delta_max {}, delta_min {}",
                delta_residuals.max_skipnan(),
                delta_residuals.min_skipnan()
            );
            let delta_loss_mse =
                &*results_cpu.metrics.loss_mse - &*results_from_gpu.metrics.loss_mse;
            println!(
                "Loss MSE: delta_max {}, delta_min {}",
                delta_loss_mse.max_skipnan(),
                delta_loss_mse.min_skipnan()
            );
            let delta_loss_mr = &*results_cpu.metrics.loss_maximum_regularization
                - &*results_from_gpu.metrics.loss_maximum_regularization;
            println!(
                "Loss MR: delta_max {}, delta_min {}",
                delta_loss_mr.max_skipnan(),
                delta_loss_mr.min_skipnan()
            );
            assert_relative_eq!(
                results_cpu
                    .metrics
                    .loss_maximum_regularization
                    .as_slice()
                    .unwrap()[..100],
                results_from_gpu
                    .metrics
                    .loss_maximum_regularization
                    .as_slice()
                    .unwrap()[..100],
                epsilon = 1e-5
            );
            let delta_loss = &*results_cpu.metrics.loss - &*results_from_gpu.metrics.loss;
            println!(
                "Loss: delta_max {}, delta_min {}",
                delta_loss.max_skipnan(),
                delta_loss.min_skipnan()
            );
            let delta_gains = &*results_cpu
                .model
                .as_ref()
                .unwrap()
                .functional_description
                .ap_params
                .gains
                - &*results_from_gpu
                    .model
                    .as_ref()
                    .unwrap()
                    .functional_description
                    .ap_params
                    .gains;
            println!(
                "Gains: delta_max {}, delta_min {}",
                delta_gains.max_skipnan(),
                delta_gains.min_skipnan()
            );
            let delta_coefs = &*results_cpu
                .model
                .as_ref()
                .unwrap()
                .functional_description
                .ap_params
                .coefs
                - &*results_from_gpu
                    .model
                    .as_ref()
                    .unwrap()
                    .functional_description
                    .ap_params
                    .coefs;
            println!(
                "coefs: delta_max {}, delta_min {}",
                delta_coefs.max_skipnan(),
                delta_coefs.min_skipnan()
            );
            let delta_delays = &*results_cpu
                .model
                .as_ref()
                .unwrap()
                .functional_description
                .ap_params
                .delays
                - &*results_from_gpu
                    .model
                    .as_ref()
                    .unwrap()
                    .functional_description
                    .ap_params
                    .delays;
            println!(
                "delays: delta_max {}, delta_min {}",
                delta_delays.max().unwrap(),
                delta_delays.min().unwrap()
            );
        }
        let delta_loss = &*results_cpu.metrics.loss - &*results_from_gpu.metrics.loss;
        println!(
            "loss: delta_max {}, delta_min {}",
            delta_loss.max_skipnan(),
            delta_loss.min_skipnan()
        );
        assert_relative_eq!(
            results_cpu.metrics.loss_batch.as_slice().unwrap(),
            results_from_gpu.metrics.loss_batch.as_slice().unwrap(),
            epsilon = 1e-5
        );
        Ok(())
    }
}
