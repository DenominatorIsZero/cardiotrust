pub mod shapes;
#[cfg(test)]
mod tests;

use anyhow::{Context, Result};
use approx::AbsDiffEq;
use ocl::Buffer;
use serde::{Deserialize, Serialize};
pub use shapes::{AverageDelays, MappedResiduals, MaximumRegularization};
use tracing::{debug, trace};

use super::Optimizer;
use crate::core::{
    algorithm::estimation::Estimations,
    config::algorithm::{APDerivative, Algorithm},
    data::shapes::{Residuals, SystemStatesAtStep},
    model::functional::{
        allpass::{
            delay_index_to_offset, from_coef_to_samples,
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
    pub maximum_regularization: MaximumRegularization,
    pub maximum_regularization_sum: f32,
}

pub struct DerivativesGPU {
    pub gains: Buffer<f32>,
    pub coefs: Buffer<f32>,
    pub coefs_iir: Buffer<f32>,
    pub coefs_fir: Buffer<f32>,
    pub mapped_residuals: Buffer<f32>,
    pub maximum_regularization: Buffer<f32>,
    pub maximum_regularization_sum: Buffer<f32>,
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
        self.maximum_regularization.fill(0.0);
        self.maximum_regularization_sum = 0.0;
    }

    #[tracing::instrument(level = "trace", skip_all)]
    pub(crate) fn to_gpu(&self, queue: &ocl::Queue) -> Result<DerivativesGPU> {
        Ok(DerivativesGPU {
            gains: self.gains.to_gpu(queue)?,
            coefs: self.coefs.to_gpu(queue)?,
            coefs_iir: self.coefs_iir.to_gpu(queue)?,
            coefs_fir: self.coefs_fir.to_gpu(queue)?,
            mapped_residuals: self.mapped_residuals.to_gpu(queue)?,
            maximum_regularization: self.maximum_regularization.to_gpu(queue)?,
            maximum_regularization_sum: ocl::Buffer::builder()
                .queue(queue.clone())
                .len(1)
                .copy_host_slice(&[self.maximum_regularization_sum])
                .build()
                .context("Failed to create maximum_regularization_sum buffer")?,
        })
    }

    #[tracing::instrument(level = "trace", skip_all)]
    pub(crate) fn update_from_gpu(&mut self, derivatives: &DerivativesGPU) -> Result<()> {
        self.gains.update_from_gpu(&derivatives.gains)?;
        self.coefs.update_from_gpu(&derivatives.coefs)?;
        self.coefs_iir.update_from_gpu(&derivatives.coefs_iir)?;
        self.coefs_fir.update_from_gpu(&derivatives.coefs_fir)?;
        self.mapped_residuals
            .update_from_gpu(&derivatives.mapped_residuals)?;
        self.maximum_regularization
            .update_from_gpu(&derivatives.maximum_regularization)?;
        let mut maximum_regularization_sum = vec![0.0f32];
        derivatives
            .maximum_regularization_sum
            .read(&mut maximum_regularization_sum)
            .enq()
            .context("Failed to read maximum_regularization_sum from GPU")?;
        self.maximum_regularization_sum = maximum_regularization_sum[0];
        Ok(())
    }
}

/// Calculates the derivatives for the given time index.
///
/// CAUTION: adds to old values. use "reset" after using the
/// derivatives to update the parameters.
///
/// # Errors
///
/// Returns an error if algorithm parameters are not properly initialized.
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
) -> Result<()> {
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
            &estimations.ap_outputs_now,
            &derivates.maximum_regularization,
            &derivates.mapped_residuals,
            config,
            number_of_sensors,
        );
    }
    if !config.freeze_delays {
        match config.ap_derivative {
            APDerivative::Simple => {
                calculate_derivatives_coefs_simple(
                    derivates,
                    estimations,
                    functional_description,
                    step,
                    config,
                )?;
            }
            APDerivative::Textbook => {
                calculate_derivatives_coefs_textbook(
                    derivates,
                    estimations,
                    functional_description,
                    step,
                    config,
                )?;
            }
        }
    }
    Ok(())
}

/// Calculates batch-wise derivatives.
///
/// CAUTION: adds to old values. use "reset" after using the
/// derivatives to update the parameters.
///
/// # Errors
///
/// Returns an error if algorithm parameters are not properly initialized.
#[inline]
#[tracing::instrument(level = "debug", skip_all)]
pub fn calculate_batch_derivatives(
    derivatives: &mut Derivatives,
    estimations: &Estimations,
    functional_description: &FunctionalDescription,
    config: &Algorithm,
) -> Result<()> {
    debug!("Calculating batch derivatives");
    if !config.freeze_delays
        && config
            .smoothness_regularization_strength
            .abs_diff_ne(&0.0, f32::EPSILON)
    {
        calculate_smoothness_derivatives(derivatives, estimations, functional_description, config)?;
    }
    Ok(())
}

