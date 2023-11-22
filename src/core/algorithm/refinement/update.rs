use crate::core::{
    config::algorithm::Algorithm,
    model::{
        functional::allpass::shapes::normal::{ArrayDelays, ArrayGains},
        functional::allpass::APParameters,
    },
};

use super::derivation::Derivatives;

impl APParameters {
    /// Performs one gradient descent step on the all-pass parameters.
    ///
    /// Derivatives must be reset before the next update.
    pub fn update(
        &mut self,
        derivatives: &Derivatives,
        config: &Algorithm,
        number_of_steps: usize,
    ) {
        let batch_size = match config.batch_size {
            0 => number_of_steps,
            _ => config.batch_size,
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

/// Performs one gradient descent step on the all-pass gains.
#[allow(clippy::cast_precision_loss)]
fn update_gains(
    gains: &mut ArrayGains<f32>,
    derivatives: &ArrayGains<f32>,
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
fn update_delays(
    ap_coefs: &mut ArrayDelays<f32>,
    delays: &mut ArrayDelays<usize>,
    derivatives: &ArrayDelays<f32>,
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

    #[test]
    fn update_delays_wrap_success() {
        let mut ap_coefs = ArrayDelays::empty(3);
        ap_coefs.values.fill(0.2);
        ap_coefs.values[[0, 0, 0, 0]] = 0.8;
        let mut delays = ArrayDelays::empty(3);
        delays.values.fill(3);
        delays.values[[0, 0, 0, 1]] = 2;
        let mut derivatives = ArrayDelays::empty(3);
        derivatives.values.fill(0.5);
        derivatives.values[[0, 0, 0, 2]] = -0.9;
        let learning_rate = 1.0;

        let mut ap_coefs_exp = ArrayDelays::empty(3);
        ap_coefs_exp.values.fill(0.7);
        ap_coefs_exp.values[[0, 0, 0, 0]] = 0.3;
        ap_coefs_exp.values[[0, 0, 0, 2]] = 0.1;
        let mut delays_exp = ArrayDelays::empty(3);
        delays_exp.values.fill(4);
        delays_exp.values[[0, 0, 0, 0]] = 3;
        delays_exp.values[[0, 0, 0, 1]] = 3;
        delays_exp.values[[0, 0, 0, 2]] = 2;

        update_delays(
            &mut ap_coefs,
            &mut delays,
            &derivatives,
            learning_rate,
            1,
            1.0,
        );

        assert!(
            ap_coefs_exp
                .values
                .relative_eq(&ap_coefs.values, 1e-5, 0.001),
            "expected:\n{}\nactual:\n{}",
            ap_coefs_exp.values,
            ap_coefs.values
        );
        assert_eq!(delays_exp.values, delays.values);
    }

    #[test]
    fn update_ap_params_success() {
        let number_of_states = 3;
        let voxels_in_dims = Dim([1, 1, 1]);
        let mut ap_parameters = APParameters::empty(number_of_states, voxels_in_dims);
        ap_parameters.coefs.values.fill(0.2);
        ap_parameters.coefs.values[[0, 0, 0, 0]] = 0.8;
        ap_parameters.delays.values.fill(3);
        ap_parameters.delays.values[[0, 0, 0, 1]] = 2;

        let mut derivatives = Derivatives::new(number_of_states);
        derivatives.gains.values.fill(0.5);
        derivatives.coefs.values.fill(0.5);
        derivatives.coefs.values[[0, 0, 0, 2]] = -0.9;

        let config = Algorithm {
            learning_rate: 1.0,
            gradient_clamping_threshold: 1.0,
            ..Default::default()
        };

        let mut ap_coefs_exp = ArrayDelays::empty(number_of_states);
        ap_coefs_exp.values.fill(0.7);
        ap_coefs_exp.values[[0, 0, 0, 0]] = 0.3;
        ap_coefs_exp.values[[0, 0, 0, 2]] = 0.1;
        let mut delays_exp = ArrayDelays::empty(number_of_states);
        delays_exp.values.fill(4);
        delays_exp.values[[0, 0, 0, 0]] = 3;
        delays_exp.values[[0, 0, 0, 1]] = 3;
        delays_exp.values[[0, 0, 0, 2]] = 2;

        ap_parameters.update(&derivatives, &config, 1);
        assert!(
            ap_coefs_exp
                .values
                .relative_eq(&ap_parameters.coefs.values, 1e-5, 0.001),
            "expected:\n{}\nactual:\n{}",
            ap_coefs_exp.values,
            ap_parameters.coefs.values
        );
        assert_eq!(delays_exp.values, ap_parameters.delays.values);
        assert_eq!(-derivatives.gains.values, ap_parameters.gains.values);
    }
}
