use std::ops::{Deref, DerefMut};

use approx::AbsDiffEq;
use ndarray::Array1;
use serde::{Deserialize, Serialize};
use tracing::{debug, trace};

use super::Optimizer;
use crate::core::{
    algorithm::estimation::Estimations,
    config::algorithm::Algorithm,
    data::shapes::{Residuals, SystemStatesAtStep},
    model::functional::{
        allpass::{
            from_coef_to_samples,
            shapes::{Coefs, Gains},
            APParameters,
        },
        measurement::MeasurementMatrixAtBeat,
        FunctionalDescription,
    },
};

/// Stuct to calculate and store the derivatives
/// of the model parameters with regards to the
/// Loss function.
#[allow(clippy::unsafe_derive_deserialize)]
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Derivatives {
    /// Derivatives of the All-pass gains
    pub gains: Gains,
    /// First moment of the gains derivatives
    pub gains_first_moment: Option<Gains>,
    /// second moment of the gains derivatives
    pub gains_second_moment: Option<Gains>,
    /// Derivatives of the All-pass coeficients
    pub coefs: Coefs,
    /// Sum of the absolute values of the coeficients derivatives
    pub coefs_abs_sum: Option<Coefs>,
    /// First moment of the coeficients derivatives
    pub coefs_first_moment: Option<Coefs>,
    /// Second moment of the coeficients derivatives
    pub coefs_second_moment: Option<Coefs>,
    pub step: usize,
    /// IIR component of the coeficients derivatives
    /// only used for internal computation
    pub coefs_iir: Gains,
    /// FIR component of the coeficients derivatives
    /// only used for internal computation
    pub coefs_fir: Gains,
    /// Residuals mapped onto the system states via
    /// the measurement matrix.
    /// Stored internally to avoid redundant computation
    pub mapped_residuals: MappedResiduals,
    /// Stored internally to avoid redundant computation
    pub average_delays_in_voxel: AverageDelays,
    pub maximum_regularization: MaximumRegularization,
    pub maximum_regularization_sum: f32,
}

impl Derivatives {
    /// Creates a new Derivatives struct with empty arrays initialized to
    /// the given number of states.
    #[must_use]
    #[tracing::instrument(level = "debug")]
    pub fn new(number_of_states: usize, optimizer: Optimizer) -> Self {
        debug!("Creating empty derivatives");
        let gains_first_moment = match optimizer {
            Optimizer::Sgd => None,
            Optimizer::Adam => Some(Gains::empty(number_of_states)),
        };
        let gains_second_moment = match optimizer {
            Optimizer::Sgd => None,
            Optimizer::Adam => Some(Gains::empty(number_of_states)),
        };
        let coefs_abs_sum = match optimizer {
            Optimizer::Sgd => {
                let mut coefs = Coefs::empty(number_of_states);
                coefs.fill(1.0);
                Some(coefs)
            }
            Optimizer::Adam => None,
        };
        let coefs_first_moment = match optimizer {
            Optimizer::Sgd => None,
            Optimizer::Adam => Some(Coefs::empty(number_of_states)),
        };
        let coefs_second_moment = match optimizer {
            Optimizer::Sgd => None,
            Optimizer::Adam => Some(Coefs::empty(number_of_states)),
        };
        Self {
            gains: Gains::empty(number_of_states),
            gains_first_moment,
            gains_second_moment,
            coefs: Coefs::empty(number_of_states),
            coefs_abs_sum,
            coefs_first_moment,
            coefs_second_moment,
            step: 1,
            coefs_iir: Gains::empty(number_of_states),
            coefs_fir: Gains::empty(number_of_states),
            mapped_residuals: MappedResiduals::new(number_of_states),
            average_delays_in_voxel: AverageDelays::new(number_of_states),
            maximum_regularization: MaximumRegularization::new(number_of_states),
            maximum_regularization_sum: 0.0,
        }
    }

