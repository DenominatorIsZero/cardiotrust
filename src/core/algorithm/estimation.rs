pub mod prediction;

use ocl::Buffer;
use serde::{Deserialize, Serialize};
use tracing::{debug, trace};

use super::refinement::derivation::AverageDelays;
use crate::core::{
    data::{
        shapes::{
            ActivationTimePerStateMs, Measurements, Residuals, SystemStates, SystemStatesSpherical, SystemStatesSphericalMax,
        },
        Data,
    },
    model::functional::allpass::{
            from_coef_to_samples,
            shapes::{Coefs, Gains, UnitDelays},
        },
};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Estimations {
    pub ap_outputs_now: Gains,
    pub ap_outputs_last: Gains,
    pub system_states: SystemStates,
    pub system_states_spherical: SystemStatesSpherical,
    pub system_states_spherical_max: SystemStatesSphericalMax,
    pub activation_times: ActivationTimePerStateMs,
    pub measurements: Measurements,
    pub residuals: Residuals,
    pub system_states_spherical_max_delta: SystemStatesSphericalMax,
    pub activation_times_delta: ActivationTimePerStateMs,
    pub average_delays: AverageDelays,
}

pub struct EstimationsGPU {
    pub ap_outputs_now: Buffer<f32>,
    pub ap_outputs_last: Buffer<f32>,
    pub system_states: Buffer<f32>,
    pub measurements: Buffer<f32>,
    pub residuals: Buffer<f32>,
    pub step: Buffer<i32>,
    pub beat: Buffer<i32>,
    pub epoch: Buffer<i32>,
}

impl Estimations {
    /// Creates a new empty Estimations struct with the given dimensions.
    #[must_use]
    #[tracing::instrument(level = "debug")]
    pub fn empty(
        number_of_states: usize,
        number_of_sensors: usize,
        number_of_steps: usize,
        number_of_beats: usize,
    ) -> Self {
        debug!("Creating empty estimations");
        Self {
            ap_outputs_now: Gains::empty(number_of_states),
            ap_outputs_last: Gains::empty(number_of_states),
            system_states: SystemStates::empty(number_of_steps, number_of_states),
            system_states_spherical: SystemStatesSpherical::empty(
                number_of_steps,
                number_of_states,
            ),
            system_states_spherical_max: SystemStatesSphericalMax::empty(number_of_states),
            activation_times: ActivationTimePerStateMs::empty(number_of_states),
            measurements: Measurements::empty(number_of_beats, number_of_steps, number_of_sensors),
            residuals: Residuals::empty(number_of_sensors),
            system_states_spherical_max_delta: SystemStatesSphericalMax::empty(number_of_states),
            activation_times_delta: ActivationTimePerStateMs::empty(number_of_states),
            average_delays: AverageDelays::empty(number_of_states),
        }
    }

    /// Resets all the internal state of the Estimations struct by filling the
    /// underlying data structures with 0.0. This is done to prepare for a new
    /// epoch.
    #[tracing::instrument(level = "debug")]
    pub fn reset(&mut self) {
        debug!("Resetting estimations");
        self.system_states.fill(0.0);
        self.ap_outputs_now.fill(0.0);
    }

    /// Saves the system states and measurements to .npy files at the given path.
    /// The filenames will be automatically generated based on the struct field names.
    #[tracing::instrument(level = "trace")]
    pub(crate) fn save_npy(&self, path: &std::path::Path) {
        trace!("Saving estimations to npy files");
        self.system_states.save_npy(path);
        self.measurements.save_npy(path);
    }

    #[tracing::instrument(level = "trace", skip_all)]
    pub(crate) fn to_gpu(&self, queue: &ocl::Queue) -> EstimationsGPU {
        EstimationsGPU {
            ap_outputs_now: self.ap_outputs_now.to_gpu(queue),
            ap_outputs_last: self.ap_outputs_last.to_gpu(queue),
            system_states: self.system_states.to_gpu(queue),
            measurements: self.measurements.to_gpu(queue),
            residuals: self.residuals.to_gpu(queue),
            step: ocl::Buffer::builder()
                .queue(queue.clone())
                .len(1)
                .copy_host_slice(&[0])
                .build()
                .unwrap(),
            beat: ocl::Buffer::builder()
                .queue(queue.clone())
                .len(1)
                .copy_host_slice(&[0])
                .build()
                .unwrap(),
            epoch: ocl::Buffer::builder()
                .queue(queue.clone())
                .len(1)
                .copy_host_slice(&[0])
                .build()
                .unwrap(),
        }
    }

