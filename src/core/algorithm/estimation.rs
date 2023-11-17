pub mod shapes;

use std::ops::AddAssign;

use itertools::Itertools;
use ndarray::{s, Array2, Zip};
use ndarray_linalg::Inverse;
use serde::{Deserialize, Serialize};

use crate::core::config::algorithm::Algorithm;
use crate::core::model::functional::allpass::from_coef_to_samples;
use crate::core::model::functional::allpass::shapes::ArrayDelays;
use crate::core::model::functional::measurement::MeasurementMatrix;
use crate::core::model::{
    functional::allpass::shapes::ArrayGains, functional::FunctionalDescription,
};

use crate::core::data::shapes::{ArrayMeasurements, ArraySystemStates};
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Estimations {
    pub ap_outputs: ArrayGains<f32>,
    pub system_states: ArraySystemStates,
    pub state_covariance_pred: ArrayGains<f32>,
    pub state_covariance_est: ArrayGains<f32>,
    pub measurements: ArrayMeasurements,
    pub residuals: ArrayMeasurements,
    pub post_update_residuals: ArrayMeasurements,
    pub system_states_delta: ArraySystemStates,
    pub gains_delta: ArrayGains<f32>,
    pub delays_delta: ArrayDelays<f32>,
    pub s: Array2<f32>,
    pub s_inv: Array2<f32>,
    pub kalman_gain_converged: bool,
}

impl Estimations {
    #[must_use]
    pub fn empty(
        number_of_states: usize,
        number_of_sensors: usize,
        number_of_steps: usize,
    ) -> Self {
        Self {
            ap_outputs: ArrayGains::empty(number_of_states),
            system_states: ArraySystemStates::empty(number_of_steps, number_of_states),
            state_covariance_pred: ArrayGains::empty(number_of_states),
            state_covariance_est: ArrayGains::empty(number_of_states),
            measurements: ArrayMeasurements::empty(number_of_steps, number_of_sensors),
            residuals: ArrayMeasurements::empty(1, number_of_sensors),
            post_update_residuals: ArrayMeasurements::empty(1, number_of_sensors),
            system_states_delta: ArraySystemStates::empty(1, number_of_states),
            gains_delta: ArrayGains::empty(number_of_states),
            delays_delta: ArrayDelays::empty(number_of_states),
            s: Array2::zeros([number_of_sensors, number_of_sensors]),
            s_inv: Array2::zeros([number_of_sensors, number_of_sensors]),
            kalman_gain_converged: false,
        }
    }

    pub fn reset(&mut self) {
        self.ap_outputs.values.fill(0.0);
        self.system_states.values.fill(0.0);
        self.state_covariance_pred.values.fill(0.0);
        self.state_covariance_est.values.fill(0.0);
        self.measurements.values.fill(0.0);
        self.residuals.values.fill(0.0);
        self.post_update_residuals.values.fill(0.0);
        self.system_states_delta.values.fill(0.0);
        self.gains_delta.values.fill(0.0);
        self.delays_delta.values.fill(0.0);
        self.kalman_gain_converged = false;
    }

    pub(crate) fn save_npy(&self, path: &std::path::Path) {
        self.system_states.save_npy(path);
        self.measurements.save_npy(path);
    }
}