    /// Sets all arrays to zero.
    ///
    /// Usually used after updating the parameters.
    #[inline]
    #[tracing::instrument(level = "debug")]
    pub fn reset(&mut self) {
        debug!("Resetting derivatives");
        self.gains.fill(0.0);
        self.coefs.fill(0.0);
        self.coefs_iir.fill(0.0);
        self.coefs_fir.fill(0.0);
        self.maximum_regularization.fill(0.0);
        self.maximum_regularization_sum = 0.0;
    }
}

/// Calculates the derivatives for the given time index.
///
/// CAUTION: adds to old values. use "reset" after using the
/// derivatives to update the parameters.
///
/// # Panics
///
/// Panics if `ap_params` is not set.
#[inline]
#[tracing::instrument(level = "debug", skip_all)]
pub fn calculate_step_derivatives(
    derivates: &mut Derivatives,
    estimations: &Estimations,
    functional_description: &FunctionalDescription,
    config: &Algorithm,
    step: usize,
    beat: usize,
    number_of_sensors: usize,
) {
    debug!("Calculating derivatives");
    calculate_mapped_residuals(
        &mut derivates.mapped_residuals,
        &estimations.residuals,
        &functional_description.measurement_matrix.at_beat(beat),
    );

    calculate_maximum_regularization(
        &mut derivates.maximum_regularization,
        &mut derivates.maximum_regularization_sum,
        &estimations.system_states.at_step(step),
        config.maximum_regularization_threshold,
    );

    if !config.freeze_gains {
        calculate_derivatives_gains(
            &mut derivates.gains,
            &estimations.ap_outputs,
            &derivates.maximum_regularization,
            &derivates.mapped_residuals,
            config.maximum_regularization_strength,
            number_of_sensors,
        );
    }
    if !config.freeze_delays {
        calculate_derivatives_coefs(derivates, estimations, functional_description, step, config);
    }
}

/// Calculates batch-wise derivatives.
///
/// CAUTION: adds to old values. use "reset" after using the
/// derivatives to update the parameters.
///
/// # Panics
///
/// Panics if `ap_params` is not set.
#[inline]
#[tracing::instrument(level = "debug", skip_all)]
pub fn calculate_batch_derivatives(
    derivates: &mut Derivatives,
    functional_description: &FunctionalDescription,
    config: &Algorithm,
) {
    if config.freeze_delays
        && config
            .smoothness_regularization_strength
            .abs_diff_ne(&0.0, f32::EPSILON)
    {
        calculate_average_delays(
            &mut derivates.average_delays_in_voxel,
            &functional_description.ap_params,
        );
        calculate_smoothness_derivatives(derivates, functional_description, config);
    }
}

#[allow(clippy::cast_precision_loss)]
#[tracing::instrument(level = "trace")]
fn calculate_smoothness_derivatives(
    derivates: &mut Derivatives,
    functional_description: &FunctionalDescription,
    config: &Algorithm,
) {
    derivates
        .coefs
        .indexed_iter_mut()
        .for_each(|((voxel_index, output_offset), derivative)| {
            let delay = functional_description.ap_params.delays[[voxel_index, output_offset]]
                as f32
                + from_coef_to_samples(
                    functional_description.ap_params.coefs[(voxel_index, output_offset)],
                );
            let mut average_delay = derivates.average_delays_in_voxel[voxel_index];
            let mut divisor = 1.0;
            for voxel_offset in 0..functional_description.ap_params.delays.shape()[1] {
                let neighbor_index = functional_description.ap_params.output_state_indices
                    [(voxel_index * 3, voxel_offset * 3)];
                if neighbor_index.is_none() {
                    continue;
                }
                let neighor_index = neighbor_index.unwrap() / 3;
                average_delay += derivates.average_delays_in_voxel[neighor_index];
                divisor += 1.0;
            }
            average_delay /= divisor;

            let difference = average_delay - delay;
            *derivative += config.smoothness_regularization_strength * difference;
        });
}