    #[tracing::instrument(level = "trace", skip_all)]
    pub(crate) fn update_from_gpu(&mut self, estimations: &EstimationsGPU) {
        self.ap_outputs_now
            .update_from_gpu(&estimations.ap_outputs_now);
        self.ap_outputs_last
            .update_from_gpu(&estimations.ap_outputs_last);
        self.system_states
            .update_from_gpu(&estimations.system_states);
        self.measurements.update_from_gpu(&estimations.measurements);
        self.residuals.update_from_gpu(&estimations.residuals);
    }
}

/// Calculates the residuals between the predicted and actual measurements for the given time index.
/// The residuals are stored in the provided `residuals` array.
#[inline]
#[tracing::instrument(level = "trace", skip_all)]
pub fn calculate_residuals(estimations: &mut Estimations, data: &Data, beat: usize, step: usize) {
    trace!("Calculating residuals");
    estimations.residuals.assign(
        &(&*estimations.measurements.at_beat(beat).at_step(step)
            - &*data.simulation.measurements.at_beat(beat).at_step(step)),
    );
}

/// Calculates the delta between the estimated gains and the actual gains.  
/// The delta is stored in the provided `gains_delta` array.
#[inline]
#[tracing::instrument(level = "trace")]
pub fn calculate_gains_delta(
    gains_delta: &mut Gains,
    estimated_gains: &Gains,
    actual_gains: &Gains,
) {
    trace!("Calculating gains delta");
    gains_delta.assign(&(&**estimated_gains - &**actual_gains));
}

/// Calculates the delta between the estimated delays and actual delays.
/// The delta is stored in the provided `delays_delta` array.
#[inline]
#[tracing::instrument(level = "trace")]
pub fn calculate_delays_delta(
    delays_delta: &mut Coefs,
    estimated_delays: &UnitDelays,
    actual_delays: &UnitDelays,
    estimated_coefs: &Coefs,
    actual_coefs: &Coefs,
) {
    trace!("Calculating delays delta");
    #[allow(clippy::cast_precision_loss)]
    delays_delta
        .indexed_iter_mut()
        .for_each(|(index, delay_delta)| {
            *delay_delta = (estimated_delays[index] as f32 - actual_delays[index] as f32)
                + (from_coef_to_samples(estimated_coefs[index])
                    - from_coef_to_samples(actual_coefs[index]));
        });
}

#[cfg(test)]
mod tests {
    use ndarray::Dim;

    use super::{calculate_residuals, prediction::calculate_system_prediction, Estimations};
    use crate::core::{
        data::Data, model::functional::FunctionalDescription,
    };

    #[test]
    fn prediction_no_crash() {
        let number_of_states = 3000;
        let number_of_sensors = 300;
        let number_of_steps = 2000;
        let number_of_beats = 10;
        let step = 333;
        let beat = 4;
        let voxels_in_dims = Dim([1000, 1, 1]);

        let mut estimations = Estimations::empty(
            number_of_states,
            number_of_sensors,
            number_of_steps,
            number_of_beats,
        );
        let functional_description = FunctionalDescription::empty(
            number_of_states,
            number_of_sensors,
            number_of_steps,
            number_of_beats,
            voxels_in_dims,
        );

        calculate_system_prediction(&mut estimations, &functional_description, beat, step);
    }

    #[test]
    fn residuals_no_crash() {
        let number_of_sensors = 300;
        let number_of_states = 3000;
        let voxels_in_dims = Dim([1000, 1, 1]);
        let number_of_steps = 2000;
        let number_of_beats = 10;
        let step = 333;
        let beat = 2;

        let mut estimations = Estimations::empty(
            number_of_states,
            number_of_sensors,
            number_of_steps,
            number_of_beats,
        );
        let data = Data::empty(
            number_of_sensors,
            number_of_states,
            number_of_steps,
            voxels_in_dims,
            number_of_beats,
        );

        calculate_residuals(&mut estimations, &data, beat, step);
    }
}
