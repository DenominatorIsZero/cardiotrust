use ndarray::Array1;

use crate::core::{
    algorithm::estimation::{ArraySystemStates, Estimations},
    model::{
        shapes::{ArrayDelays, ArrayGains},
        FunctionalDescription,
    },
};
/// Shape for the mapped residuals.
///
/// Has dimensions (number_of_states)
///
/// The residuals (measurements) of the state estimation
/// get mapped onto the system states.
/// These values are then used for the calcualtion of the derivatives
///
/// The mapped residuals are calculated as
/// H_T * y
struct ArrayMappedResiduals {
    pub values: Array1<f32>,
}

impl ArrayMappedResiduals {
    pub fn new(number_of_states: usize) -> ArrayMappedResiduals {
        ArrayMappedResiduals {
            values: Array1::zeros(number_of_states),
        }
    }
}

pub struct Derivatives {
    pub gains: ArrayGains<f32>,
    pub coefs: ArrayDelays<f32>,
    coefs_iir: ArrayDelays<f32>,
    coefs_fir: ArrayDelays<f32>,
    mapped_residuals: ArrayMappedResiduals,
}

impl Derivatives {
    pub fn new(number_of_states: usize) -> Derivatives {
        Derivatives {
            gains: ArrayGains::new(number_of_states),
            coefs: ArrayDelays::new(number_of_states),
            coefs_iir: ArrayDelays::new(number_of_states),
            coefs_fir: ArrayDelays::new(number_of_states),
            mapped_residuals: ArrayMappedResiduals::new(number_of_states),
        }
    }

    fn calculate(
        &mut self,
        functional_description: &FunctionalDescription,
        estimations: &Estimations,
        time_index: u32,
    ) {
        // residuals = ...
        // mapped residuals = ...
        calculate_derivatives_gains(
            &mut self.gains,
            &estimations.ap_outputs,
            &self.mapped_residuals,
            &functional_description.ap_params.output_state_indices,
        );
        calculate_derivatives_coefs(
            &mut self.coefs,
            &mut self.coefs_fir,
            &mut self.coefs_iir,
            &estimations.ap_outputs,
            &self.mapped_residuals,
            &estimations.system_states,
            &functional_description.ap_params.gains,
            &functional_description.ap_params.delays,
            &functional_description.ap_params.output_state_indices,
            time_index,
        );
    }
}

fn calculate_derivatives_gains(
    // This gets updated
    derivatives_gains: &mut ArrayGains<f32>,
    // Based on these values
    ap_outputs: &ArrayGains<f32>,
    mapped_residuals: &ArrayMappedResiduals,
    // This needed for indexing
    output_state_indices: &ArrayGains<u32>,
) {
    todo!();
}

fn calculate_derivatives_coefs(
    // These get updated
    derivatives_coefs: &mut ArrayDelays<f32>,
    derivatives_coefs_fir: &mut ArrayDelays<f32>,
    derivatives_coefs_iir: &mut ArrayDelays<f32>,
    // Based on these values
    ap_outputs: &ArrayGains<f32>,
    mapped_residuals: &ArrayMappedResiduals,
    estimated_system_states: &ArraySystemStates,
    ap_gains: &ArrayGains<f32>,
    // These are needed for indexing
    delays: &ArrayDelays<u32>,
    output_state_indices: &ArrayGains<u32>,
    time_index: u32,
) {
    todo!();
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn gains_success() {
        let number_of_states = 12;
        let mut derivatives_gains = ArrayGains::new(number_of_states);
        let ap_outputs = ArrayGains::new(number_of_states);
        let mapped_residuals = ArrayMappedResiduals::new(number_of_states);
        let output_state_indices = ArrayGains::new(number_of_states);

        let derivatives_gains_exp: ArrayGains<f32> = ArrayGains::new(number_of_states);

        calculate_derivatives_gains(
            &mut derivatives_gains,
            &ap_outputs,
            &mapped_residuals,
            &output_state_indices,
        );

        assert!(
            derivatives_gains_exp
                .values
                .relative_eq(&derivatives_gains.values, 1e-5, 0.001),
            "expected:\n{}\nactual:\n{}",
            derivatives_gains_exp.values,
            derivatives_gains.values
        )
    }
}
