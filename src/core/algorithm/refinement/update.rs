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
    #[inline]
    #[tracing::instrument(level = "debug")]
    pub fn update(
        &mut self,
        derivatives: &mut Derivatives,
        config: &Algorithm,
        number_of_steps: usize,
        number_of_beats: usize,
    ) {
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
                    update_gains_adam(
                        &mut self.gains,
                        &derivatives.gains,
                        derivatives.gains_first_moment.as_mut().unwrap(),
                        derivatives.gains_second_moment.as_mut().unwrap(),
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
                ),
                Optimizer::Adam => update_delays_adam(
                    &mut self.coefs,
                    &derivatives.coefs,
                    derivatives.coefs_first_moment.as_mut().unwrap(),
                    derivatives.coefs_second_moment.as_mut().unwrap(),
                    derivatives.step,
                    config.learning_rate,
                    batch_size,
                ),
            };
            roll_delays(&mut self.coefs, &mut self.delays);
        }
        derivatives.step += 1;
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
    **first_moment = &**first_moment * 0.9 + (0.1 * &**derivatives);
    **second_moment = &**second_moment * 0.999 + (0.001 * &**derivatives * &**derivatives);

    let first_moment_cor = &**first_moment / (1. - 0.9_f32.powf(step as f32));
    let second_moment_cor = &**second_moment / (1. - 0.999_f32.powf(step as f32));

    let factor = first_moment_cor / (second_moment_cor.mapv(f32::sqrt) + 1e-8);

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
    **first_moment = &**first_moment * 0.9 + (0.1 * &**derivatives);
    **second_moment = &**second_moment * 0.999 + (0.001 * &**derivatives * &**derivatives);

    let first_moment_cor = &**first_moment / (1. - 0.9_f32.powf(step as f32));
    let second_moment_cor = &**second_moment / (1. - 0.999_f32.powf(step as f32));

    let factor = first_moment_cor / (second_moment_cor.mapv(f32::sqrt) + 1e-8);

    **ap_coefs -= &(learning_rate / batch_size as f32 * factor);
}

// make sure to keep the all pass coefficients between 0 and 1 by
// wrapping them around and adjusting the delays accordingly.
#[inline]
#[tracing::instrument(level = "debug")]
pub fn roll_delays(ap_coefs: &mut Coefs, delays: &mut UnitDelays) {
    ap_coefs
        .iter_mut()
        .zip(delays.iter_mut())
        .for_each(|(ap_coef, delay)| {
            if *ap_coef > 0.99 {
                if *delay > 0 {
                    *ap_coef = 0.01;
                    *delay -= 1;
                } else {
                    *ap_coef = 0.99;
                }
            } else if *ap_coef < 0.01 {
                if *delay < 1000 {
                    *ap_coef = 0.99;
                    *delay += 1;
                } else {
                    *ap_coef = 0.01;
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

        update_delays_sgd(&mut ap_coefs, &derivatives, learning_rate, 1);
        roll_delays(&mut ap_coefs, &mut delays);

        assert_eq!(-&*derivatives, &*ap_coefs);
    }
}
