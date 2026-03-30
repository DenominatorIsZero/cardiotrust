use anyhow::{Context, Result};
use approx::assert_relative_eq;
use ndarray::{Array2, Dim};

use super::*;
use crate::core::{
    algorithm::estimation::Estimations,
    model::functional::{allpass::from_samples_to_coef, FunctionalDescription},
};

#[test]
fn coef_no_crash() -> Result<()> {
    let number_of_steps = 2000;
    let number_of_states = 3000;
    let number_of_sensors = 10;
    let number_of_beats = 1;
    let step = 10;
    let mut derivatives = Derivatives::new(number_of_states, Optimizer::Sgd);
    let estimations = Estimations::empty(
        number_of_states,
        number_of_sensors,
        number_of_steps,
        number_of_beats,
    );
    let functional_description = FunctionalDescription::empty(
        number_of_states,
        number_of_sensors,
        number_of_steps,
        number_of_beats,
        Dim([1000, 1, 1]),
    );
    let config = Algorithm {
        maximum_regularization_strength: 0.0,
        smoothness_regularization_strength: 0.0,
        ..Default::default()
    };

    calculate_derivatives_coefs_simple(
        &mut derivatives,
        &estimations,
        &functional_description,
        step,
        &config,
    )?;
    Ok(())
}

#[test]
fn calculate_no_crash() -> anyhow::Result<()> {
    let number_of_states = 1500;
    let number_of_sensors = 300;
    let number_of_steps = 2000;
    let number_of_beats = 10;
    let step = 333;
    let voxels_in_dims = Dim([1000, 1, 1]);
    let config = Algorithm {
        maximum_regularization_strength: 0.0,
        smoothness_regularization_strength: 0.0,
        ..Default::default()
    };

    let mut derivates = Derivatives::new(number_of_states, config.optimizer);
    let functional_description = FunctionalDescription::empty(
        number_of_states,
        number_of_sensors,
        number_of_steps,
        number_of_beats,
        voxels_in_dims,
    );
    let estimations = Estimations::empty(
        number_of_states,
        number_of_sensors,
        number_of_steps,
        number_of_beats,
    );

    calculate_step_derivatives(
        &mut derivates,
        &estimations,
        &functional_description,
        &config,
        step,
        0,
        estimations.measurements.num_sensors(),
    )?;
    Ok(())
}

#[test]
fn calculate_average_delays_single_voxel() -> Result<()> {
    let mut ap_params = APParameters::empty(3, Dim([1, 1, 1]));

    let mut average_delays = AverageDelays::empty(3);
    let delays = Array2::from_elem((1, 26), 2);
    let coefs = Array2::from_elem((1, 26), from_samples_to_coef(0.5));
    let gains = Array2::from_elem((3, 78), 1.0);

    ap_params.delays.assign(&delays);
    ap_params.coefs.assign(&coefs);
    ap_params.gains.assign(&gains);

    calculate_average_delays(&mut average_delays, &ap_params)?;
    assert_relative_eq!(
        average_delays[0].context("Expected average delay at index 0")?,
        1.836_931_7,
        epsilon = 1e-6
    );
    Ok(())
}

#[test]
fn test_calculate_average_delays_multiple_voxels() -> Result<()> {
    let mut ap_params = APParameters::empty(6, Dim([2, 1, 1]));

    let mut average_delays = AverageDelays::empty(6);
    let delays = Array2::from_elem((2, 26), 2);
    let coefs = Array2::from_elem((2, 26), from_samples_to_coef(0.4));
    let gains = Array2::from_elem((6, 78), 1.0);

    ap_params.delays.assign(&delays);
    ap_params.coefs.assign(&coefs);
    ap_params.gains.assign(&gains);

    calculate_average_delays(&mut average_delays, &ap_params)?;
    assert_relative_eq!(
        average_delays[0].context("Expected average delay at index 0")?,
        1.763_453_2,
        epsilon = 1e-4
    );
    assert_relative_eq!(
        average_delays[1].context("Expected average delay at index 1")?,
        1.763_453,
        epsilon = 1e-4
    );
    Ok(())
}

#[test]
fn test_calculate_average_delays_zero_gains() -> Result<()> {
    let mut ap_params = APParameters::empty(3, Dim([1, 1, 1]));

    let mut average_delays = AverageDelays::empty(3);
    let delays = Array2::from_elem((1, 26), 2);
    let coefs = Array2::from_elem((1, 26), from_samples_to_coef(0.5));
    let gains = Array2::from_elem((3, 78), 0.0);

    ap_params.delays.assign(&delays);
    ap_params.coefs.assign(&coefs);
    ap_params.gains.assign(&gains);

    calculate_average_delays(&mut average_delays, &ap_params)?;
    assert!(average_delays[0].is_none());
    Ok(())
}

#[test]
fn test_calculate_average_delays_mixed_gains() -> Result<()> {
    let mut ap_params = APParameters::empty(3, Dim([1, 1, 1]));

    let mut average_delays = AverageDelays::empty(3);
    let delays = Array2::from_elem((1, 26), 2);
    let coefs = Array2::from_elem((1, 26), from_samples_to_coef(0.1));
    let mut gains = Array2::from_elem((3, 78), 0.0);
    gains[[0, 10]] = 1.0;
    gains[[1, 20]] = 4.0;
    gains[[2, 30]] = 2.0;

    ap_params.delays.assign(&delays);
    ap_params.coefs.assign(&coefs);
    ap_params.gains.assign(&gains);

    calculate_average_delays(&mut average_delays, &ap_params)?;
    assert_relative_eq!(
        average_delays[0].context("Expected average delay at index 0")?,
        1.504_952_5,
        epsilon = 1e-6
    );
    Ok(())
}
