use anyhow::{Context, Result};
use tracing::debug;

use super::derivation::Derivatives;
use crate::core::{
    algorithm::refinement::Optimizer,
    config::algorithm::Algorithm,
    model::functional::allpass::{
        shapes::{Coefs, Gains, UnitDelays},
        APParameters,
    },
};

impl APParameters {
    /// Updates the allpass filter parameters based on the provided derivatives.
    ///
    /// This takes in the derivatives calculated during backpropagation and uses them
    /// to update the filter's gains and delays, based on the provided learning rate
    /// and batch size. Freezing gains or delays can be configured via the Algorithm
    /// config. Gradient clamping is also applied based on the config threshold.
    ///
    /// # Errors
    ///
    /// Returns an error if optimizer configuration is invalid (e.g. Adam optimizer without moment arrays).
    #[inline]
    #[tracing::instrument(level = "debug")]
    pub fn update(
        &mut self,
        derivatives: &mut Derivatives,
        config: &Algorithm,
        number_of_steps: usize,
        number_of_beats: usize,
    ) -> Result<()> {
        debug!("Updating allpass filter parameters");
        let batch_size = match config.batch_size {
            0 => number_of_steps * number_of_beats,
            _ => number_of_steps * config.batch_size,
        };

        if !config.freeze_gains {
            match config.optimizer {
                Optimizer::Sgd => {
                    update_gains_sgd(
                        &mut self.gains,
                        &derivatives.gains,
                        config.learning_rate,
                        batch_size,
                    );
                }
                Optimizer::Adam => {
                    let gains_first_moment = derivatives.gains_first_moment.as_mut()
                        .context("Adam optimizer requires first moment arrays - optimizer configuration error")?;
                    let gains_second_moment = derivatives.gains_second_moment.as_mut()
                        .context("Adam optimizer requires second moment arrays - optimizer configuration error")?;
                    update_gains_adam(
                        &mut self.gains,
                        &derivatives.gains,
                        gains_first_moment,
                        gains_second_moment,
                        derivatives.step,
                        config.learning_rate,
                        batch_size,
                    );
                }
            }
        }

        if !config.freeze_delays {
            match config.optimizer {
                Optimizer::Sgd => update_delays_sgd(
                    &mut self.coefs,
                    &derivatives.coefs,
                    config.learning_rate,
                    batch_size,
                    config.slow_down_stregth,
                ),
                Optimizer::Adam => {
                    let coefs_first_moment = derivatives.coefs_first_moment.as_mut()
                        .context("Adam optimizer requires coefficient first moment arrays - optimizer configuration error")?;
                    let coefs_second_moment = derivatives.coefs_second_moment.as_mut()
                        .context("Adam optimizer requires coefficient second moment arrays - optimizer configuration error")?;
                    update_delays_adam(
                        &mut self.coefs,
                        &derivatives.coefs,
                        coefs_first_moment,
                        coefs_second_moment,
                        derivatives.step,
                        config.learning_rate,
                        batch_size,
                    );
                },
            }
            roll_delays(&mut self.coefs, &mut self.delays);
        }
        derivatives.step += 1;
        Ok(())
    }
}

/// Updates the gains based on the provided derivatives, learning rate,
/// batch size, and gradient clamping threshold. The gains are updated
/// by subtracting the scaled and clamped derivatives.
#[allow(clippy::cast_precision_loss)]
#[inline]
#[tracing::instrument(level = "debug")]
pub fn update_gains_sgd(
    gains: &mut Gains,
    derivatives: &Gains,
    learning_rate: f32,
    batch_size: usize,
) {
    debug!("Updating gains");
    **gains -= &(learning_rate / batch_size as f32 * &**derivatives);
}

#[allow(clippy::cast_precision_loss)]
#[inline]
#[tracing::instrument(level = "debug")]
pub fn update_gains_adam(
    gains: &mut Gains,
    derivatives: &Gains,
    first_moment: &mut Gains,
    second_moment: &mut Gains,
    step: usize,
    learning_rate: f32,
    batch_size: usize,
) {
    debug!("Updating gains");
    // these need to be parameters in the config...
    let beta1 = 0.9;
    let one_minus_beta1 = 1. - beta1;
    let beta2 = 0.999;
    let one_minus_beta2 = 1. - beta2;
    let epsilon = 1e-8;

    **first_moment = &**first_moment * beta1 + (one_minus_beta1 * &**derivatives);
    **second_moment =
        &**second_moment * beta2 + (one_minus_beta2 * &**derivatives * &**derivatives);

    let first_moment_cor = &**first_moment / (1. - beta1.powf(step as f32));
    let second_moment_cor = &**second_moment / (1. - beta2.powf(step as f32));

    let factor = first_moment_cor / (second_moment_cor.mapv(f32::sqrt) + epsilon);

    **gains -= &(learning_rate / batch_size as f32 * factor);
}

