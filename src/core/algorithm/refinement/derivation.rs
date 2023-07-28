use ndarray::{s, Array1};
use serde::{Deserialize, Serialize};

use crate::core::{
    algorithm::estimation::Estimations,
    model::functional::{
        allpass::{
            shapes::{ArrayDelays, ArrayGains, ArrayIndicesGains},
            APParameters,
        },
        FunctionalDescription,
    },
};

use crate::core::data::shapes::ArraySystemStates;

/// Stuct to calculate and store the derivatives
/// of the model parameters with regards to the
/// Loss function.
#[allow(clippy::unsafe_derive_deserialize)]
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Derivatives {
    /// Derivatives of the All-pass gains
    pub gains: ArrayGains<f32>,
    /// Derivatives of the All-pass coeficients
    pub coefs: ArrayDelays<f32>,
    /// IIR component of the coeficients derivatives
    /// only used for internal computation
    coefs_iir: ArrayGains<f32>,
    /// FIR component of the coeficients derivatives
    /// only used for internal computation
    coefs_fir: ArrayGains<f32>,
    /// Residuals mapped onto the system states via
    /// the measurement matrix.
    /// Stored internally to avoid redundant computation
    mapped_residuals: ArrayMappedResiduals,
}

impl Derivatives {
    #[must_use]
    pub fn new(number_of_states: usize) -> Self {
        Self {
            gains: ArrayGains::empty(number_of_states),
            coefs: ArrayDelays::empty(number_of_states),
            coefs_iir: ArrayGains::empty(number_of_states),
            coefs_fir: ArrayGains::empty(number_of_states),
            mapped_residuals: ArrayMappedResiduals::new(number_of_states),
        }
    }

    /// Sets all arrays to zero.
    ///
    /// Usually used after updating the parameters.
    pub fn reset(&mut self) {
        self.gains.values.fill(0.0);
        self.coefs.values.fill(0.0);
        self.coefs_iir.values.fill(0.0);
        self.coefs_fir.values.fill(0.0);
        self.mapped_residuals.values.fill(0.0);
    }

    /// Calculates the derivatives for the given time index.
    ///
    /// CAUTION: adds to old values. use "reset" after using the
    /// derivatives to update the parameters.
    pub fn calculate(
        &mut self,
        functional_description: &FunctionalDescription,
        estimations: &Estimations,
        time_index: usize,
    ) {
        self.mapped_residuals.values = functional_description
            .measurement_matrix
            .values
            .t()
            .dot(&estimations.residuals.values.slice(s![0, ..]));
        self.calculate_derivatives_gains(
            &estimations.ap_outputs,
            &functional_description.ap_params.output_state_indices,
        );
        self.calculate_derivatives_coefs(
            &estimations.ap_outputs,
            &estimations.system_states,
            &functional_description.ap_params,
            time_index,
        );
    }

    fn calculate_derivatives_gains(
        // This gets updated
        &mut self,
        // Based on these values
        ap_outputs: &ArrayGains<f32>,
        // This needed for indexing
        output_state_indices: &ArrayIndicesGains,
    ) {
        self.gains
            .values
            .iter_mut()
            .zip(ap_outputs.values.iter())
            .zip(output_state_indices.values.iter())
            .filter(|(_, index_output_state)| index_output_state.is_some())
            .for_each(|((derivative, ap_output), index_output_state)| {
                *derivative +=
                    ap_output * self.mapped_residuals.values[index_output_state.unwrap()];
            });
    }

    fn calculate_derivatives_coefs(
        // These get updated
        &mut self,
        // Based on these values
        ap_outputs: &ArrayGains<f32>,
        estimated_system_states: &ArraySystemStates,
        ap_params: &APParameters,
        time_index: usize,
    ) {
        self.coefs_fir
            .values
            .indexed_iter_mut()
            .zip(ap_params.output_state_indices.values.iter())
            .filter(|(_, output_state_index)| output_state_index.is_some())
            .for_each(
                |(
                    ((state_index, x_offset, y_offset, z_offset, _), derivative),
                    output_state_index,
                )| {
                    let coef_index = (state_index / 3, x_offset, y_offset, z_offset);
                    if time_index >= ap_params.delays.values[coef_index] {
                        *derivative = ap_params.coefs.values[coef_index].mul_add(
                            *derivative,
                            estimated_system_states.values[(
                                time_index - ap_params.delays.values[coef_index],
                                output_state_index.unwrap(),
                            )],
                        );
                    }
                },
            );
        self.coefs_iir
            .values
            .indexed_iter_mut()
            .zip(ap_outputs.values.iter())
            .for_each(
                |(((state_index, x_offset, y_offset, z_offset, _), derivative), ap_output)| {
                    let coef_index = (state_index / 3, x_offset, y_offset, z_offset);
                    *derivative =
                        ap_params.coefs.values[coef_index].mul_add(*derivative, *ap_output);
                },
            );
        self.coefs_iir
            .values
            .indexed_iter()
            .zip(self.coefs_fir.values.iter())
            .zip(ap_params.gains.values.iter())
            .zip(ap_params.output_state_indices.values.iter())
            .filter(|(_, output_state_index)| output_state_index.is_some())
            .for_each(
                |(
                    ((((state_index, x_offset, y_offset, z_offset, _), iir), fir), ap_gain),
                    output_state_index,
                )| {
                    let coef_index = (state_index / 3, x_offset, y_offset, z_offset);
                    self.coefs.values[coef_index] += (fir + iir)
                        * ap_gain
                        * self.mapped_residuals.values[output_state_index.unwrap()];
                },
            );
    }
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
struct ArrayMappedResiduals {
    pub values: Array1<f32>,
}

impl ArrayMappedResiduals {
    #[must_use]
    pub fn new(number_of_states: usize) -> Self {
        Self {
            values: Array1::zeros(number_of_states),
        }
    }
}

#[cfg(test)]
mod tests {
    use ndarray::Dim;

