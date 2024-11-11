pub mod prediction;

use itertools::Itertools;
use nalgebra::DMatrix;
use ndarray::s;
use serde::{Deserialize, Serialize};
use tracing::{debug, trace};

use crate::core::{
    config::algorithm::Algorithm,
    data::shapes::{
        ActivationTimePerStateMs, Measurements, MeasurementsAtStep, MeasurementsAtStepMut,
        Residuals, SystemStates, SystemStatesAtStep, SystemStatesAtStepMut, SystemStatesSpherical,
        SystemStatesSphericalMax,
    },
    model::functional::{
        allpass::{
            from_coef_to_samples, gain_index_to_offset, offset_to_gain_index,
            shapes::{Coefs, Gains, UnitDelays},
            APParameters,
        },
        kalman::KalmanGain,
        measurement::{MeasurementCovariance, MeasurementMatrixAtBeat},
    },
};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Estimations {
    pub ap_outputs: Gains,
    pub system_states: SystemStates,
    pub system_states_spherical: SystemStatesSpherical,
    pub system_states_spherical_max: SystemStatesSphericalMax,
    pub activation_times: ActivationTimePerStateMs,
    pub state_covariance_pred: Gains,
    pub state_covariance_est: Gains,
    pub measurements: Measurements,
    pub residuals: Residuals,
    pub post_update_residuals: Residuals,
    pub system_states_delta: SystemStates,
    pub system_states_spherical_max_delta: SystemStatesSphericalMax,
    pub activation_times_delta: ActivationTimePerStateMs,
    pub gains_delta: Gains,
    pub delays_delta: Coefs,
    pub innovation_covariance: DMatrix<f32>,
    pub kalman_gain_converged: bool,
}

impl Estimations {
    /// Creates a new empty Estimations struct with the given dimensions.
    #[must_use]
    #[tracing::instrument(level = "debug")]
    pub fn empty(
        number_of_states: usize,
        number_of_sensors: usize,
        number_of_steps: usize,
        number_of_beats: usize,
    ) -> Self {
        debug!("Creating empty estimations");
        Self {
            ap_outputs: Gains::empty(number_of_states),
            system_states: SystemStates::empty(number_of_steps, number_of_states),
            system_states_spherical: SystemStatesSpherical::empty(
                number_of_steps,
                number_of_states,
            ),
            system_states_spherical_max: SystemStatesSphericalMax::empty(number_of_states),
            activation_times: ActivationTimePerStateMs::empty(number_of_states),
            state_covariance_pred: Gains::empty(number_of_states),
            state_covariance_est: Gains::empty(number_of_states),
            measurements: Measurements::empty(number_of_beats, number_of_steps, number_of_sensors),
            residuals: Residuals::empty(number_of_sensors),
            post_update_residuals: Residuals::empty(number_of_sensors),
            system_states_delta: SystemStates::empty(number_of_steps, number_of_states),
            system_states_spherical_max_delta: SystemStatesSphericalMax::empty(number_of_states),
            activation_times_delta: ActivationTimePerStateMs::empty(number_of_states),
            gains_delta: Gains::empty(number_of_states),
            delays_delta: Coefs::empty(number_of_states),
            innovation_covariance: DMatrix::zeros(number_of_sensors, number_of_sensors),
            kalman_gain_converged: false,
        }
    }

    /// Resets all the internal state of the Estimations struct by filling the
    /// underlying data structures with 0.0. This is done to prepare for a new
    /// epoch.
    #[tracing::instrument(level = "debug")]
    pub fn reset(&mut self) {
        debug!("Resetting estimations");
        self.system_states.fill(0.0);
    }

    /// Saves the system states and measurements to .npy files at the given path.
    /// The filenames will be automatically generated based on the struct field names.
    #[tracing::instrument(level = "trace")]
    pub(crate) fn save_npy(&self, path: &std::path::Path) {
        trace!("Saving estimations to npy files");
        self.system_states.save_npy(path);
        self.measurements.save_npy(path);
    }
}

/// Calculates the residuals between the predicted and actual measurements for the given time index.
/// The residuals are stored in the provided `residuals` array.
#[inline]
#[tracing::instrument(level = "trace", skip_all)]
pub fn calculate_residuals(
    residuals: &mut Residuals,
    predicted_measurements: &MeasurementsAtStepMut,
    actual_measurements: &MeasurementsAtStep,
) {
    trace!("Calculating residuals");
    residuals.assign(&(&**predicted_measurements - &**actual_measurements));
}