pub fn par_calculate_system_prediction(
    ap_outputs: &mut ArrayGains<f32>,
    system_states: &mut ArraySystemStates,
    measurements: &mut ArrayMeasurements,
    functional_description: &FunctionalDescription,
    time_index: usize,
) {
    // Calculate ap outputs and system states
    Zip::indexed(&mut ap_outputs.values)
        .and(&functional_description.ap_params.output_state_indices.values)
        .par_for_each(|gain_index, ap_output, output_state_index_option| {
            if let Some(output_state_index) = output_state_index_option {
                let coef_index = (gain_index.0 / 3, gain_index.1, gain_index.2, gain_index.3);
                let coef = functional_description.ap_params.coefs.values[coef_index];
                let delay = functional_description.ap_params.delays.values[coef_index];
                let input = if delay <= time_index {
                    system_states.values[(time_index - delay, *output_state_index)]
                } else {
                    0.0
                };
                let input_delayed = if delay < time_index {
                    system_states.values[(time_index - delay - 1, *output_state_index)]
                } else {
                    0.0
                };
                *ap_output = coef.mul_add(input - *ap_output, input_delayed);
            }
        });
    Zip::indexed(&mut system_states.values.slice_mut(s![time_index, ..])).par_for_each(
        |state_index, state| {
            for (((x, y), z), d) in (0..=2) // over neighors of input voxel
                .cartesian_product(0..=2)
                .cartesian_product(0..=2)
                .cartesian_product(0..=2)
            {
                *state += functional_description.ap_params.gains.values[[state_index, x, y, z, d]]
                    * ap_outputs.values[[state_index, x, y, z, d]];
            }
        },
    );
    // Add control function
    let control_function_value = functional_description.control_function_values.values[time_index];
    system_states
        .values
        .slice_mut(s![time_index, ..])
        .add_assign(&(&functional_description.control_matrix.values * control_function_value));
    // Prediction of measurements H * x
    measurements.values.slice_mut(s![time_index, ..]).assign(
        &functional_description
            .measurement_matrix
            .values
            .dot(&system_states.values.slice(s![time_index, ..])),
    );
}

#[inline]
pub fn calculate_system_prediction(
    ap_outputs: &mut ArrayGains<f32>,
    system_states: &mut ArraySystemStates,
    measurements: &mut ArrayMeasurements,
    functional_description: &FunctionalDescription,
    time_index: usize,
) {
    innovate_system_states(
        ap_outputs,
        functional_description,
        time_index,
        system_states,
    );
    add_control_function(functional_description, time_index, system_states);
    predict_measurements(
        measurements,
        time_index,
        &functional_description.measurement_matrix,
        system_states,
    );
}

#[inline]
pub fn innovate_system_states(
    ap_outputs: &mut ArrayGains<f32>,
    functional_description: &FunctionalDescription,
    time_index: usize,
    system_states: &mut ArraySystemStates,
) {
    // Calculate ap outputs and system states
    ap_outputs
        .values
        .indexed_iter_mut()
        .zip(
            functional_description
                .ap_params
                .output_state_indices
                .values
                .iter(),
        )
        .filter(|((gain_index, _), output_state_index)| {
            output_state_index.is_some()
                && !(gain_index.1 == 1 && gain_index.2 == 1 && gain_index.3 == 1)
        })
        .for_each(|((gain_index, ap_output), output_state_index)| {
            let coef_index = (gain_index.0 / 3, gain_index.1, gain_index.2, gain_index.3);
            let coef = functional_description.ap_params.coefs.values[coef_index];
            let delay = functional_description.ap_params.delays.values[coef_index];
            let input = if delay <= time_index {
                system_states.values[(time_index - delay, output_state_index.unwrap_or_default())]
            } else {
                0.0
            };
            let input_delayed = if delay < time_index {
                system_states.values[(
                    time_index - delay - 1,
                    output_state_index.unwrap_or_default(),
                )]
            } else {
                0.0
            };
            *ap_output = coef.mul_add(input - *ap_output, input_delayed);
            let gain = functional_description.ap_params.gains.values[gain_index];
            system_states.values[(time_index, gain_index.0)] += gain * *ap_output;
        });
}

#[inline]
pub fn add_control_function(
    functional_description: &FunctionalDescription,
    time_index: usize,
    system_states: &mut ArraySystemStates,
) {
    // Add control function
    let control_function_value = functional_description.control_function_values.values[time_index];
    system_states
        .values
        .slice_mut(s![time_index, ..])
        .iter_mut()
        .zip(functional_description.control_matrix.values.iter())
        .for_each(|(system_state, coef)| {
            *system_state += coef * control_function_value;
        });
}

#[inline]
pub fn predict_measurements(
    measurements: &mut ArrayMeasurements,
    time_index: usize,
    measurement_matrix: &MeasurementMatrix,
    system_states: &mut ArraySystemStates,
) {
    // Prediction of measurements H * x
    measurements.values.slice_mut(s![time_index, ..]).assign(
        &measurement_matrix
            .values
            .dot(&system_states.values.slice(s![time_index, ..])),
    );
}

