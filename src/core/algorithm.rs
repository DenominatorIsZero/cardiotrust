use bevy::prelude::warn;

use super::model::{
    shapes::{ArrayDelays, ArrayGains},
    APParameters,
};

impl APParameters {
    fn update(&mut self, derivatives: &Derivatives, learning_rate: f32) {
        update_gains(&mut self.gains, &derivatives.gains, learning_rate);
        update_delays(
            &mut self.coefs,
            &mut self.delays,
            &derivatives.delays,
            learning_rate,
        );
    }
}

struct Derivatives {
    gains: ArrayGains,
    delays: ArrayDelays<f32>,
}

impl Derivatives {
    fn new(number_of_states: usize) -> Derivatives {
        Derivatives {
            gains: ArrayGains::new(number_of_states),
            delays: ArrayDelays::new(number_of_states),
        }
    }
}

fn update_gains(gains: &mut ArrayGains, derivatives: &ArrayGains, learning_rate: f32) {
    gains.values -= &(learning_rate * &derivatives.values);
}

fn update_delays(
    ap_coefs: &mut ArrayDelays<f32>,
    delays: &mut ArrayDelays<u32>,
    derivatives: &ArrayDelays<f32>,
    learning_rate: f32,
) {
    ap_coefs.values -= &(learning_rate * &derivatives.values);
    // make sure to keep the all pass coefficients between 0 and 1 by
    // wrapping them around and adjusting the delays accordingly.
    ap_coefs
        .values
        .iter_mut()
        .zip(delays.values.iter_mut())
        .for_each(|(ap_coef, delay)| {
            if *ap_coef > 1.0 {
                *ap_coef -= 1.0;
                if *delay > 0 {
                    *delay -= 1;
                } else {
                    warn!(
                        "Tried to reduce delay due to ap coef wrapping, but delay is already zero."
                    );
                }
            } else if *ap_coef < 0.0 {
                *ap_coef += 1.0;
                *delay += 1;
            }
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn update_gains_success() {
        let mut gains = ArrayGains::new(10);
        let mut derivatives = ArrayGains::new(10);
        derivatives.values.fill(-0.5);
        let learning_rate = 1.0;

        update_gains(&mut gains, &derivatives, learning_rate);

        assert_eq!(-derivatives.values, gains.values);
    }

    #[test]
    fn update_delays_success() {
        let mut ap_coefs = ArrayDelays::<f32>::new(12);
        let mut delays = ArrayDelays::<u32>::new(12);
        let mut derivatives = ArrayDelays::<f32>::new(12);
        derivatives.values.fill(-0.5);
        let learning_rate = 1.0;

        update_delays(&mut ap_coefs, &mut delays, &derivatives, learning_rate);

        assert_eq!(-derivatives.values, ap_coefs.values);
    }

    #[test]
    fn update_delays_wrap_success() {
        let mut ap_coefs = ArrayDelays::<f32>::new(3);
        ap_coefs.values.fill(0.2);
        ap_coefs.values[[0, 0, 0, 0]] = 0.8;
        let mut delays = ArrayDelays::<u32>::new(3);
        delays.values.fill(3);
        delays.values[[0, 0, 0, 1]] = 2;
        let mut derivatives = ArrayDelays::<f32>::new(3);
        derivatives.values.fill(0.5);
        derivatives.values[[0, 0, 0, 2]] = -0.9;
        let learning_rate = 1.0;

        let mut ap_coefs_exp = ArrayDelays::<f32>::new(3);
        ap_coefs_exp.values.fill(0.7);
        ap_coefs_exp.values[[0, 0, 0, 0]] = 0.3;
        ap_coefs_exp.values[[0, 0, 0, 2]] = 0.1;
        let mut delays_exp = ArrayDelays::<u32>::new(3);
        delays_exp.values.fill(4);
        delays_exp.values[[0, 0, 0, 0]] = 3;
        delays_exp.values[[0, 0, 0, 1]] = 3;
        delays_exp.values[[0, 0, 0, 2]] = 2;

        update_delays(&mut ap_coefs, &mut delays, &derivatives, learning_rate);

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
        let mut ap_parameters = APParameters::new(3);
        ap_parameters.coefs.values.fill(0.2);
        ap_parameters.coefs.values[[0, 0, 0, 0]] = 0.8;
        ap_parameters.delays.values.fill(3);
        ap_parameters.delays.values[[0, 0, 0, 1]] = 2;

        let mut derivatives = Derivatives::new(3);
        derivatives.gains.values.fill(0.5);
        derivatives.delays.values.fill(0.5);
        derivatives.delays.values[[0, 0, 0, 2]] = -0.9;

        let learning_rate = 1.0;

        let mut ap_coefs_exp = ArrayDelays::<f32>::new(3);
        ap_coefs_exp.values.fill(0.7);
        ap_coefs_exp.values[[0, 0, 0, 0]] = 0.3;
        ap_coefs_exp.values[[0, 0, 0, 2]] = 0.1;
        let mut delays_exp = ArrayDelays::<u32>::new(3);
        delays_exp.values.fill(4);
        delays_exp.values[[0, 0, 0, 0]] = 3;
        delays_exp.values[[0, 0, 0, 1]] = 3;
        delays_exp.values[[0, 0, 0, 2]] = 2;

        ap_parameters.update(&derivatives, learning_rate);
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
