pub mod standard;
#[cfg(test)]
mod tests;

use std::{io, path::Path};

use anyhow::Result;
use ndarray::{Array1, ArrayBase, Data, Ix1};
use ndarray_stats::QuantileExt;
use plotters::prelude::*;
pub use standard::{plot_state_xyz, standard_log_y_plot, standard_time_plot, standard_y_plot};
use tracing::trace;

use super::PngBundle;
use crate::vis::plotting::{
    allocate_buffer, AXIS_LABEL_AREA, AXIS_STYLE, CAPTION_STYLE, CHART_MARGIN, COLORS,
    LEGEND_OPACITY, LEGEND_PATH_LENGTH, STANDARD_RESOLUTION, X_MARGIN, Y_MARGIN,
};

/// Generates an XY plot from the provided x and y data.
///
/// Saves the plot to the optionally provided path as a PNG,
/// returns the raw pixel buffer.
#[allow(clippy::cast_precision_loss, clippy::too_many_arguments)]
#[tracing::instrument(level = "trace")]
pub fn line_plot<A>(
    x: Option<&Array1<f32>>,
    ys: Vec<&ArrayBase<A, Ix1>>,
    path: Option<&Path>,
    title: Option<&str>,
    y_label: Option<&str>,
    x_label: Option<&str>,
    item_labels: Option<&Vec<&str>>,
    resolution: Option<(u32, u32)>,
) -> Result<PngBundle>
where
    A: Data<Elem = f32>,
{
    trace!("Generating xy plot.");

    let (width, height) = resolution.unwrap_or(STANDARD_RESOLUTION);

    let mut buffer = allocate_buffer(width, height);

    let y_len = ys[0].len();

    for y in &ys {
        if y.len() != y_len {
            return Err(std::io::Error::new(
                io::ErrorKind::InvalidInput,
                "y data must have same length",
            )
            .into());
        }
    }

    if let Some(item_labels) = item_labels {
        if item_labels.len() != ys.len() {
            return Err(std::io::Error::new(
                io::ErrorKind::InvalidInput,
                "if not None, item_labels must be same length as ys",
            )
            .into());
        }
    }

    let default_x = Array1::linspace(0.0, y_len as f32, y_len);
    let x = x.unwrap_or(&default_x);

    if x.len() != y_len {
        return Err(std::io::Error::new(
            io::ErrorKind::InvalidInput,
            "x and y must have same length",
        )
        .into());
    }

    let title = title.unwrap_or("Plot");
    let y_label = y_label.unwrap_or("y");
    let x_label = x_label.unwrap_or("x");

    let x_min = x.min()?;
    let x_max = x.max()?;
    let mut y_min = f32::INFINITY;
    let mut y_max = -f32::INFINITY;

    for y in &ys {
        let min = y.min()?;
        let max = y.max()?;
        y_min = y_min.min(*min);
        y_max = y_max.max(*max);
    }

    let x_range = x_max - x_min;
    let y_range = y_max - y_min;

    let x_min = x_min - x_range * X_MARGIN;
    let x_max = x_max + x_range * X_MARGIN;
    let y_min = y_range.mul_add(-Y_MARGIN, y_min);
    let y_max = y_range.mul_add(Y_MARGIN, y_max);

    {
        let root = BitMapBackend::with_buffer(&mut buffer[..], (width, height)).into_drawing_area();
        root.fill(&WHITE)?;

        let mut chart = ChartBuilder::on(&root)
            .caption(title, CAPTION_STYLE.into_font())
            .margin(CHART_MARGIN)
            .x_label_area_size(AXIS_LABEL_AREA)
            .y_label_area_size(AXIS_LABEL_AREA)
            .build_cartesian_2d(x_min..x_max, y_min..y_max)?;

        chart
            .configure_mesh()
            .x_desc(x_label)
            .x_label_style(AXIS_STYLE.into_font())
            .y_desc(y_label)
            .y_label_style(AXIS_STYLE.into_font())
            .draw()?;

        for (i, y) in ys.iter().enumerate() {
            let color = &COLORS[i % COLORS.len()];
            if let Some(item_labels) = item_labels {
                chart
                    .draw_series(LineSeries::new(
                        x.iter().zip(y.iter()).map(|(x, y)| (*x, *y)),
                        color,
                    ))?
                    .label(item_labels[i])
                    .legend(move |(x, y)| {
                        PathElement::new(vec![(x, y), (x + LEGEND_PATH_LENGTH, y)], color)
                    });
            } else {
                chart.draw_series(LineSeries::new(
                    x.iter().zip(y.iter()).map(|(x, y)| (*x, *y)),
                    color,
                ))?;
            }
        }

        if item_labels.is_some() {
            chart
                .configure_series_labels()
                .background_style(WHITE.mix(LEGEND_OPACITY))
                .border_style(BLACK)
                .label_font(AXIS_STYLE.into_font())
                .draw()?;
        }

        root.present()?;
    } // dropping bitmap backend

    if let Some(path) = path {
        image::save_buffer_with_format(
            path,
            &buffer,
            width,
            height,
            image::ColorType::Rgb8,
            image::ImageFormat::Png,
        )?;
    }

    Ok(PngBundle {
        data: buffer,
        width,
        height,
    })
}

