use crate::core::{
    config::algorithm::Algorithm,
    model::functional::allpass::{
        flat::APParametersFlat,
        shapes::flat::{ArrayDelaysFlat, ArrayGainsFlat},
    },
};

use super::derivation::DerivativesFlat;

impl APParametersFlat {
    /// Performs one gradient descent step on the all-pass parameters.
    ///
    /// Derivatives must be reset before the next update.
    pub fn update(
        &mut self,
        derivatives: &DerivativesFlat,
        config: &Algorithm,
        number_of_steps: usize,
    ) {
        let batch_size = match config.batch_size {
            0 => number_of_steps,
            _ => config.batch_size,
        };
        if !config.freeze_gains {
            update_gains_flat(
                &mut self.gains,
                &derivatives.gains,
                config.learning_rate,
                batch_size,
                config.gradient_clamping_threshold,
            );
        }
        if !config.freeze_delays {
            update_delays_flat(
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

/// Performs one gradient descent step on the all-pass gains.
#[allow(clippy::cast_precision_loss)]
fn update_gains_flat(
    gains: &mut ArrayGainsFlat<f32>,
    derivatives: &ArrayGainsFlat<f32>,
    learning_rate: f32,
    batch_size: usize,
    clamping_threshold: f32,
) {
    gains.values -= &(learning_rate / batch_size as f32 * &derivatives.values)
        .map(|v| v.clamp(-clamping_threshold, clamping_threshold));
}

/// Performs one gradient descent step on the all-pass coeffs.
///
/// Coefficients are kept between 0 and 1.
///
/// When a step would place a coefficient outside this range,
/// the integer delay parameter is adjusted to "roll" the
/// coefficient.
#[allow(clippy::cast_precision_loss)]
fn update_delays_flat(
    ap_coefs: &mut ArrayDelaysFlat<f32>,
    delays: &mut ArrayDelaysFlat<usize>,
    derivatives: &ArrayDelaysFlat<f32>,
    learning_rate: f32,
    batch_size: usize,
    clamping_threshold: f32,
) {
    ap_coefs.values -= &(learning_rate / batch_size as f32 * &derivatives.values)
        .map(|v| v.clamp(-clamping_threshold, clamping_threshold));
    // make sure to keep the all pass coefficients between 0 and 1 by
    // wrapping them around and adjusting the delays accordingly.
    ap_coefs
        .values
        .iter_mut()
        .zip(delays.values.iter_mut())
        .for_each(|(ap_coef, delay)| {
            if *ap_coef > 1.0 {
                if *delay > 0 {
                    *ap_coef -= 1.0;
                    *delay -= 1;
                } else {
                    *ap_coef = 1.0;
                }
            } else if *ap_coef < 0.0 {
                *ap_coef += 1.0;
                *delay += 1;
            }
        });
}

#[cfg(test)]
mod tests {
    use ndarray::Dim;

    use super::*;

    #[test]
    fn update_gains_success() {
        let number_of_states = 10;
        let mut gains = ArrayGainsFlat::empty(number_of_states);
        let mut derivatives = ArrayGainsFlat::empty(number_of_states);
        derivatives.values.fill(-0.5);
        let learning_rate = 1.0;

        update_gains_flat(&mut gains, &derivatives, learning_rate, 1, 1.0);

        assert_eq!(-derivatives.values, gains.values);
    }

    #[test]
    fn update_delays_success() {
        let number_of_states = 12;
        let mut ap_coefs = ArrayDelaysFlat::empty(number_of_states);
        let mut delays = ArrayDelaysFlat::empty(number_of_states);
        let mut derivatives = ArrayDelaysFlat::empty(number_of_states);
        derivatives.values.fill(-0.5);
        let learning_rate = 1.0;

        update_delays_flat(
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
