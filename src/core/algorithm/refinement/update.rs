use tracing::debug;

use super::derivation::Derivatives;
use crate::core::{
    config::algorithm::Algorithm,
    model::functional::allpass::{
        shapes::{ArrayDelays, ArrayGains},
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
        derivatives: &Derivatives,
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
            update_gains(
                &mut self.gains,
                &derivatives.gains,
                config.learning_rate,
                batch_size,
                config.gradient_clamping_threshold,
            );
        }
        if !config.freeze_delays {
            update_delays(
                &mut self.coefs,
                &mut self.delays,
                &derivatives.coefs,
                config.learning_rate,
                batch_size,
                config.gradient_clamping_threshold,
            );
        }
    }
}

/// Updates the gains based on the provided derivatives, learning rate,
/// batch size, and gradient clamping threshold. The gains are updated
/// by subtracting the scaled and clamped derivatives.
#[allow(clippy::cast_precision_loss)]
#[inline]
#[tracing::instrument(level = "debug")]
pub fn update_gains(
    gains: &mut ArrayGains<f32>,
    derivatives: &ArrayGains<f32>,
    learning_rate: f32,
    batch_size: usize,
    clamping_threshold: f32,
) {
    debug!("Updating gains");
    gains.values -= &(learning_rate / batch_size as f32 * &derivatives.values)
        .map(|v| v.clamp(-clamping_threshold, clamping_threshold));
}

/// Updates the all-pass coefficients and integer delays
/// based on the provided derivatives and specified
/// learning rate, batch size, and gradient clamping threshold.
///
/// The all-pass coefficients are updated by subtracting the
/// scaled and clamped derivatives. The coefficients are kept
/// between 0 and 1 by adjusting the integer delays accordingly.
///
/// When a step would place a coefficient outside this range,
/// the integer delay is adjusted to "roll" the coefficient.
#[allow(clippy::cast_precision_loss)]
#[inline]
#[tracing::instrument(level = "debug")]
pub fn update_delays(
    ap_coefs: &mut ArrayDelays<f32>,
    delays: &mut ArrayDelays<usize>,
    derivatives: &ArrayDelays<f32>,
    learning_rate: f32,
    batch_size: usize,
    clamping_threshold: f32,
) {
    debug!("Updating coefficients and delays");
    ap_coefs.values -= &(learning_rate / batch_size as f32 * &derivatives.values)
        .map(|v| v.clamp(-clamping_threshold, clamping_threshold));
    // make sure to keep the all pass coefficients between 0 and 1 by
    // wrapping them around and adjusting the delays accordingly.
    ap_coefs
        .values
        .iter_mut()
        .zip(delays.values.iter_mut())
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
        let mut gains = ArrayGains::empty(number_of_states);
        let mut derivatives = ArrayGains::empty(number_of_states);
        derivatives.values.fill(-0.5);
        let learning_rate = 1.0;

        update_gains(&mut gains, &derivatives, learning_rate, 1, 1.0);

        assert_eq!(-derivatives.values, gains.values);
    }

    #[test]
    fn update_delays_success() {
        let number_of_states = 12;
        let mut ap_coefs = ArrayDelays::empty(number_of_states);
        let mut delays = ArrayDelays::empty(number_of_states);
        let mut derivatives = ArrayDelays::empty(number_of_states);
        derivatives.values.fill(-0.5);
        let learning_rate = 1.0;

        update_delays(
            &mut ap_coefs,
            &mut delays,
            &derivatives,
            learning_rate,
            1,
            1.0,
        );

        assert_eq!(-derivatives.values, ap_coefs.values);
    }
}