#[allow(clippy::cast_precision_loss, clippy::too_many_arguments)]
#[tracing::instrument(level = "trace")]
pub fn log_y_plot<A>(
    x: Option<&Array1<f32>>,
    ys: Vec<&ArrayBase<A, Ix1>>,
    path: Option<&Path>,
    title: Option<&str>,
    y_label: Option<&str>,
    x_label: Option<&str>,
    item_labels: Option<&Vec<&str>>,
    resolution: Option<(u32, u32)>,
) -> Result<PngBundle>
where
    A: Data<Elem = f32>,
{
    trace!("Generating xy plot.");

    let (width, height) = resolution.unwrap_or(STANDARD_RESOLUTION);

    let mut buffer = allocate_buffer(width, height);

    let y_len = ys[0].len();

    for y in &ys {
        if y.len() != y_len {
            return Err(std::io::Error::new(
                io::ErrorKind::InvalidInput,
                "y data must have same length",
            )
            .into());
        }
    }

    if let Some(item_labels) = item_labels {
        if item_labels.len() != ys.len() {
            return Err(std::io::Error::new(
                io::ErrorKind::InvalidInput,
                "if not None, item_labels must be same length as ys",
            )
            .into());
        }
    }

    let default_x = Array1::linspace(0.0, y_len as f32, y_len);
    let x = x.map_or(&default_x, |x| x);

    if x.len() != y_len {
        return Err(std::io::Error::new(
            io::ErrorKind::InvalidInput,
            "x and y must have same length",
        )
        .into());
    }

    let title = title.unwrap_or("Plot");
    let y_label = y_label.unwrap_or("y");
    let x_label = x_label.unwrap_or("x");

    let x_min = x.min()?;
    let x_max = x.max()?;
    let mut y_min = f32::INFINITY;
    let mut y_max = -f32::INFINITY;

    for y in &ys {
        let min = y.min()?;
        let max = y.max()?;
        y_min = y_min.min(*min);
        y_max = y_max.max(*max);
    }

    let x_range = x_max - x_min;

    let x_min = x_min - x_range * X_MARGIN;
    let x_max = x_max + x_range * X_MARGIN;
    let y_min = (y_min * 0.1).max(1e-20);
    let y_max = y_max * 10.0;

    {
        let root = BitMapBackend::with_buffer(&mut buffer[..], (width, height)).into_drawing_area();
        root.fill(&WHITE)?;

        let mut chart = ChartBuilder::on(&root)
            .caption(title, CAPTION_STYLE.into_font())
            .margin(CHART_MARGIN)
            .x_label_area_size(AXIS_LABEL_AREA)
            .y_label_area_size(AXIS_LABEL_AREA)
            .build_cartesian_2d(x_min..x_max, (y_min..y_max).log_scale())?;

        chart
            .configure_mesh()
            .x_desc(x_label)
            .x_label_style(AXIS_STYLE.into_font())
            .y_desc(y_label)
            .y_label_style(AXIS_STYLE.into_font())
            .y_label_formatter(&|y| format!("{y:e}"))
            .draw()?;

        for (i, y) in ys.iter().enumerate() {
            let color = &COLORS[i % COLORS.len()];
            if let Some(item_labels) = item_labels {
                chart
                    .draw_series(LineSeries::new(
                        x.iter().zip(y.iter()).map(|(x, y)| (*x, *y)),
                        color,
                    ))?
                    .label(item_labels[i])
                    .legend(move |(x, y)| {
                        PathElement::new(vec![(x, y), (x + LEGEND_PATH_LENGTH, y)], color)
                    });
            } else {
                chart.draw_series(LineSeries::new(
                    x.iter().zip(y.iter()).map(|(x, y)| (*x, *y)),
                    color,
                ))?;
            }
        }

        if item_labels.is_some() {
            chart
                .configure_series_labels()
                .background_style(WHITE.mix(LEGEND_OPACITY))
                .border_style(BLACK)
                .label_font(AXIS_STYLE.into_font())
                .draw()?;
        }

        root.present()?;
    } // dropping bitmap backend

    if let Some(path) = path {
        image::save_buffer_with_format(
            path,
            &buffer,
            width,
            height,
            image::ColorType::Rgb8,
            image::ImageFormat::Png,
        )?;
    }

    Ok(PngBundle {
        data: buffer,
        width,
        height,
    })
}