/// Updates the all-pass coefficients and integer delays
/// based on the provided derivatives and specified
/// learning rate, batch size, and gradient clamping threshold.
///
/// The all-pass coefficients are updated by subtracting the
/// scaled and clamped derivatives. The coefficients are kept
/// between 0 and 1 by adjusting the integer delays accordingly.
#[allow(clippy::cast_precision_loss)]
#[inline]
#[tracing::instrument(level = "debug")]
pub fn update_delays_sgd(
    ap_coefs: &mut Coefs,
    derivatives: &Coefs,
    learning_rate: f32,
    batch_size: usize,
    slow_down_strength: f32,
) {
    debug!("Updating coefficients and delays");

    **ap_coefs -= &(learning_rate / batch_size as f32 * &**derivatives);
}

#[allow(clippy::cast_precision_loss)]
#[inline]
#[tracing::instrument(level = "debug")]
pub fn update_delays_adam(
    ap_coefs: &mut Coefs,
    derivatives: &Coefs,
    first_moment: &mut Coefs,
    second_moment: &mut Coefs,
    step: usize,
    learning_rate: f32,
    batch_size: usize,
) {
    debug!("Updating coefficients and delays");
    // these need to be parameters in the config...
    let beta1 = 0.9;
    let one_minus_beta1 = 1. - beta1;
    let beta2 = 0.999;
    let one_minus_beta2 = 1. - beta2;
    let epsilon = 1e-8;

    **first_moment = &**first_moment * beta1 + (one_minus_beta1 * &**derivatives);
    **second_moment =
        &**second_moment * beta2 + (one_minus_beta2 * &**derivatives * &**derivatives);

    let first_moment_cor = &**first_moment / (1. - beta1.powf(step as f32));
    let second_moment_cor = &**second_moment / (1. - beta2.powf(step as f32));

    let factor = first_moment_cor / (second_moment_cor.mapv(f32::sqrt) + epsilon);

    **ap_coefs -= &(learning_rate / batch_size as f32 * factor);
}

// make sure to keep the all pass coefficients between 0 and 1 by
// wrapping them around and adjusting the delays accordingly.
#[inline]
#[tracing::instrument(level = "debug")]
pub fn roll_delays(ap_coefs: &mut Coefs, delays: &mut UnitDelays) {
    let margin = 1e-4;
    ap_coefs
        .iter_mut()
        .zip(delays.iter_mut())
        .for_each(|(ap_coef, delay)| {
            if *ap_coef > 1.0 - margin {
                if *delay > 1 {
                    *ap_coef = 2.0 * margin;
                    *delay -= 1;
                } else {
                    *ap_coef = 1.0 - margin;
                }
            } else if *ap_coef < margin {
                if *delay < 1000 {
                    *ap_coef = 2.0f32.mul_add(-margin, 1.0);
                    *delay += 1;
                } else {
                    *ap_coef = margin;
                }
            }
        });
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn update_gains_success() {
        let number_of_states = 10;
        let mut gains = Gains::empty(number_of_states);
        let mut derivatives = Gains::empty(number_of_states);
        derivatives.fill(-0.5);
        let learning_rate = 1.0;

        update_gains_sgd(&mut gains, &derivatives, learning_rate, 1);

        assert_eq!(-&*derivatives, &*gains);
    }

    #[test]
    fn update_delays_success() {
        let number_of_states = 12;
        let mut ap_coefs = Coefs::empty(number_of_states);
        let mut delays = UnitDelays::empty(number_of_states);
        let mut derivatives = Coefs::empty(number_of_states);
        derivatives.fill(-0.5);
        let learning_rate = 1.0;

        update_delays_sgd(&mut ap_coefs, &derivatives, learning_rate, 1, 0.);
        roll_delays(&mut ap_coefs, &mut delays);

        assert_eq!(-&*derivatives, &*ap_coefs);
    }
}