/// Calculates the residuals between the estimated measurements from the
/// estimated system states and the actual measurements. The residuals are
/// stored in the provided `post_update_residuals` array.
#[inline]
#[tracing::instrument(level = "trace", skip_all)]
pub fn calculate_post_update_residuals(
    post_update_residuals: &mut Residuals,
    measurement_matrix: &MeasurementMatrixAtBeat,
    estimated_system_states: &SystemStatesAtStepMut,
    actual_measurements: &MeasurementsAtStep,
) {
    trace!("Calculating post update residuals");
    post_update_residuals
        .assign(&(measurement_matrix.dot(&**estimated_system_states) - **actual_measurements));
}

/// Calculates the delta between the estimated system states and the actual system states for the given time index.
/// The delta is stored in the provided `system_states_delta` array.
#[inline]
#[tracing::instrument(level = "trace", skip_all)]
pub fn calculate_system_states_delta(
    system_states_delta: &mut SystemStatesAtStepMut,
    estimated_system_states: &SystemStatesAtStepMut,
    actual_system_states: &SystemStatesAtStep,
) {
    trace!("Calculating system states delta");
    system_states_delta.assign(&(&**estimated_system_states - &**actual_system_states));
}

/// Calculates the delta between the estimated gains and the actual gains.  
/// The delta is stored in the provided `gains_delta` array.
#[inline]
#[tracing::instrument(level = "trace")]
pub fn calculate_gains_delta(
    gains_delta: &mut Gains,
    estimated_gains: &Gains,
    actual_gains: &Gains,
) {
    trace!("Calculating gains delta");
    gains_delta.assign(&(&**estimated_gains - &**actual_gains));
}

/// Calculates the delta between the estimated delays and actual delays.
/// The delta is stored in the provided `delays_delta` array.
#[inline]
#[tracing::instrument(level = "trace")]
pub fn calculate_delays_delta(
    delays_delta: &mut Coefs,
    estimated_delays: &UnitDelays,
    actual_delays: &UnitDelays,
    estimated_coefs: &Coefs,
    actual_coefs: &Coefs,
) {
    trace!("Calculating delays delta");
    #[allow(clippy::cast_precision_loss)]
    delays_delta
        .indexed_iter_mut()
        .for_each(|(index, delay_delta)| {
            *delay_delta = (estimated_delays[index] as f32 - actual_delays[index] as f32)
                + (from_coef_to_samples(estimated_coefs[index])
                    - from_coef_to_samples(actual_coefs[index]));
        });
}

/// Updates the system state estimations based on the Kalman gain and residuals.
/// If configured, calculates the Kalman gain. Checks for Kalman gain convergence.
#[inline]
#[tracing::instrument(level = "trace")]
pub fn calculate_system_update(
    system_states: &mut SystemStates,
    kalman_gain: &KalmanGain,
    residuals: &Residuals,
    time_index: usize,
    config: &Algorithm,
) {
    trace!("Calculating system update");
    let mut states: ndarray::prelude::ArrayBase<
        ndarray::ViewRepr<&mut f32>,
        ndarray::prelude::Dim<[usize; 1]>,
    > = system_states.slice_mut(s![time_index, ..]);
    states.assign(&(&states + kalman_gain.dot(&**residuals)));
}

/// Updates the Kalman gain matrix if not already converged
/// and then checks if it has converged.
/// The Kalman gain is updated by calculating the new value and comparing
/// it to the previous value. Convergence is detected when the difference
/// between the new and old Kalman gain drops below a threshold. The
/// convergence status is tracked in the estimations struct.
#[inline]
#[tracing::instrument(level = "trace", skip_all)]
pub fn update_kalman_gain_and_check_convergence(
    kalman_gain: &mut KalmanGain,
    kalman_gain_converged: &mut bool,
    state_covariance_est: &mut Gains,
    state_covariance_pred: &mut Gains,
    innovation_covariance: &mut DMatrix<f32>,
    ap_params: &APParameters,
    process_covariance: &Gains,
    measurement_covariance: &MeasurementCovariance,
    measurement_matrix: &MeasurementMatrixAtBeat,
) {
    trace!("Updating Kalman gain and checking convergence");
    if !*kalman_gain_converged {
        let kalman_gain_old = kalman_gain.clone();
        calculate_kalman_gain(
            kalman_gain,
            state_covariance_est,
            state_covariance_pred,
            innovation_covariance,
            ap_params,
            process_covariance,
            measurement_covariance,
            measurement_matrix,
        );
        let difference = (&*kalman_gain_old - &**kalman_gain)
            .mapv(|v| v.powi(2))
            .sum();
        if difference < 1e-6 {
            *kalman_gain_converged = true;
        }
    }
}

