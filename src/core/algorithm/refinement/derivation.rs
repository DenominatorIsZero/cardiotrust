use std::ops::{Deref, DerefMut};

use ndarray::Array1;
use serde::{Deserialize, Serialize};
use tracing::{debug, trace};

use crate::core::{
    config::algorithm::Algorithm,
    data::shapes::{Residuals, SystemStates, SystemStatesAtStep},
    model::functional::{
        allpass::{
            shapes::{Coefs, Gains},
            APParameters,
        },
        measurement::MeasurementMatrixAtBeat,
    },
};

use super::Optimizer;

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
            coefs_first_moment,
            coefs_second_moment,
            step: 1,
            coefs_iir: Gains::empty(number_of_states),
            coefs_fir: Gains::empty(number_of_states),
            mapped_residuals: MappedResiduals::new(number_of_states),
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
        self.mapped_residuals.fill(0.0);
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
pub fn calculate_derivatives(
    derivatives_gains: &mut Gains,
    derivatives_coefs: &mut Coefs,
    derivatives_iir: &mut Gains,
    derivatives_fir: &mut Gains,
    mapped_residuals: &mut MappedResiduals,
    maximum_regularization: &mut MaximumRegularization,
    maximum_regularization_sum: &mut f32,
    residuals: &Residuals,
    estimated_system_states: &SystemStates,
    ap_outputs: &Gains,
    ap_params: &APParameters,
    measurement_matrix: &MeasurementMatrixAtBeat,
    config: &Algorithm,
    step: usize,
    number_of_sensors: usize,
) {
    debug!("Calculating derivatives");
    calculate_mapped_residuals(mapped_residuals, residuals, measurement_matrix);

    calculate_maximum_regularization(
        maximum_regularization,
        maximum_regularization_sum,
        &estimated_system_states.at_step(step),
        config.regularization_threshold,
    );

    if !config.freeze_gains {
        calculate_derivatives_gains(
            derivatives_gains,
            ap_outputs,
            maximum_regularization,
            mapped_residuals,
            config.regularization_strength,
            number_of_sensors,
        );
    }
    if !config.freeze_delays {
        calculate_derivatives_coefs(
            derivatives_coefs,
            derivatives_iir,
            derivatives_fir,
            ap_outputs,
            estimated_system_states,
            ap_params,
            mapped_residuals,
            step,
            number_of_sensors,
        );
    }
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
    derivatives_coefs: &mut Coefs,
    derivatives_iir: &mut Gains,
    derivatives_fir: &mut Gains,
    ap_outputs: &Gains,
    estimated_system_states: &SystemStates,
    ap_params: &APParameters,
    mapped_residuals: &MappedResiduals,
    step: usize,
    number_of_sensors: usize,
) {
    trace!("Calculating derivatives for coefficients");
    derivatives_fir
        .indexed_iter_mut()
        .zip(ap_params.output_state_indices.iter())
        .filter(|(_, output_state_index)| output_state_index.is_some())
        .for_each(
            |(((state_index, offset_index), derivative), output_state_index)| {
                let coef_index = (state_index / 3, offset_index / 3);
                if step >= ap_params.delays[coef_index] {
                    *derivative = ap_params.coefs[coef_index].mul_add(
                        *derivative,
                        estimated_system_states[(
                            step - ap_params.delays[coef_index],
                            output_state_index.unwrap(),
                        )],
                    );
                }
            },
        );
    derivatives_iir
        .indexed_iter_mut()
        .zip(ap_outputs.iter())
        .for_each(|(((state_index, offset_index), derivative), ap_output)| {
            let coef_index = (state_index / 3, offset_index / 3);
            *derivative = ap_params.coefs[coef_index].mul_add(*derivative, *ap_output);
        });
    #[allow(clippy::cast_precision_loss)]
    let scaling = 1.0 / number_of_sensors as f32;
    #[allow(clippy::cast_precision_loss)]
    derivatives_iir
        .indexed_iter()
        .zip(derivatives_fir.iter())
        .zip(ap_params.gains.iter())
        .for_each(|((((state_index, offset_index), iir), fir), ap_gain)| {
            let coef_index = (state_index / 3, offset_index / 3);
            derivatives_coefs[coef_index] +=
                (fir + iir) * ap_gain * mapped_residuals[state_index] * scaling;
        });
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
    mapped_residuals.assign(&measurement_matrix.t().dot(&**residuals));
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

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for MappedResiduals {
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

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for MaximumRegularization {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[cfg(test)]
mod tests {
    use ndarray::Dim;

    use crate::core::{
        algorithm::estimation::Estimations,
        model::functional::{
            allpass::shapes::{Indices, UnitDelays},
            FunctionalDescription,
        },
    };

    use super::*;
    #[test]
    fn coef_no_crash() {
        let number_of_steps = 2000;
        let number_of_states = 3000;
        let ap_outputs = Gains::empty(number_of_states);
        let estimated_system_states = SystemStates::empty(number_of_steps, number_of_states);
        let ap_params = APParameters::empty(number_of_states, Dim([1000, 1, 1]));
        let mut delays = UnitDelays::empty(number_of_states);
        delays.fill(30);
        let mut output_state_indices = Indices::empty(number_of_states);
        output_state_indices.fill(Some(3));
        let step = 10;

        let mut derivatives = Derivatives::new(number_of_states, Optimizer::Sgd);

        calculate_derivatives_coefs(
            &mut derivatives.coefs,
            &mut derivatives.coefs_iir,
            &mut derivatives.coefs_fir,
            &ap_outputs,
            &estimated_system_states,
            &ap_params,
            &derivatives.mapped_residuals,
            step,
            1,
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
            regularization_strength: 0.0,
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

        calculate_derivatives(
            &mut derivates.gains,
            &mut derivates.coefs,
            &mut derivates.coefs_iir,
            &mut derivates.coefs_fir,
            &mut derivates.mapped_residuals,
            &mut derivates.maximum_regularization,
            &mut derivates.maximum_regularization_sum,
            &estimations.residuals,
            &estimations.system_states,
            &estimations.ap_outputs,
            &functional_description.ap_params,
            &functional_description.measurement_matrix.at_beat(0),
            &config,
            step,
            estimations.measurements.num_sensors(),
        );
    }
}