/// Calculates the derivatives for the allpass filter gains.
#[inline]
#[tracing::instrument(level = "trace")]
pub fn calculate_derivatives_gains(
    derivatives_gains: &mut Gains,
    ap_outputs: &Gains,
    maximum_regularization: &MaximumRegularization,
    mapped_residuals: &MappedResiduals,
    regularization_strength: f32,
    number_of_sensors: usize,
) {
    trace!("Calculating derivatives for gains");
    #[allow(clippy::cast_precision_loss)]
    let scaling = 1.0 / number_of_sensors as f32;
    #[allow(clippy::cast_precision_loss)]
    let regularization_scaling = regularization_strength;

    derivatives_gains
        .indexed_iter_mut()
        .zip(ap_outputs.iter())
        .for_each(|((gain_index, derivative), ap_output)| {
            let maximum_regularization = maximum_regularization[gain_index.0];

            *derivative += ap_output
                * mapped_residuals[gain_index.0]
                    .mul_add(scaling, maximum_regularization * regularization_scaling);
        });
}

/// Calculates the derivatives for the allpass filter coefficients.
///
/// This mutates the `self.coefs` values based on the provided `ap_outputs`,
/// `estimated_system_states`, `ap_params`, `time_index`, and `number_of_sensors`.
/// It calculates the FIR and IIR coefficient derivatives separately,
/// then combines them to update `self.coefs`.
#[inline]
#[tracing::instrument(level = "trace")]
pub fn calculate_derivatives_coefs(
    derivatives: &mut Derivatives,
    estimations: &Estimations,
    functional_description: &FunctionalDescription,
    step: usize,
    config: &Algorithm,
) {
    let scaling = 1.0 / estimations.measurements.num_sensors() as f32;

    // FIR derivatives calculation
    for state_index in 0..derivatives.coefs_fir.shape()[0] {
        for offset_index in 0..derivatives.coefs_fir.shape()[1] {
            let output_state = unsafe {
                functional_description
                    .ap_params
                    .output_state_indices
                    .uget((state_index, offset_index))
            };
            if output_state.is_none() {
                continue;
            }

            let coef_index = (state_index / 3, offset_index / 3);
            let delay = unsafe { functional_description.ap_params.delays.uget(coef_index) };
            let coef = unsafe { functional_description.ap_params.coefs.uget(coef_index) };

            if step >= *delay {
                let state_val = unsafe {
                    estimations
                        .system_states
                        .uget((step - delay, output_state.unwrap()))
                };
                let derivative =
                    unsafe { derivatives.coefs_fir.uget_mut((state_index, offset_index)) };
                *derivative = (1.0 - *coef).mul_add(*derivative, -*state_val);
            }
        }
    }

    // IIR derivatives calculation
    for state_index in 0..derivatives.coefs_iir.shape()[0] {
        for offset_index in 0..derivatives.coefs_iir.shape()[1] {
            let coef_index = (state_index / 3, offset_index / 3);
            let delay = unsafe { functional_description.ap_params.delays.uget(coef_index) };

            if step >= *delay {
                let coef = unsafe { functional_description.ap_params.coefs.uget(coef_index) };
                let ap_output = unsafe { estimations.ap_outputs.uget((state_index, offset_index)) };
                let derivative =
                    unsafe { derivatives.coefs_iir.uget_mut((state_index, offset_index)) };
                *derivative = (*coef).mul_add(*derivative, -*ap_output);
            }
        }
    }

    // Combine results
    for state_index in 0..derivatives.coefs_iir.shape()[0] {
        for offset_index in 0..derivatives.coefs_iir.shape()[1] {
            let coef_index = (state_index / 3, offset_index / 3);
            let delay = unsafe { *functional_description.ap_params.delays.uget(coef_index) } as f32
                + from_coef_to_samples(unsafe {
                    *functional_description.ap_params.coefs.uget(coef_index)
                });
            let delay_delta = (unsafe {
                *functional_description
                    .ap_params
                    .initial_delays
                    .uget(coef_index)
            } - delay)
                .powi(5);

            let iir = unsafe { derivatives.coefs_iir.uget((state_index, offset_index)) };
            let fir = unsafe { derivatives.coefs_fir.uget((state_index, offset_index)) };
            let ap_gain = unsafe {
                functional_description
                    .ap_params
                    .gains
                    .uget((state_index, offset_index))
            };
            let residual = unsafe { derivatives.mapped_residuals.uget(state_index) };

            let coef_derivative = unsafe { derivatives.coefs.uget_mut(coef_index) };
            *coef_derivative += ((fir + iir) * ap_gain * residual).mul_add(
                scaling,
                config.difference_regularization_strength * delay_delta,
            );
        }
    }
}