#[allow(clippy::cast_precision_loss)]
#[tracing::instrument(level = "trace")]
pub fn calculate_smoothness_derivatives(
    derivatives: &mut Derivatives,
    estimations: &Estimations,
    functional_description: &FunctionalDescription,
    config: &Algorithm,
) -> Result<()> {
    debug!("Calculating smoothness derivatives");
    for voxel_index in 0..derivatives.coefs.shape()[0] {
        for output_offset in 0..derivatives.coefs.shape()[1] {
            let average_delay_in_voxel = unsafe { *estimations.average_delays.uget(voxel_index) };
            let Some(average_delay_in_voxel) = average_delay_in_voxel else {
                continue;
            };
            let mut average_delay_in_neighborhood = average_delay_in_voxel;
            let mut divisor = 1.0;

            for voxel_offset in 0..functional_description.ap_params.delays.shape()[1] {
                let neighbor_index = unsafe {
                    functional_description
                        .ap_params
                        .output_state_indices
                        .uget((voxel_index * 3, voxel_offset * 3))
                };
                let Some(neighbor_index) = neighbor_index else {
                    continue;
                };
                let neighbor_index = neighbor_index / 3;
                let delay = unsafe { *estimations.average_delays.uget(neighbor_index) };
                if let Some(delay) = delay {
                    average_delay_in_neighborhood += delay;
                    divisor += 1.0;
                }
            }
            average_delay_in_neighborhood /= divisor;

            let difference = average_delay_in_neighborhood - average_delay_in_voxel;

            let derivative = unsafe { derivatives.coefs.uget_mut((voxel_index, output_offset)) };
            *derivative += config.smoothness_regularization_strength * difference;
        }
    }
    Ok(())
}

/// Calculates the derivatives for the allpass filter gains.
#[inline]
#[allow(clippy::cast_precision_loss)]
#[tracing::instrument(level = "trace")]
pub fn calculate_derivatives_gains(
    derivatives_gains: &mut Gains,
    ap_outputs: &Gains,
    maximum_regularization: &MaximumRegularization,
    mapped_residuals: &MappedResiduals,
    config: &Algorithm,
    number_of_sensors: usize,
) {
    let mse_scaling = 1.0 / number_of_sensors as f32 * config.mse_strength;
    let regularization_scaling = config.maximum_regularization_strength;

    for gain_index in 0..derivatives_gains.shape()[0] {
        for offset_index in 0..derivatives_gains.shape()[1] {
            let ap_output = unsafe { ap_outputs.uget((gain_index, offset_index)) };
            let max_reg = unsafe { maximum_regularization.uget(gain_index) };
            let residual = unsafe { mapped_residuals.uget(gain_index) };
            let derivative = unsafe { derivatives_gains.uget_mut((gain_index, offset_index)) };

            *derivative +=
                ap_output * residual.mul_add(mse_scaling, max_reg * regularization_scaling);
        }
    }
}

/// Calculates the derivatives for the allpass filter coefficients using a simplified form for the AP derivative.
///
/// # Errors
///
/// Returns an error if algorithm parameters are not properly initialized.
#[inline]
#[allow(clippy::cast_precision_loss)]
#[tracing::instrument(level = "trace")]
pub fn calculate_derivatives_coefs_simple(
    derivatives: &mut Derivatives,
    estimations: &Estimations,
    functional_description: &FunctionalDescription,
    step: usize,
    config: &Algorithm,
) -> Result<()> {
    let mse_scaling = 1.0 / estimations.measurements.num_sensors() as f32 * config.mse_strength;
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
            let delay = unsafe { functional_description.ap_params.delays.uget(coef_index) };
            let output_state = unsafe {
                functional_description
                    .ap_params
                    .output_state_indices
                    .uget((state_index, offset_index))
            };
            if output_state.is_none() {
                continue;
            }
            if step >= *delay {
                let ap_output_last = unsafe {
                    estimations
                        .ap_outputs_last
                        .uget((state_index, offset_index))
                };
                let output_state = output_state.context(
                    "Output state index not initialized - algorithm parameter corruption",
                )?;
                let state_val =
                    unsafe { estimations.system_states.uget((step - delay, output_state)) };
                let ap_gain = unsafe {
                    functional_description
                        .ap_params
                        .gains
                        .uget((state_index, offset_index))
                };
                let mapped_residual = unsafe { derivatives.mapped_residuals.uget(state_index) };
                let coef_derivative = unsafe { derivatives.coefs.uget_mut(coef_index) };
                *coef_derivative += ((state_val - ap_output_last) * ap_gain * mapped_residual)
                    .mul_add(
                        mse_scaling,
                        config.difference_regularization_strength * delay_delta,
                    );
            }
        }
    }
    Ok(())
}

