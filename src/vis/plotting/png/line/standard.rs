use std::{io, path::Path};

use anyhow::Result;
use ndarray::{Array1, ArrayBase, Data, Ix1};
use tracing::trace;

use super::{super::PngBundle, line_plot, log_y_plot};
use crate::core::data::shapes::SystemStates;

/// Generates a standard y plot from the provided y values.
///
/// Plots the y values against their index. Saves the plot to the provided path
/// as a PNG image. Applies the provided title, axis labels, etc.
///
/// Returns the plot data as a `Vec<u8>`, or an error if the plot could not be
/// generated.
#[tracing::instrument(level = "trace")]
pub fn standard_y_plot<A>(
    y: &ArrayBase<A, Ix1>,
    path: Option<&Path>,
    title: &str,
    y_label: &str,
    x_label: &str,
) -> Result<PngBundle>
where
    A: Data<Elem = f32>,
{
    trace!("Generating y plot.");
    line_plot(
        None,
        vec![y],
        path,
        Some(title),
        Some(y_label),
        Some(x_label),
        None,
        None,
    )
}

#[tracing::instrument(level = "trace")]
pub fn standard_log_y_plot<A>(
    y: &ArrayBase<A, Ix1>,
    path: Option<&Path>,
    title: &str,
    y_label: &str,
    x_label: &str,
) -> Result<PngBundle>
where
    A: Data<Elem = f32>,
{
    trace!("Generating y plot.");
    log_y_plot(
        None,
        vec![y],
        path,
        Some(title),
        Some(y_label),
        Some(x_label),
        None,
        None,
    )
}

/// Generates a standard time plot from the provided y values and sample rate.
///
/// Plots the y values against time in seconds based on the provided sample rate.
/// Saves the plot to the provided path as a PNG image. Applies the provided
/// title and axis labels.
///
/// Returns the plot data as a `Vec<u8>`, or an error if the plot could not be
/// generated.
#[allow(clippy::cast_precision_loss)]
#[tracing::instrument(level = "trace")]
pub fn standard_time_plot<A>(
    y: &ArrayBase<A, Ix1>,
    sample_rate_hz: f32,
    path: Option<&Path>,
    title: &str,
    y_label: &str,
) -> Result<PngBundle>
where
    A: Data<Elem = f32>,
{
    trace!("Generating time plot.");
    if sample_rate_hz <= 0.0 {
        return Err(std::io::Error::new(
            io::ErrorKind::InvalidInput,
            "sample_rate_hz must be greater than zero",
        )
        .into());
    }
    let x = Array1::linspace(0.0, y.len() as f32 / sample_rate_hz, y.len());
    line_plot(
        Some(&x),
        vec![y],
        path,
        Some(title),
        Some(y_label),
        Some("t [s]"),
        None,
        None,
    )
}

/// Generates a plot of the x, y, and z values for a specific state index from
/// the provided system state data.
///
/// Plots the x, y, and z values for the state at the given index against time
/// in seconds based on the provided sample rate. Saves the plot to the provided
/// path as a PNG image. Applies the provided title and axis labels.
///
/// `system_states` - The system state data to extract values from.
/// `state_index` - The index of the state to plot.
/// `sample_rate_hz` - The sample rate of the data in Hz.  
/// `path` - The path to save the generated plot to.
/// `title` - The title for the plot.
///
/// Returns the plot data as a `Vec<u8>`, or an error if the plot could not be
/// generated.
#[allow(clippy::cast_precision_loss)]
#[tracing::instrument(level = "trace")]
pub fn plot_state_xyz(
    system_states: &SystemStates,
    state_index: usize,
    sample_rate_hz: f32,
    path: Option<&Path>,
    title: &str,
) -> Result<PngBundle> {
    use ndarray::s;
    trace!("Generating state xyz plot.");

    if state_index >= (system_states.num_states() - 2) {
        return Err(
            std::io::Error::new(io::ErrorKind::InvalidInput, "state_index out of bounds").into(),
        );
    }

    let state_x = system_states.slice(s![.., state_index]);
    let state_y = system_states.slice(s![.., state_index + 1]);
    let state_z = system_states.slice(s![.., state_index + 2]);
    let x = Array1::linspace(0.0, state_x.len() as f32 / sample_rate_hz, state_x.len());
    let y = vec![&state_x, &state_y, &state_z];
    let labels: Vec<&str> = vec!["x", "y", "z"];
    let title = format!("{title} - State Index: {state_index}");
    line_plot(
        Some(&x),
        y,
        path,
        Some(title.as_str()),
        Some("j [A/mm^2]"),
        Some("t [s]"),
        Some(&labels),
        None,
    )
}