    use crate::core::model::functional::allpass::shapes::ArrayIndicesGains;

    use super::*;
    #[test]
    fn gains_success() {
        let number_of_states = 3;
        let mut ap_outputs = ArrayGains::empty(number_of_states);
        ap_outputs.values.fill(1.0);
        ap_outputs.values[(0, 0, 0, 0, 0)] = 2.0;
        ap_outputs.values[(0, 1, 0, 0, 0)] = 999.0;
        let mut mapped_residuals = ArrayMappedResiduals::new(number_of_states);
        mapped_residuals.values[0] = -1.0;
        mapped_residuals.values[1] = 1.0;
        mapped_residuals.values[2] = 2.0;
        let mut output_state_indices = ArrayIndicesGains::empty(number_of_states);
        output_state_indices
            .values
            .indexed_iter_mut()
            .filter(|((_, x_offset, y_offset, z_offset, _), _)| {
                *x_offset == 0 && *y_offset == 0 && *z_offset == 0
            })
            .for_each(|((_, _, _, _, output_direction), value)| {
                *value = Some(output_direction);
            });
        let mut derivatives_gains_exp: ArrayGains<f32> = ArrayGains::empty(number_of_states);
        derivatives_gains_exp.values[(0, 0, 0, 0, 0)] = -2.0;
        derivatives_gains_exp.values[(0, 0, 0, 0, 1)] = 1.0;
        derivatives_gains_exp.values[(0, 0, 0, 0, 2)] = 2.0;

        derivatives_gains_exp.values[(1, 0, 0, 0, 0)] = -1.0;
        derivatives_gains_exp.values[(1, 0, 0, 0, 1)] = 1.0;
        derivatives_gains_exp.values[(1, 0, 0, 0, 2)] = 2.0;

        derivatives_gains_exp.values[(2, 0, 0, 0, 0)] = -1.0;
        derivatives_gains_exp.values[(2, 0, 0, 0, 1)] = 1.0;
        derivatives_gains_exp.values[(2, 0, 0, 0, 2)] = 2.0;

        let mut derivatives = Derivatives::new(number_of_states);

        derivatives.mapped_residuals = mapped_residuals;
        derivatives.calculate_derivatives_gains(&ap_outputs, &output_state_indices);

        assert!(
            derivatives_gains_exp
                .values
                .relative_eq(&derivatives.gains.values, 1e-5, 0.001),
            "expected:\n{}\nactual:\n{}",
            derivatives_gains_exp.values,
            derivatives.gains.values
        );
    }

    #[allow(clippy::similar_names)]
    #[test]
    fn coef_no_crash() {
        let number_of_steps = 2000;
        let number_of_states = 3000;
        let ap_outputs = ArrayGains::empty(number_of_states);
        let estimated_system_states = ArraySystemStates::empty(number_of_steps, number_of_states);
        let ap_params = APParameters::empty(number_of_states, Dim([1000, 1, 1]));
        let mut delays = ArrayDelays::empty(number_of_states);
        delays.values.fill(30);
        let mut output_state_indices = ArrayIndicesGains::empty(number_of_states);
        output_state_indices.values.fill(Some(3));
        let time_index = 10;

        let mut derivatives = Derivatives::new(number_of_states);

        derivatives.calculate_derivatives_coefs(
            &ap_outputs,
            &estimated_system_states,
            &ap_params,
            time_index,
        );
    }

    #[test]
    fn calculate_no_crash() {
        let number_of_states = 1500;
        let number_of_sensors = 300;
        let number_of_steps = 2000;
        let time_index = 333;
        let voxels_in_dims = Dim([1000, 1, 1]);

        let mut derivates = Derivatives::new(number_of_states);
        let functional_description = FunctionalDescription::empty(
            number_of_states,
            number_of_sensors,
            number_of_steps,
            voxels_in_dims,
        );
        let estimations = Estimations::new(number_of_states, number_of_sensors, number_of_steps);

        derivates.calculate(&functional_description, &estimations, time_index);
    }
}