#[inline]
pub fn calculate_residuals(
    residuals: &mut ArrayMeasurements,
    predicted_measurements: &ArrayMeasurements,
    actual_measurements: &ArrayMeasurements,
    time_index: usize,
) {
    residuals.values.slice_mut(s![0, ..]).assign(
        &(&predicted_measurements.values.slice(s![time_index, ..])
            - &actual_measurements.values.slice(s![time_index, ..])),
    );
}

#[inline]
pub fn calculate_post_update_residuals(
    post_update_residuals: &mut ArrayMeasurements,
    measurement_matrix: &MeasurementMatrix,
    estimated_system_states: &ArraySystemStates,
    actual_measurements: &ArrayMeasurements,
    time_index: usize,
) {
    post_update_residuals.values.slice_mut(s![0, ..]).assign(
        &(measurement_matrix
            .values
            .dot(&estimated_system_states.values.slice(s![time_index, ..]))
            - actual_measurements.values.slice(s![time_index, ..])),
    );
}

#[inline]
pub fn calculate_system_states_delta(
    system_states_delta: &mut ArraySystemStates,
    estimated_system_states: &ArraySystemStates,
    actual_system_states: &ArraySystemStates,
    time_index: usize,
) {
    system_states_delta.values.slice_mut(s![0, ..]).assign(
        &(&estimated_system_states.values.slice(s![time_index, ..])
            - &actual_system_states.values.slice(s![time_index, ..])),
    );
}

#[inline]
pub fn calculate_gains_delta(
    gains_delta: &mut ArrayGains<f32>,
    estimated_gains: &ArrayGains<f32>,
    actual_gains: &ArrayGains<f32>,
) {
    gains_delta
        .values
        .assign(&(&estimated_gains.values - &actual_gains.values));
}

#[inline]
pub fn calculate_delays_delta(
    delays_delta: &mut ArrayDelays<f32>,
    estimated_delays: &ArrayDelays<usize>,
    actual_delays: &ArrayDelays<usize>,
    estimated_coefs: &ArrayDelays<f32>,
    actual_coefs: &ArrayDelays<f32>,
) {
    #[allow(clippy::cast_precision_loss)]
    delays_delta
        .values
        .indexed_iter_mut()
        .for_each(|(index, delay_delta)| {
            *delay_delta = (estimated_delays.values[index] as f32
                - actual_delays.values[index] as f32)
                + (from_coef_to_samples(estimated_coefs.values[index])
                    - from_coef_to_samples(actual_coefs.values[index]));
        });
}

#[inline]
pub fn calculate_system_update(
    estimations: &mut Estimations,
    time_index: usize,
    functional_description: &mut FunctionalDescription,
    config: &Algorithm,
) {
    if config.calculate_kalman_gain && !estimations.kalman_gain_converged {
        let kalman_gain_old = functional_description.kalman_gain.values.clone();
        calculate_kalman_gain(estimations, functional_description);
        let difference = (kalman_gain_old - &functional_description.kalman_gain.values)
            .mapv(|v| v.powi(2))
            .sum();
        if difference == 0.0 {
            estimations.kalman_gain_converged = true;
        }
    }
    let mut states = estimations
        .system_states
        .values
        .slice_mut(s![time_index, ..]);
    states.assign(
        &(&states
            + functional_description
                .kalman_gain
                .values
                .dot(&estimations.residuals.values.slice(s![0, ..]))),
    );
}

#[inline]
fn calculate_kalman_gain(
    estimations: &mut Estimations,
    functional_description: &mut FunctionalDescription,
) {
    predict_state_covariance(estimations, functional_description);
    calculate_s_inv(estimations, functional_description);
    calculate_k(functional_description, estimations);
    estimate_state_covariance(estimations, functional_description);
}