/// Calculates the maximum regularization for the given system states.
/// Iterates through the states, calculates the sum of the absolute values,
/// compares to the threshold, and calculates & assigns maximum regularization
/// accordingly.
#[inline]
#[tracing::instrument(level = "trace", skip_all)]
pub fn calculate_maximum_regularization(
    maximum_regularization: &mut MaximumRegularization,
    maximum_regularization_sum: &mut f32,
    system_states: &SystemStatesAtStep,
    regularization_threshold: f32,
) {
    trace!("Calculating maximum regularization");
    // self.maximum_regularization_sum = 0.0; // This is probably wrong, no?
    for state_index in (0..system_states.raw_dim()[0]).step_by(3) {
        let sum = system_states[[state_index]].abs()
            + system_states[[state_index + 1]].abs()
            + system_states[[state_index + 2]].abs();
        if sum > regularization_threshold {
            let factor = sum - regularization_threshold;
            *maximum_regularization_sum += factor.powi(2);
            maximum_regularization[state_index] = factor * system_states[[state_index]].signum();
            maximum_regularization[state_index + 1] =
                factor * system_states[[state_index + 1]].signum();
            maximum_regularization[state_index + 2] =
                factor * system_states[[state_index + 2]].signum();
        } else {
            maximum_regularization[state_index] = 0.0;
            maximum_regularization[state_index + 1] = 0.0;
            maximum_regularization[state_index + 2] = 0.0;
        }
    }
}
#[inline]
#[tracing::instrument(level = "trace", skip_all)]
pub fn calculate_mapped_residuals(
    mapped_residuals: &mut MappedResiduals,
    residuals: &Residuals,
    measurement_matrix: &MeasurementMatrixAtBeat,
) {
    trace!("Calculating mapped residuals");
    ndarray::linalg::general_mat_mul(
        1.0,
        &measurement_matrix.t(),
        &residuals.view().insert_axis(ndarray::Axis(1)),
        0.0,
        &mut mapped_residuals.view_mut().insert_axis(ndarray::Axis(1)),
    );
}
#[allow(clippy::cast_precision_loss)]
#[tracing::instrument(level = "trace", skip_all)]
pub fn calculate_average_delays(average_delays: &mut AverageDelays, ap_params: &APParameters) {
    trace!("Calculating average delays");
    average_delays
        .indexed_iter_mut()
        .for_each(|(voxel_index, average_delay)| {
            let mut delay_sum = 0.0;
            let mut gain_sum = 0.0;
            for offset in 0..ap_params.delays.shape()[1] {
                let delay = ap_params.delays[[voxel_index, offset]] as f32
                    + from_coef_to_samples(ap_params.coefs[[voxel_index, offset]]);
                for input_dimension in 0..3 {
                    for output_dimension in 0..3 {
                        let gain = ap_params.gains[[
                            voxel_index * 3 + input_dimension,
                            offset * 3 + output_dimension,
                        ]];
                        delay_sum += gain * delay;
                        gain_sum += gain;
                    }
                }
            }
            *average_delay = delay_sum / gain_sum;
        });
}

/// Shape for the mapped residuals.
///
/// Has dimensions (`number_of_states`)
///
/// The residuals (measurements) of the state estimation
/// get mapped onto the system states.
/// These values are then used for the calcualtion of the derivatives
///
/// The mapped residuals are calculated as
/// `H_T` * y
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct MappedResiduals(Array1<f32>);