/// Calculates the Kalman gain matrix based on the current state covariance
/// and measurement covariance.
#[inline]
#[tracing::instrument(level = "trace", skip_all)]
pub fn calculate_kalman_gain(
    kalman_gain: &mut KalmanGain,
    state_covariance_est: &mut Gains,
    state_covariance_pred: &mut Gains,
    innovation_covariance: &mut DMatrix<f32>,
    ap_params: &APParameters,
    process_covariance: &Gains,
    measurement_covariance: &MeasurementCovariance,
    measurement_matrix: &MeasurementMatrixAtBeat,
) {
    trace!("Calculating Kalman gain");
    predict_state_covariance(
        state_covariance_pred,
        state_covariance_est,
        ap_params,
        process_covariance,
    );
    calculate_s_inv(
        innovation_covariance,
        state_covariance_pred,
        measurement_covariance,
        ap_params,
        measurement_matrix,
    );
    calculate_k(
        kalman_gain,
        state_covariance_pred,
        innovation_covariance,
        ap_params,
        measurement_matrix,
    );
    estimate_state_covariance(
        state_covariance_est,
        state_covariance_pred,
        ap_params,
        measurement_matrix,
        kalman_gain,
    );
}

/// Estimates the state covariance matrix based on the Kalman gain and
/// predicted state covariance.
#[inline]
#[tracing::instrument(level = "trace", skip_all)]
pub fn estimate_state_covariance(
    state_covariance_est: &mut Gains,
    state_covariance_pred: &Gains,
    ap_params: &APParameters,
    measurement_matrix: &MeasurementMatrixAtBeat,
    kalman_gain: &KalmanGain,
) {
    trace!("Estimating state covariance");
    state_covariance_est
        .indexed_iter_mut()
        .zip(ap_params.output_state_indices.iter())
        .filter(|(_, output_state_index)| output_state_index.is_some())
        .for_each(|((index, variance), output_state_index)| {
            *variance = 0.0;
            for (((k_x, k_y), k_z), k_d) in (-1..=1) // over neighors of input voxel
                .cartesian_product(-1..=1)
                .cartesian_product(-1..=1)
                .cartesian_product(0..=2)
            {
                if k_x == 0 && k_y == 0 && k_z == 0 {
                    continue;
                }
                let k = ap_params.output_state_indices[[
                    output_state_index.unwrap(),
                    offset_to_gain_index(k_x, k_y, k_z, k_d).expect("Offsets to be valid."),
                ]];
                if k.is_none() {
                    continue;
                }
                let mut sum = 0.0;
                for m in 0..measurement_matrix.raw_dim()[0] {
                    sum += kalman_gain[[index.0, m]] * measurement_matrix[[m, k.unwrap()]];
                }
                let i = if index.0 == k.unwrap() { 1.0 } else { 0.0 };
                *variance += state_covariance_pred[[
                    k.unwrap(),
                    offset_to_gain_index(-k_x, -k_y, -k_z, output_state_index.unwrap() % 3)
                        .expect("Offsets to be valid."),
                ]] * (i - sum);
            }
        });
}

/// Calculates the Kalman gain matrix based on the current state covariance
/// prediction and measurement matrix. Iterates through each element of the
/// Kalman gain matrix and computes it based on the weighted sum of relevant
/// elements from the state covariance and measurement matrices.
#[inline]
#[tracing::instrument(level = "trace", skip_all)]
pub fn calculate_k(
    kalman_gain: &mut KalmanGain,
    state_covariance_pred: &Gains,
    innovation_covariance: &DMatrix<f32>,
    ap_params: &APParameters,
    measurement_matrix: &MeasurementMatrixAtBeat,
) {
    trace!("Calculating Kalman gain");
    kalman_gain.indexed_iter_mut().for_each(|(index, value)| {
        *value = 0.0;
        for k in 0..innovation_covariance.shape().0 {
            let mut sum = 0.0;
            for (((m_x, m_y), m_z), m_d) in (-1..=1) // over neighbors of output voxel
                .cartesian_product(-1..=1)
                .cartesian_product(-1..=1)
                .cartesian_product(0..=2)
            {
                if m_x == 0 && m_y == 0 && m_z == 0 {
                    continue;
                }
                let m = ap_params.output_state_indices[[
                    index.0,
                    offset_to_gain_index(m_x, m_y, m_z, m_d).expect("Offsets to be valid."),
                ]];
                if m.is_none() {
                    continue;
                }
                sum += state_covariance_pred[[
                    index.0,
                    offset_to_gain_index(m_x, m_y, m_z, m_d).expect("Offset to be valid."),
                ]] * measurement_matrix[[k, m.unwrap()]];
            }
            *value += unsafe { innovation_covariance.get_unchecked((k, index.1)) } * sum;
        }
    });
}