#[inline]
fn estimate_state_covariance(
    estimations: &mut Estimations,
    functional_description: &mut FunctionalDescription,
) {
    estimations
        .state_covariance_est
        .values
        .indexed_iter_mut()
        .zip(
            functional_description
                .ap_params
                .output_state_indices
                .values
                .iter(),
        )
        .filter(|(_, output_state_index)| output_state_index.is_some())
        .for_each(|((index, variance), output_state_index)| {
            *variance = 0.0;
            for (((k_x, k_y), k_z), k_d) in (0..=2) // over neighors of input voxel
                .cartesian_product(0..=2)
                .cartesian_product(0..=2)
                .cartesian_product(0..=2)
            {
                let k = functional_description.ap_params.output_state_indices.values
                    [[output_state_index.unwrap(), k_x, k_y, k_z, k_d]];
                if k.is_none() {
                    continue;
                }
                let mut sum = 0.0;
                for m in 0..functional_description.measurement_matrix.values.raw_dim()[0] {
                    sum += functional_description.kalman_gain.values[[index.0, m]]
                        * functional_description.measurement_matrix.values[[m, k.unwrap()]];
                }
                let i = if index.0 == k.unwrap() { 1.0 } else { 0.0 };
                *variance += estimations.state_covariance_pred.values[[
                    k.unwrap(),
                    flip(k_x),
                    flip(k_y),
                    flip(k_z),
                    output_state_index.unwrap() % 3,
                ]] * (i - sum);
            }
        });
}

#[inline]
fn calculate_k(functional_description: &mut FunctionalDescription, estimations: &mut Estimations) {
    functional_description
        .kalman_gain
        .values
        .indexed_iter_mut()
        .for_each(|(index, value)| {
            *value = 0.0;
            for k in 0..estimations.s.raw_dim()[0] {
                let mut sum = 0.0;
                for (((m_x, m_y), m_z), m_d) in (0..=2) // over neighbors of output voxel
                    .cartesian_product(0..=2)
                    .cartesian_product(0..=2)
                    .cartesian_product(0..=2)
                {
                    let m = functional_description.ap_params.output_state_indices.values
                        [[index.0, m_x, m_y, m_z, m_d]];
                    if m.is_none() {
                        continue;
                    }
                    sum += estimations.state_covariance_pred.values[[index.0, m_x, m_y, m_z, m_d]]
                        * functional_description.measurement_matrix.values[[k, m.unwrap()]];
                }
                *value += estimations.s_inv[[k, index.1]] * sum;
            }
        });
}

#[inline]
fn calculate_s_inv(estimations: &mut Estimations, functional_description: &FunctionalDescription) {
    estimations.s.indexed_iter_mut().for_each(|(index, value)| {
        *value = functional_description.measurement_covariance.values[index];
        for k in 0..functional_description
            .measurement_covariance
            .values
            .raw_dim()[1]
        {
            let mut sum = 0.0;
            for (((m_x, m_y), m_z), m_d) in (0..=2) // over neighors of input voxel
                .cartesian_product(0..=2)
                .cartesian_product(0..=2)
                .cartesian_product(0..=2)
            {
                // check if voxel m exists.
                let m = functional_description.ap_params.output_state_indices.values
                    [[k, m_x, m_y, m_z, m_d]];
                if m.is_none() {
                    continue;
                }
                sum += functional_description.measurement_matrix.values[[index.0, m.unwrap()]]
                    * estimations.state_covariance_pred.values
                        [[m.unwrap(), flip(m_x), flip(m_y), flip(m_z), k % 3]];
            }
            *value += functional_description.measurement_matrix.values[[index.1, k]] * sum;
        }
    });
    estimations.s_inv = estimations.s.inv().unwrap();
}