impl MappedResiduals {
    #[must_use]
    #[tracing::instrument(level = "trace")]
    pub fn new(number_of_states: usize) -> Self {
        trace!("Creating ArrayMappedResiduals");
        Self(Array1::zeros(number_of_states))
    }
}

impl Deref for MappedResiduals {
    type Target = Array1<f32>;

    #[tracing::instrument(level = "trace")]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for MappedResiduals {
    #[tracing::instrument(level = "trace")]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Shape for the average delays in each voxel.
///
/// Has dimensions (`number_of_states / 3`)
///
/// The average delays are calculated as a
/// weighted sum of the delays by the gains in that direction.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct AverageDelays(Array1<f32>);

impl AverageDelays {
    #[must_use]
    #[tracing::instrument(level = "trace")]
    pub fn new(number_of_states: usize) -> Self {
        trace!("Creating AverageDelays");
        Self(Array1::zeros(number_of_states / 3))
    }
}

impl Deref for AverageDelays {
    type Target = Array1<f32>;

    #[tracing::instrument(level = "trace")]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for AverageDelays {
    #[tracing::instrument(level = "trace")]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Shape for the maximum system states regularization.
///
/// Has dimensions (`number_of_states`)
///
/// The maximum current density in a single voxel should not exceed one.
/// For this we have to add up all three absoutle values of
/// components in each voxel.
/// If this sum is greater than one, the system state get's copied into
/// this array. Otherwise the component get's set to zero.
///
/// You can think about it like a kind of relu activation.
/// Only if all three components added up are greater than one,
/// do we want to dercease the components, otherwise the
/// magnitude should not influence the loss and therefore
/// the derivatives.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct MaximumRegularization(Array1<f32>);

impl MaximumRegularization {
    #[must_use]
    #[tracing::instrument(level = "trace")]
    pub fn new(number_of_states: usize) -> Self {
        trace!("Creating ArrayMaximumRegularization");
        Self(Array1::zeros(number_of_states))
    }
}

impl Deref for MaximumRegularization {
    type Target = Array1<f32>;

    #[tracing::instrument(level = "trace")]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for MaximumRegularization {
    #[tracing::instrument(level = "trace")]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[cfg(test)]
mod tests {
    use ndarray::Dim;

    use super::*;
    use crate::core::{
        algorithm::estimation::Estimations, model::functional::FunctionalDescription,
    };
    #[test]
    fn coef_no_crash() {
        let number_of_steps = 2000;
        let number_of_states = 3000;
        let number_of_sensors = 10;
        let number_of_beats = 1;
        let step = 10;
        let mut derivatives = Derivatives::new(number_of_states, Optimizer::Sgd);
        let estimations = Estimations::empty(
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
            Dim([1000, 1, 1]),
        );
        let config = Algorithm {
            maximum_regularization_strength: 0.0,
            smoothness_regularization_strength: 0.0,
            ..Default::default()
        };

        calculate_derivatives_coefs(
            &mut derivatives,
            &estimations,
            &functional_description,
            step,
            &config,
        );
    }

    #[test]
    fn calculate_no_crash() {
        let number_of_states = 1500;
        let number_of_sensors = 300;
        let number_of_steps = 2000;
        let number_of_beats = 10;
        let step = 333;
        let voxels_in_dims = Dim([1000, 1, 1]);
        let config = Algorithm {
            maximum_regularization_strength: 0.0,
            smoothness_regularization_strength: 0.0,
            ..Default::default()
        };

        let mut derivates = Derivatives::new(number_of_states, config.optimizer);
        let functional_description = FunctionalDescription::empty(
            number_of_states,
            number_of_sensors,
            number_of_steps,
            number_of_beats,
            voxels_in_dims,
        );
        let estimations = Estimations::empty(
            number_of_states,
            number_of_sensors,
            number_of_steps,
            number_of_beats,
        );

        calculate_step_derivatives(
            &mut derivates,
            &estimations,
            &functional_description,
            &config,
            step,
            0,
            estimations.measurements.num_sensors(),
        );
    }
}
