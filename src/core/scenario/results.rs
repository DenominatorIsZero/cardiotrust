use ocl::Queue;
use serde::{Deserialize, Serialize};
use tracing::{debug, trace};

use super::algorithm::metrics::Metrics;
use crate::core::{
    algorithm::{
        estimation::{Estimations, EstimationsGPU},
        metrics::MetricsGPU,
        refinement::{
            derivation::{Derivatives, DerivativesGPU},
            Optimizer,
        },
    },
    config::algorithm::Algorithm,
    model::{functional::FunctionalDescription, Model, ModelGPU},
};

/// Results contains the outputs from running a scenario.
///
/// This includes metrics, estimations, derivatives, snapshots,
/// the model, etc. It is returned after running the scenario.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Results {
    pub metrics: Metrics,
    pub estimations: Estimations,
    pub derivatives: Derivatives,
    pub snapshots: Vec<Snapshot>,
    pub model: Option<Model>,
}

pub struct ResultsGPU {
    pub metrics: MetricsGPU,
    pub estimations: EstimationsGPU,
    pub derivatives: DerivativesGPU,
    pub model: ModelGPU,
}

#[allow(
    clippy::useless_let_if_seq,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)]
impl Results {
    /// Creates a new Results instance with empty estimations, derivatives,
    /// snapshots, and model. The metrics are initialized based on the provided
    /// number of epochs, steps, sensors, and states.
    #[must_use]
    #[tracing::instrument(level = "debug")]
    pub fn new(
        number_of_epochs: usize,
        number_of_steps: usize,
        number_of_sensors: usize,
        number_of_states: usize,
        number_of_beats: usize,
        batch_size: usize,
        optimizer: Optimizer,
    ) -> Self {
        debug!("Creating results with empty estimations, derivatives, snapshots, and model");
        let estimations = Estimations::empty(
            number_of_states,
            number_of_sensors,
            number_of_steps,
            number_of_beats,
        );
        let derivatives = Derivatives::new(number_of_states, optimizer);
        let snapshots = Vec::new();
        let batch_size = if batch_size > 0 {
            batch_size
        } else {
            number_of_beats
        };

        let number_of_batches = (number_of_beats as f32 / batch_size as f32).ceil() as usize;

        Self {
            metrics: Metrics::new(number_of_epochs, number_of_steps, number_of_batches),
            estimations,
            derivatives,
            model: None,
            snapshots,
        }
    }

    /// Saves the metrics, estimations, and model as .npy files to the given path.
    #[tracing::instrument(level = "trace")]
    pub(crate) fn save_npy(&self, path: &std::path::Path) {
        trace!("Saving results to.npy files");
        self.metrics.save_npy(&path.join("metrics"));
        self.estimations.save_npy(&path.join("estimations"));
        self.model.as_ref().unwrap().save_npy(&path.join("model"));
    }

    #[allow(clippy::missing_panics_doc)]
    #[must_use]
    pub fn to_gpu(&self, queue: &Queue) -> ResultsGPU {
        ResultsGPU {
            metrics: self.metrics.to_gpu(queue),
            estimations: self.estimations.to_gpu(queue),
            derivatives: self.derivatives.to_gpu(queue),
            model: self.model.as_ref().unwrap().to_gpu(queue),
        }
    }

    #[allow(clippy::missing_panics_doc)]
    pub fn update_from_gpu(&mut self, results: &ResultsGPU) {
        self.metrics.update_from_gpu(&results.metrics);
        self.estimations.update_from_gpu(&results.estimations);
        self.derivatives.update_from_gpu(&results.derivatives);
        self.model.as_mut().unwrap().from_gpu(&results.model);
    }

    #[allow(dead_code)]
    #[must_use]
    pub fn get_default() -> Self {
        let model = Model::get_default();
        let algorithm_config = Algorithm::default();
        Self {
            metrics: Metrics::new(
                algorithm_config.epochs,
                model.functional_description.control_function_values.len(),
                algorithm_config.epochs,
            ),
            estimations: Estimations::empty(
                model.spatial_description.voxels.count_states(),
                model.spatial_description.sensors.count(),
                model.functional_description.control_function_values.len(),
                model.functional_description.measurement_matrix.shape()[0],
            ),
            derivatives: Derivatives::new(
                model.spatial_description.voxels.count_states(),
                Optimizer::default(),
            ),
            model: Some(model),
            snapshots: Vec::new(),
        }
    }
}

/// Snapshot contains estimations and functional description at a point in time.
/// Used to capture model state during scenario execution.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub estimations: Estimations,
    pub functional_description: FunctionalDescription,
}

impl Snapshot {
    #[must_use]
    /// Creates a new Snapshot instance with the provided estimations and
    /// functional description.
    #[tracing::instrument(level = "trace")]
    pub fn new(estimations: &Estimations, functional_description: FunctionalDescription) -> Self {
        trace!("Creating snapshot with estimations and functional description");
        Self {
            estimations: estimations.clone(),
            functional_description,
        }
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;
    use ocl::{Kernel, Program};

    use super::*;
    use crate::core::algorithm::gpu::GPU;
    #[test]
    #[allow(clippy::cast_precision_loss, clippy::similar_names)]
    fn test_results_gpu_transfer() {
        let mut results_from_cpu = Results::get_default();
        let gpu = GPU::new();
        let results_gpu = results_from_cpu.to_gpu(&gpu.queue);

        // Create and build the modification kernel
        let kernel_src = r"
    __kernel void modify_results(__global float* ap_outputs_now) {
    int index_state = get_global_id(0);
    int index_offset = get_global_id(1);
        ap_outputs_now[index_state*78+index_offset] = index_state*78+index_offset;  
    }
";

        let program = Program::builder()
            .src(kernel_src)
            .build(&gpu.context)
            .unwrap();

        let kernel = Kernel::builder()
            .program(&program)
            .name("modify_results")
            .queue(gpu.queue)
            .global_work_size([
                results_from_cpu.estimations.ap_outputs_now.shape()[0],
                results_from_cpu.estimations.ap_outputs_now.shape()[1],
            ])
            .arg_named("ap_outputs_now", &results_gpu.estimations.ap_outputs_now)
            .build()
            .unwrap();

        unsafe {
            kernel.enq().unwrap();
        }

        for index_state in 0..results_from_cpu
            .model
            .as_ref()
            .unwrap()
            .spatial_description
            .voxels
            .count_states()
        {
            for index_offset in 0..78 {
                results_from_cpu.estimations.ap_outputs_now[[index_state, index_offset]] =
                    (index_state as f32).mul_add(78.0, index_offset as f32);
            }
        }

        let mut results_from_gpu = results_from_cpu.clone();
        results_from_gpu.update_from_gpu(&results_gpu);

        assert_relative_eq!(
            results_from_gpu
                .estimations
                .ap_outputs_now
                .as_slice()
                .unwrap(),
            results_from_cpu
                .estimations
                .ap_outputs_now
                .as_slice()
                .unwrap(),
            epsilon = 1e-6
        );
        assert_eq!(results_from_cpu, results_from_gpu);
    }
}
