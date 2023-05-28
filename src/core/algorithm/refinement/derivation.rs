use ndarray::Array1;

use crate::core::{
    algorithm::estimation::{ArraySystemStates, Estimations},
    model::{
        shapes::{ArrayDelays, ArrayGains, ArrayIndicesGains},
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
    coefs_iir: ArrayGains<f32>,
    coefs_fir: ArrayGains<f32>,
    mapped_residuals: ArrayMappedResiduals,
}

impl Derivatives {
    pub fn new(number_of_states: usize) -> Derivatives {
        Derivatives {
            gains: ArrayGains::new(number_of_states),
            coefs: ArrayDelays::new(number_of_states),
            coefs_iir: ArrayGains::new(number_of_states),
            coefs_fir: ArrayGains::new(number_of_states),
            mapped_residuals: ArrayMappedResiduals::new(number_of_states),
        }
    }

    fn calculate(
        &mut self,
        functional_description: &FunctionalDescription,
        estimations: &Estimations,
        time_index: usize,
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
            &functional_description.ap_params.coefs,
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
    output_state_indices: &ArrayIndicesGains,
) {
    derivatives_gains
        .values
        .iter_mut()
        .zip(ap_outputs.values.iter())
        .zip(output_state_indices.values.iter())
        .filter(|(_, index_output_state)| index_output_state.is_some())
        .for_each(|((derivative, ap_output), index_output_state)| {
            *derivative += ap_output * mapped_residuals.values[index_output_state.unwrap()];
        });
}

fn calculate_derivatives_coefs(
    // These get updated
    derivatives_coefs: &mut ArrayDelays<f32>,
    derivatives_coefs_fir: &mut ArrayGains<f32>,
    derivatives_coefs_iir: &mut ArrayGains<f32>,
    // Based on these values
    ap_outputs: &ArrayGains<f32>,
    mapped_residuals: &ArrayMappedResiduals,
    estimated_system_states: &ArraySystemStates,
    ap_gains: &ArrayGains<f32>,
    ap_coefs: &ArrayDelays<f32>,
    // These are needed for indexing
    delays: &ArrayDelays<usize>,
    output_state_indices: &ArrayIndicesGains,
    time_index: usize,
) {
    derivatives_coefs_fir
        .values
        .indexed_iter_mut()
        .zip(output_state_indices.values.iter())
        .filter(|(_, output_state_index)| output_state_index.is_some())
        .for_each(
            |(((state_index, x_offset, y_offset, z_offset, _), derivative), output_state_index)| {
                let coef_index = (usize::from(state_index / 3), x_offset, y_offset, z_offset);
                *derivative = estimated_system_states.values[(
                    usize::from(time_index - delays.values[coef_index]),
                    output_state_index.unwrap(),
                )] + ap_coefs.values[coef_index] * *derivative;
            },
        );
    derivatives_coefs_iir
        .values
        .indexed_iter_mut()
        .zip(ap_outputs.values.iter())
        .for_each(
            |(((state_index, x_offset, y_offset, z_offset, _), derivative), ap_output)| {
                let coef_index = (usize::from(state_index / 3), x_offset, y_offset, z_offset);
                *derivative = ap_output + ap_coefs.values[coef_index] * *derivative;
            },
        );
}

#[cfg(test)]
mod tests {
    use crate::core::model::shapes::ArrayIndicesGains;

    use super::*;
    #[test]
    fn gains_success() {
        let number_of_states = 3;
        let mut derivatives_gains = ArrayGains::new(number_of_states);
        let mut ap_outputs = ArrayGains::new(number_of_states);
        ap_outputs.values.fill(1.0);
        ap_outputs.values[(0, 0, 0, 0, 0)] = 2.0;
        ap_outputs.values[(0, 1, 0, 0, 0)] = 999.0;
        let mut mapped_residuals = ArrayMappedResiduals::new(number_of_states);
        mapped_residuals.values[0] = -1.0;
        mapped_residuals.values[1] = 1.0;
        mapped_residuals.values[2] = 2.0;
        let mut output_state_indices = ArrayIndicesGains::new(number_of_states);
        output_state_indices
            .values
            .indexed_iter_mut()
            .filter(|((_, x_offset, y_offset, z_offset, _), _)| {
                *x_offset == 0 && *y_offset == 0 && *z_offset == 0
            })
            .for_each(|((_, _, _, _, output_direction), value)| {
                *value = Some(output_direction);
            });
        let mut derivatives_gains_exp: ArrayGains<f32> = ArrayGains::new(number_of_states);
        derivatives_gains_exp.values[(0, 0, 0, 0, 0)] = -2.0;
        derivatives_gains_exp.values[(0, 0, 0, 0, 1)] = 1.0;
        derivatives_gains_exp.values[(0, 0, 0, 0, 2)] = 2.0;

        derivatives_gains_exp.values[(1, 0, 0, 0, 0)] = -1.0;
        derivatives_gains_exp.values[(1, 0, 0, 0, 1)] = 1.0;
        derivatives_gains_exp.values[(1, 0, 0, 0, 2)] = 2.0;

        derivatives_gains_exp.values[(2, 0, 0, 0, 0)] = -1.0;
        derivatives_gains_exp.values[(2, 0, 0, 0, 1)] = 1.0;
        derivatives_gains_exp.values[(2, 0, 0, 0, 2)] = 2.0;

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
