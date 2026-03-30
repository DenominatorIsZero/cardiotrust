use anyhow::{Context, Result};

use super::results::Results;
use crate::core::data::Data;

#[tracing::instrument(level = "trace", skip_all)]
pub fn calculate_plotting_arrays(results: &mut Results, data: &Data) -> Result<()> {
    results
        .estimations
        .system_states_spherical
        .calculate(&results.estimations.system_states);
    results
        .estimations
        .system_states_spherical_max
        .calculate(&results.estimations.system_states_spherical)?;

    results
        .estimations
        .system_states_spherical_max_delta
        .theta
        .assign(
            &(&data.simulation.system_states_spherical_max.theta
                - &results.estimations.system_states_spherical_max.theta),
        );

    results
        .estimations
        .system_states_spherical_max_delta
        .phi
        .assign(
            &(&data.simulation.system_states_spherical_max.phi
                - &results.estimations.system_states_spherical_max.phi),
        );

    results
        .estimations
        .system_states_spherical_max_delta
        .magnitude
        .assign(
            &(&data.simulation.system_states_spherical_max.magnitude
                - &results.estimations.system_states_spherical_max.magnitude),
        );

    results.estimations.activation_times.calculate(
        &results.estimations.system_states_spherical,
        data.simulation.sample_rate_hz,
    )?;

    results
        .estimations
        .activation_times_delta
        .assign(&(&*data.simulation.activation_times - &*results.estimations.activation_times));

    results
        .model
        .as_mut()
        .context("Model should be set after algorithm execution")?
        .update_activation_time(&results.estimations.activation_times);
    Ok(())
}