/// Calculates the derivatives for the allpass filter coefficients using the textbook form for the AP derivative.
///
/// # Errors
///
/// Returns an error if algorithm parameters are not properly initialized.
#[inline]
#[allow(clippy::cast_precision_loss)]
#[tracing::instrument(level = "trace")]
pub fn calculate_derivatives_coefs_textbook(
    derivatives: &mut Derivatives,
    estimations: &Estimations,
    functional_description: &FunctionalDescription,
    step: usize,
    config: &Algorithm,
) -> Result<()> {
    let mse_scaling = 1.0 / estimations.measurements.num_sensors() as f32 * config.mse_strength;

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
                let output_state = output_state.context(
                    "Output state index not initialized - algorithm parameter corruption",
                )?;
                let state_val =
                    unsafe { estimations.system_states.uget((step - delay, output_state)) };
                let derivative_fir =
                    unsafe { derivatives.coefs_fir.uget_mut((state_index, offset_index)) };
                *derivative_fir = (-*coef).mul_add(*derivative_fir, *state_val);
            }
        }
    }

    // IIR derivatives calculation
    for state_index in 0..derivatives.coefs_iir.shape()[0] {
        for offset_index in 0..derivatives.coefs_iir.shape()[1] {
            let coef_index = (state_index / 3, offset_index / 3);
            let delay = unsafe { functional_description.ap_params.delays.uget(coef_index) };
            let coef = unsafe { functional_description.ap_params.coefs.uget(coef_index) };

            if step >= *delay {
                let ap_output_last = unsafe {
                    estimations
                        .ap_outputs_last
                        .uget((state_index, offset_index))
                };
                let derivative_iir =
                    unsafe { derivatives.coefs_iir.uget_mut((state_index, offset_index)) };
                *derivative_iir = (-*coef).mul_add(*derivative_iir, *ap_output_last);
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
            let mapped_residual = unsafe { derivatives.mapped_residuals.uget(state_index) };

            let coef_derivative = unsafe { derivatives.coefs.uget_mut(coef_index) };
            *coef_derivative += ((fir - iir) * ap_gain * mapped_residual).mul_add(
                mse_scaling,
                config.difference_regularization_strength * delay_delta,
            );
        }
    }
    Ok(())
}

/// Calculates the maximum regularization for the given system states.
///
/// Iterates through the states, calculates the sum of the absolute values,
/// compares to the threshold, and assigns maximum regularization accordingly.
#[inline]
#[tracing::instrument(level = "trace", skip_all)]
pub fn calculate_maximum_regularization(
    maximum_regularization: &mut MaximumRegularization,
    maximum_regularization_sum: &mut f32,
    system_states: &SystemStatesAtStep,
    regularization_threshold: f32,
) {
    trace!("Calculating maximum regularization");
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
pub fn calculate_average_delays(
    average_delays: &mut AverageDelays,
    ap_params: &APParameters,
) -> Result<()> {
    for voxel_index in 0..average_delays.shape()[0] {
        let mut delay_sum = 0.0;
        let mut gain_sum = 0.0;

        for offset in 0..ap_params.delays.shape()[1] {
            let x_y_z_offset = delay_index_to_offset(offset)
                .context("Invalid delay offset index - algorithm parameter corruption")?;
            let x_y_z_sum: f32 = x_y_z_offset.map(i32::abs).iter().sum::<i32>() as f32;

            let delay = unsafe { *ap_params.delays.uget((voxel_index, offset)) } as f32
                + from_coef_to_samples(unsafe { *ap_params.coefs.uget((voxel_index, offset)) });

            let delay_corrected = delay / (x_y_z_sum.sqrt());

            for input_dimension in 0..3 {
                for output_dimension in 0..3 {
                    let gain = unsafe {
                        *ap_params.gains.uget((
                            voxel_index * 3 + input_dimension,
                            offset * 3 + output_dimension,
                        ))
                    };
                    delay_sum += gain.abs() * delay_corrected;
                    gain_sum += gain.abs();
                }
            }
        }

        let average_delay = unsafe { average_delays.uget_mut(voxel_index) };
        if gain_sum == 0.0 {
            *average_delay = None;
        } else {
            *average_delay = Some(delay_sum / gain_sum);
        }
    }
    Ok(())
}