/// Calculates the inverse of the innovation covariance matrix S.
/// Iterates through each element of S and initializes it to the
/// corresponding element of the measurement covariance matrix.
/// Then iterates through each column of S and updates it by computing
/// the weighted sum of relevant elements from the state covariance
/// prediction matrix and the measurement matrix. Finally inverts S
/// in place.
#[inline]
#[tracing::instrument(level = "trace", skip_all)]
pub fn calculate_s_inv(
    innovation_covariance: &mut DMatrix<f32>,
    state_covariance_pred: &mut Gains,
    measurement_covariance: &MeasurementCovariance,
    ap_params: &APParameters,
    measurement_matrix: &MeasurementMatrixAtBeat,
) {
    trace!("Calculating S^-1");
    for i in 0..innovation_covariance.shape().0 {
        for j in 0..innovation_covariance.shape().1 {
            unsafe {
                *innovation_covariance.get_unchecked_mut((i, j)) = measurement_covariance[(i, j)];
            };
            for k in 0..measurement_covariance.raw_dim()[1] {
                let mut sum = 0.0;
                for (((m_x, m_y), m_z), m_d) in (-1..=1) // over neighors of input voxel
                    .cartesian_product(-1..=1)
                    .cartesian_product(-1..=1)
                    .cartesian_product(0..=2)
                {
                    if m_x == 0 && m_y == 0 && m_z == 0 {
                        continue;
                    }
                    // check if voxel m exists.
                    let m = ap_params.output_state_indices[[
                        k,
                        offset_to_gain_index(m_x, m_y, m_z, m_d).expect("Offset to be valid."),
                    ]];
                    if m.is_none() {
                        continue;
                    }
                    sum += measurement_matrix[[i, m.unwrap()]]
                        * state_covariance_pred[[
                            m.unwrap(),
                            offset_to_gain_index(-m_x, -m_y, -m_z, k % 3)
                                .expect("Offset to be valid"),
                        ]];
                }
                unsafe {
                    *innovation_covariance.get_unchecked_mut((i, j)) +=
                        measurement_matrix[[j, k]] * sum;
                };
            }
        }
    }
    innovation_covariance.try_inverse_mut();
}

/// Predicts the state covariance for the next time step using the
/// autoregressive process model. Iterates over the output state indices,
/// updating each variance using the process covariance and gains between
/// connected voxels.
#[allow(clippy::cast_sign_loss)]
#[inline]
#[tracing::instrument(level = "trace")]
pub fn predict_state_covariance(
    state_covariance_pred: &mut Gains,
    state_covariance_est: &Gains,
    ap_params: &APParameters,
    process_covariance: &Gains,
) {
    trace!("Predicting state covariance");
    state_covariance_pred
        .indexed_iter_mut()
        .zip(ap_params.output_state_indices.iter())
        .filter(|(_, output_state_index)| output_state_index.is_some())
        .for_each(|((index, variance), output_state_index)| {
            *variance = process_covariance[index];
            for (((k_x, k_y), k_z), k_d) in (-1..=1) // over neighbors of output voxel
                .cartesian_product(-1..=1)
                .cartesian_product(-1..=1)
                .cartesian_product(0..=2)
            {
                if k_x == 0 && k_y == 0 && k_z == 0 {
                    continue;
                }

                // skip if neighbor doesn't exist
                let k = ap_params.output_state_indices[[
                    output_state_index.unwrap(),
                    offset_to_gain_index(k_x, k_y, k_z, k_d).expect("Offset to be valid."),
                ]];

                if k.is_none() {
                    continue;
                }
                let mut sum = 0.0;
                for (((m_x, m_y), m_z), m_d) in (-1..=1) // over neighors of input voxel
                    .cartesian_product(-1..=1)
                    .cartesian_product(-1..=1)
                    .cartesian_product(0..=2)
                {
                    if m_x == 0 && m_y == 0 && m_z == 0 {
                        continue;
                    }
                    // skip if neighbor doesn't exist
                    let m = ap_params.output_state_indices[[
                        index.0,
                        offset_to_gain_index(m_x, m_y, m_z, m_d).expect("Offset to be valid"),
                    ]];

                    if m.is_none() {
                        continue;
                    }

                    // we have to check if m and k are adjacent to see if P_{m, k} exists
                    let offset =
                        gain_index_to_offset(index.1).expect("Gain index to be less than 78");
                    #[allow(clippy::cast_possible_wrap, clippy::cast_possible_truncation)]
                    let m_to_k_x = m_x + k_x + offset[0];
                    #[allow(clippy::cast_possible_wrap, clippy::cast_possible_truncation)]
                    let m_to_k_y = m_y + k_y + offset[1];
                    #[allow(clippy::cast_possible_wrap, clippy::cast_possible_truncation)]
                    let m_to_k_z = m_z + k_z + offset[2];

                    if !(-1..=1).contains(&m_to_k_x)
                        || !(-1..=1).contains(&m_to_k_y)
                        || !(-1..=1).contains(&m_to_k_z)
                        || (m_to_k_x == 0 && m_to_k_y == 0 && m_to_k_z == 0)
                    {
                        continue;
                    }

                    sum += ap_params.gains[[
                        index.0,
                        offset_to_gain_index(m_x, m_y, m_z, m_d).expect("Offset to be valid"),
                    ]] * state_covariance_est[[
                        m.unwrap(),
                        offset_to_gain_index(m_to_k_x, m_to_k_y, m_to_k_z, m_d)
                            .expect("Offset to be valid"),
                    ]];
                }
                *variance += ap_params.gains[[
                    output_state_index.unwrap(),
                    offset_to_gain_index(k_x, k_y, k_z, k_d).expect("Offset to be valid"),
                ]] * sum;
            }
        });
}

