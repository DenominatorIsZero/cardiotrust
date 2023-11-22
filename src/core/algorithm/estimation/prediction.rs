use ndarray::s;

use crate::core::model::functional::measurement::MeasurementMatrix;
use crate::core::model::{
    functional::allpass::shapes::normal::ArrayGains, functional::FunctionalDescription,
};

use crate::core::data::shapes::{ArrayMeasurements, ArraySystemStates};

#[allow(clippy::module_name_repetitions)]
#[inline]
pub fn calculate_system_prediction(
    ap_outputs: &mut ArrayGains<f32>,
    system_states: &mut ArraySystemStates,
    measurements: &mut ArrayMeasurements,
    functional_description: &FunctionalDescription,
    time_index: usize,
) {
    innovate_system_states_v3(
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

/// Naive version of state innovation. uses indexed iter.
///
#[inline]
pub fn innovate_system_states_v1(
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

/// Uses manual loops. Faster than v1.
///
/// # Panics
///
/// Panics if output state indices are not initialized corrrectly.
#[inline]
pub fn innovate_system_states_v2(
    ap_outputs: &mut ArrayGains<f32>,
    functional_description: &FunctionalDescription,
    time_index: usize,
    system_states: &mut ArraySystemStates,
) {
    // Calculate ap outputs and system states
    let output_state_indices = &functional_description.ap_params.output_state_indices.values;
    for index_state in 0..ap_outputs.values.shape()[0] {
        for x in 0..3 {
            for y in 0..3 {
                for z in 0..3 {
                    if (x == 1 && y == 1 && z == 1)
                        || output_state_indices[(index_state, x, y, z, 0)].is_none()
                    {
                        continue;
                    }
                    let coef_index = (index_state / 3, x, y, z);
                    let coef = functional_description.ap_params.coefs.values[coef_index];
                    let delay = functional_description.ap_params.delays.values[coef_index];
                    for dim in 0..3 {
                        let output_state_index =
                            output_state_indices[(index_state, x, y, z, dim)].unwrap();
                        let input = if delay <= time_index {
                            system_states.values[(time_index - delay, output_state_index)]
                        } else {
                            0.0
                        };
                        let input_delayed = if delay < time_index {
                            system_states.values[(time_index - delay - 1, output_state_index)]
                        } else {
                            0.0
                        };
                        ap_outputs.values[(index_state, x, y, z, dim)] = coef.mul_add(
                            input - ap_outputs.values[(index_state, x, y, z, dim)],
                            input_delayed,
                        );
                        let gain = functional_description.ap_params.gains.values
                            [(index_state, x, y, z, dim)];
                        system_states.values[(time_index, index_state)] +=
                            gain * ap_outputs.values[(index_state, x, y, z, dim)];
                    }
                }
            }
        }
    }
}

/// Uses unsafe get operations.
///
/// # Panics
///
/// Panics if output state indices are not initialized corrrectly.
#[inline]
pub fn innovate_system_states_v3(
    ap_outputs: &mut ArrayGains<f32>,
    functional_description: &FunctionalDescription,
    time_index: usize,
    system_states: &mut ArraySystemStates,
) {
    // Calculate ap outputs and system states
    let output_state_indices = &functional_description.ap_params.output_state_indices.values;
    for index_state in 0..ap_outputs.values.shape()[0] {
        for x in 0..3 {
            for y in 0..3 {
                for z in 0..3 {
                    if (x == 1 && y == 1 && z == 1)
                        || unsafe {
                            output_state_indices
                                .uget((index_state, x, y, z, 0))
                                .is_none()
                        }
                    {
                        continue;
                    }
                    let coef_index = (index_state / 3, x, y, z);
                    let coef = unsafe {
                        *functional_description
                            .ap_params
                            .coefs
                            .values
                            .uget(coef_index)
                    };
                    let delay = unsafe {
                        *functional_description
                            .ap_params
                            .delays
                            .values
                            .uget(coef_index)
                    };
                    for dim in 0..3 {
                        let output_state_index = unsafe {
                            output_state_indices
                                .uget((index_state, x, y, z, dim))
                                .unwrap()
                        };
                        let input = if delay <= time_index {
                            unsafe {
                                *system_states
                                    .values
                                    .uget((time_index - delay, output_state_index))
                            }
                        } else {
                            0.0
                        };
                        let input_delayed = if delay < time_index {
                            *unsafe {
                                system_states
                                    .values
                                    .uget((time_index - delay - 1, output_state_index))
                            }
                        } else {
                            0.0
                        };
                        let ap_output =
                            unsafe { ap_outputs.values.uget_mut((index_state, x, y, z, dim)) };
                        *ap_output = coef.mul_add(input - *ap_output, input_delayed);
                        let gain = unsafe {
                            *functional_description.ap_params.gains.values.uget((
                                index_state,
                                x,
                                y,
                                z,
                                dim,
                            ))
                        };
                        unsafe {
                            *system_states.values.uget_mut((time_index, index_state)) +=
                                gain * ap_outputs.values.uget((index_state, x, y, z, dim));
                        };
                    }
                }
            }
        }
    }
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

#[cfg(test)]
mod tests {
    use crate::core::{
        algorithm::estimation::{prediction::innovate_system_states_v3, Estimations},
        config::Config,
        data::Data,
        model::Model,
    };

    use super::{innovate_system_states_v1, innovate_system_states_v2};

    #[test]
    fn innovate_system_states_v2_equality() {
        let config = Config::default();
        let simulation_config = config.simulation.as_ref().unwrap();
        let data = Data::from_simulation_config(simulation_config);
        let model = Model::from_model_config(
            &config.algorithm.model,
            simulation_config.sample_rate_hz,
            simulation_config.duration_s,
        )
        .unwrap();
        let mut estimations_v1 = Estimations::empty(
            model.spatial_description.voxels.count_states(),
            model.spatial_description.sensors.count(),
            data.get_measurements().values.shape()[0],
        );
        let mut estimations_v2 = Estimations::empty(
            model.spatial_description.voxels.count_states(),
            model.spatial_description.sensors.count(),
            data.get_measurements().values.shape()[0],
        );
        for time_index in 0..estimations_v2.measurements.values.shape()[0] {
            innovate_system_states_v2(
                &mut estimations_v2.ap_outputs,
                &model.functional_description,
                time_index,
                &mut estimations_v2.system_states,
            );
            innovate_system_states_v1(
                &mut estimations_v1.ap_outputs,
                &model.functional_description,
                time_index,
                &mut estimations_v1.system_states,
            );
        }
        assert_eq!(
            estimations_v1.system_states.values,
            estimations_v2.system_states.values
        );
    }
    #[test]
    fn innovate_system_states_v3_equality() {
        let config = Config::default();
        let simulation_config = config.simulation.as_ref().unwrap();
        let data = Data::from_simulation_config(simulation_config);
        let model = Model::from_model_config(
            &config.algorithm.model,
            simulation_config.sample_rate_hz,
            simulation_config.duration_s,
        )
        .unwrap();
        let mut estimations_v1 = Estimations::empty(
            model.spatial_description.voxels.count_states(),
            model.spatial_description.sensors.count(),
            data.get_measurements().values.shape()[0],
        );
        let mut estimations_v3 = Estimations::empty(
            model.spatial_description.voxels.count_states(),
            model.spatial_description.sensors.count(),
            data.get_measurements().values.shape()[0],
        );
        for time_index in 0..estimations_v3.measurements.values.shape()[0] {
            innovate_system_states_v3(
                &mut estimations_v3.ap_outputs,
                &model.functional_description,
                time_index,
                &mut estimations_v3.system_states,
            );
            innovate_system_states_v1(
                &mut estimations_v1.ap_outputs,
                &model.functional_description,
                time_index,
                &mut estimations_v1.system_states,
            );
        }
        assert_eq!(
            estimations_v1.system_states.values,
            estimations_v3.system_states.values
        );
    }
}