#[allow(clippy::cast_sign_loss)]
#[inline]
fn predict_state_covariance(
    estimations: &mut Estimations,
    functional_description: &FunctionalDescription,
) {
    estimations
        .state_covariance_pred
        .values
        .indexed_iter_mut()
        .zip(
            functional_description
                .ap_params
                .output_state_indices
                .values
                .iter(),
        )
        .filter(|(_, output_state_index)| output_state_index.is_some())
        .for_each(|((index, variance), output_state_index)| {
            *variance = functional_description.process_covariance.values[index];
            for (((k_x, k_y), k_z), k_d) in (0..=2) // over neighbors of output voxel
                .cartesian_product(0..=2)
                .cartesian_product(0..=2)
                .cartesian_product(0..=2)
            {
                // skip if neighbor doesn't exist
                let k = functional_description.ap_params.output_state_indices.values
                    [[output_state_index.unwrap(), k_x, k_y, k_z, k_d]];

                if k.is_none() {
                    continue;
                }
                let mut sum = 0.0;
                for (((m_x, m_y), m_z), m_d) in (0..=2) // over neighors of input voxel
                    .cartesian_product(0..=2)
                    .cartesian_product(0..=2)
                    .cartesian_product(0..=2)
                {
                    // skip if neighbor doesn't exist
                    let m = functional_description.ap_params.output_state_indices.values
                        [[index.0, m_x, m_y, m_z, m_d]];

                    if m.is_none() {
                        continue;
                    }

                    // we have to check if m and k are adjacent to see if P_{m, k} exists
                    #[allow(clippy::cast_possible_wrap, clippy::cast_possible_truncation)]
                    let m_to_k_x = (m_x as i32 + k_x as i32 + index.1 as i32) - 3;
                    #[allow(clippy::cast_possible_wrap, clippy::cast_possible_truncation)]
                    let m_to_k_y = (m_y as i32 + k_y as i32 + index.2 as i32) - 3;
                    #[allow(clippy::cast_possible_wrap, clippy::cast_possible_truncation)]
                    let m_to_k_z = (m_z as i32 + k_z as i32 + index.3 as i32) - 3;

                    if !(-1..=1).contains(&m_to_k_x)
                        || !(-1..=1).contains(&m_to_k_y)
                        || !(-1..=1).contains(&m_to_k_z)
                    {
                        continue;
                    }

                    sum += functional_description.ap_params.gains.values
                        [[index.0, m_x, m_y, m_z, m_d]]
                        * estimations.state_covariance_est.values[[
                            m.unwrap(),
                            (m_to_k_x + 1) as usize,
                            (m_to_k_y + 1) as usize,
                            (m_to_k_z + 1) as usize,
                            m_d,
                        ]];
                }
                *variance += functional_description.ap_params.gains.values
                    [[output_state_index.unwrap(), k_x, k_y, k_z, k_d]]
                    * sum;
            }
        });
}

#[inline]
fn flip(x: usize) -> usize {
    // Output state indicies:
    // 0 = -1
    // 1 = 0
    // 2 = 1
    //
    // Now if we need it the other way around,
    // 0 needs to be 2
    // 2 needs to be 0
    // 1 needs to be 1
    // Finally, getting d the other way around d needs to map to index.0 / 3
    match x {
        0 => 2,
        1 => 1,
        2 => 0,
        _ => panic!("Please nothing greater than 2. Thanks."),
    }
}

#[cfg(test)]
mod tests {
    use ndarray::Dim;

    use super::*;
    #[test]
    fn prediction_no_crash() {
        let number_of_states = 3000;
        let number_of_sensors = 300;
        let number_of_steps = 2000;
        let time_index = 333;
        let voxels_in_dims = Dim([1000, 1, 1]);

        let mut ap_outputs = ArrayGains::empty(number_of_states);
        let mut system_states = ArraySystemStates::empty(number_of_steps, number_of_states);
        let mut measurements = ArrayMeasurements::empty(number_of_steps, number_of_sensors);
        let functional_description = FunctionalDescription::empty(
            number_of_states,
            number_of_sensors,
            number_of_steps,
            voxels_in_dims,
        );

        calculate_system_prediction(
            &mut ap_outputs,
            &mut system_states,
            &mut measurements,
            &functional_description,
            time_index,
        );
    }

    #[test]
    fn update_no_crash() {
        let number_of_states = 3000;
        let number_of_sensors = 300;
        let number_of_steps = 2000;
        let time_index = 333;
        let config = Algorithm::default();

        let mut estimations =
            Estimations::empty(number_of_states, number_of_sensors, number_of_steps);
        let mut functional_desrciption = FunctionalDescription::empty(
            number_of_states,
            number_of_sensors,
            number_of_steps,
            Dim([number_of_states / 3, 1, 1]),
        );

        calculate_system_update(
            &mut estimations,
            time_index,
            &mut functional_desrciption,
            &config,
        );
    }

    #[test]
    fn residuals_no_crash() {
        let number_of_sensors = 300;
        let number_of_steps = 2000;
        let time_index = 333;

        let mut residuals = ArrayMeasurements::empty(1, number_of_sensors);
        let predicted_measurements = ArrayMeasurements::empty(number_of_steps, number_of_sensors);
        let actual_measurements = ArrayMeasurements::empty(number_of_steps, number_of_sensors);

        calculate_residuals(
            &mut residuals,
            &predicted_measurements,
            &actual_measurements,
            time_index,
        );
    }
}