#[cfg(test)]
mod tests {
    use ndarray::Dim;

    use crate::core::{
        config::algorithm::Algorithm,
        data::shapes::{Measurements, Residuals, SystemStates},
        model::functional::{allpass::shapes::Gains, FunctionalDescription},
    };

    use super::{
        calculate_residuals, calculate_system_update, prediction::calculate_system_prediction,
        Estimations,
    };

    #[test]
    fn prediction_no_crash() {
        let number_of_states = 3000;
        let number_of_sensors = 300;
        let number_of_steps = 2000;
        let number_of_beats = 10;
        let step = 333;
        let beat = 4;
        let voxels_in_dims = Dim([1000, 1, 1]);

        let mut ap_outputs = Gains::empty(number_of_states);
        let mut system_states = SystemStates::empty(number_of_steps, number_of_states);
        let mut measurements =
            Measurements::empty(number_of_beats, number_of_steps, number_of_sensors);
        let functional_description = FunctionalDescription::empty(
            number_of_states,
            number_of_sensors,
            number_of_steps,
            number_of_beats,
            voxels_in_dims,
        );

        let measurement_matrix = functional_description.measurement_matrix.at_beat(beat);

        calculate_system_prediction(
            &mut ap_outputs,
            &mut system_states,
            &mut measurements.at_beat_mut(beat).at_step_mut(step),
            &functional_description.ap_params,
            &measurement_matrix,
            functional_description.control_function_values[step],
            &functional_description.control_matrix,
            step,
        );
    }

    #[test]
    fn update_no_crash() {
        let number_of_states = 3000;
        let number_of_sensors = 300;
        let number_of_steps = 2000;
        let number_of_beats = 10;
        let time_index = 333;
        let config = Algorithm::default();

        let mut estimations = Estimations::empty(
            number_of_states,
            number_of_sensors,
            number_of_steps,
            number_of_beats,
        );
        let functional_desrciption = FunctionalDescription::empty(
            number_of_states,
            number_of_sensors,
            number_of_steps,
            number_of_beats,
            Dim([number_of_states / 3, 1, 1]),
        );

        calculate_system_update(
            &mut estimations.system_states,
            &functional_desrciption.kalman_gain,
            &estimations.residuals,
            time_index,
            &config,
        );
    }

    #[test]
    fn residuals_no_crash() {
        let number_of_sensors = 300;
        let number_of_steps = 2000;
        let number_of_beats = 10;
        let step = 333;
        let beat = 2;

        let mut residuals = Residuals::empty(number_of_sensors);
        let mut predicted_measurements =
            Measurements::empty(number_of_beats, number_of_steps, number_of_sensors);
        let actual_measurements =
            Measurements::empty(number_of_beats, number_of_steps, number_of_sensors);

        calculate_residuals(
            &mut residuals,
            &predicted_measurements.at_beat_mut(beat).at_step_mut(step),
            &actual_measurements.at_beat(beat).at_step(step),
        );
    }
}
