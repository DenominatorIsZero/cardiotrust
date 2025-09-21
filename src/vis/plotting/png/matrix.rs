use std::{f32::consts::PI, io, path::Path};

use anyhow::Result;

use ndarray::{ArrayBase, Ix2};
use ndarray_stats::QuantileExt;
use plotters::prelude::*;
use scarlet::colormap::{ColorMap, ListedColorMap};
use tracing::trace;

use super::PngBundle;
use crate::vis::plotting::{
    allocate_buffer, AXIS_LABEL_AREA, AXIS_LABEL_NUM_MAX, AXIS_STYLE, CAPTION_STYLE, CHART_MARGIN,
    COLORBAR_BOTTOM_MARGIN, COLORBAR_COLOR_NUMBERS, COLORBAR_TOP_MARGIN, COLORBAR_WIDTH,
    LABEL_AREA_RIGHT_MARGIN, LABEL_AREA_WIDTH, STANDARD_RESOLUTION, UNIT_AREA_TOP_MARGIN,
};

/// Generates a 2D matrix plot from the given input data array.
///
/// The matrix values are mapped to colors based on the viridis color map.
/// Additional options allow customizing the axis ranges, labels, title,
/// output resolution, etc. If a file path is provided the plot is saved
/// to that location. The raw pixel buffer is returned.
#[allow(
    clippy::cast_precision_loss,
    clippy::too_many_arguments,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap,
    clippy::cast_lossless
)]
#[tracing::instrument(level = "trace", skip(data))]
pub fn matrix_plot<A>(
    data: &ArrayBase<A, Ix2>,
    range: Option<(f32, f32)>,
    step: Option<(f32, f32)>,
    offset: Option<(f32, f32)>,
    path: Option<&Path>,
    title: Option<&str>,
    y_label: Option<&str>,
    x_label: Option<&str>,
    unit: Option<&str>,
    resolution: Option<(u32, u32)>,
    flip_axis: Option<(bool, bool)>,
) -> Result<PngBundle>
where
    A: ndarray::Data<Elem = f32>,
{
    trace!("Generating matrix plot.");

    let (x_step, y_step) = step.map_or((1.0, 1.0), |step| step);

    if x_step <= 0.0 {
        return Err(std::io::Error::new(
            io::ErrorKind::InvalidInput,
            "x_step must be greater than zero",
        )
        .into());
    }
    if y_step <= 0.0 {
        return Err(std::io::Error::new(
            io::ErrorKind::InvalidInput,
            "y_step must be greater than zero",
        )
        .into());
    }

    let dim_x = data.shape()[0];
    let dim_y = data.shape()[1];

    let (width, height) = resolution.map_or_else(
        || {
            let ratio = ((dim_x as f32 * x_step) / (dim_y as f32 * y_step)).clamp(0.1, 10.0);

            if ratio > 1.0 {
                (
                    STANDARD_RESOLUTION.0
                        + AXIS_LABEL_AREA
                        + CHART_MARGIN
                        + COLORBAR_WIDTH
                        + LABEL_AREA_WIDTH
                        + LABEL_AREA_RIGHT_MARGIN,
                    (STANDARD_RESOLUTION.0 as f32 / ratio) as u32
                        + AXIS_LABEL_AREA
                        + CHART_MARGIN
                        + CAPTION_STYLE.1 as u32,
                )
            } else {
                (
                    (STANDARD_RESOLUTION.0 as f32 * ratio) as u32
                        + AXIS_LABEL_AREA
                        + CHART_MARGIN
                        + COLORBAR_WIDTH
                        + LABEL_AREA_WIDTH
                        + LABEL_AREA_RIGHT_MARGIN,
                    STANDARD_RESOLUTION.0 + AXIS_LABEL_AREA + CHART_MARGIN + CAPTION_STYLE.1 as u32,
                )
            }
        },
        |resolution| resolution,
    );

    let mut buffer = allocate_buffer(width, height);

    let (x_offset, y_offset) = offset.map_or((0.0, 0.0), |offset| offset);
    let (flip_x, flip_y) = flip_axis.map_or((false, false), |flip_axis| flip_axis);

    let title = title.unwrap_or("Plot");
    let y_label = y_label.unwrap_or("y");
    let x_label = x_label.unwrap_or("x");
    let unit = unit.unwrap_or("[a.u.]");

    let (data_min, data_max) = if let Some(range) = range {
        range
    } else {
        (*data.min()?, *data.max()?)
    };

    let data_range = (data_max - data_min).max(f32::EPSILON);

    let x_min = x_offset - x_step / 2.0;
    let x_max = (dim_x as f32).mul_add(x_step, x_offset - x_step / 2.0);
    let y_min = y_offset - y_step / 2.0;
    let y_max = (dim_y as f32).mul_add(y_step, y_offset - y_step / 2.0);

    let x_range = if flip_x { x_max..x_min } else { x_min..x_max };
    let y_range = if flip_y { y_max..y_min } else { y_min..y_max };

    let color_map = ListedColorMap::viridis();

    {
        let root = BitMapBackend::with_buffer(&mut buffer[..], (width, height)).into_drawing_area();
        root.fill(&WHITE)?;
        let (root_width, root_height) = root.dim_in_pixel();

        let colorbar_area = root.margin(
            COLORBAR_TOP_MARGIN,
            COLORBAR_BOTTOM_MARGIN,
            root_width - COLORBAR_WIDTH - LABEL_AREA_WIDTH - LABEL_AREA_RIGHT_MARGIN,
            LABEL_AREA_WIDTH + LABEL_AREA_RIGHT_MARGIN,
        );

        let (colorbar_width, colorbar_height) = colorbar_area.dim_in_pixel();

        for i in 0..COLORBAR_COLOR_NUMBERS {
            let color: scarlet::color::RGBColor =
                color_map.transform_single(1.0 - i as f64 / (COLORBAR_COLOR_NUMBERS - 1) as f64);
            let color = RGBColor(
                (color.r * u8::MAX as f64) as u8,
                (color.g * u8::MAX as f64) as u8,
                (color.b * u8::MAX as f64) as u8,
            );
            colorbar_area.draw(&Rectangle::new(
                [
                    (0, (i * colorbar_height / COLORBAR_COLOR_NUMBERS) as i32),
                    (
                        colorbar_width as i32,
                        ((i + 1) * colorbar_height / COLORBAR_COLOR_NUMBERS) as i32,
                    ),
                ],
                color.filled(),
            ))?;
        }

        // Drawing labels for the colorbar
        let label_area = root.margin(
            COLORBAR_TOP_MARGIN,
            COLORBAR_BOTTOM_MARGIN,
            root_width - LABEL_AREA_WIDTH,
            LABEL_AREA_RIGHT_MARGIN,
        ); // Adjust margins to align with the colorbar
        let num_labels = 4; // Number of labels on the colorbar
        for i in 0..=num_labels {
            label_area.draw(&Text::new(
                format!(
                    "{:.2}",
                    (i as f32 / num_labels as f32).mul_add(-data_range, data_max)
                ),
                (5, (i * colorbar_height / num_labels) as i32),
                AXIS_STYLE.into_font(),
            ))?;
        }

        // Drawing units for colorbar
        let unit_area = root.margin(
            root_height - colorbar_height - COLORBAR_TOP_MARGIN - COLORBAR_BOTTOM_MARGIN,
            UNIT_AREA_TOP_MARGIN,
            root_width - COLORBAR_WIDTH - LABEL_AREA_WIDTH - LABEL_AREA_RIGHT_MARGIN,
            LABEL_AREA_WIDTH + LABEL_AREA_RIGHT_MARGIN,
        ); // Adjust margins to align with the colorbar
        unit_area.draw(&Text::new(
            unit,
            (
                COLORBAR_WIDTH as i32 / 2 - AXIS_STYLE.1,
                COLORBAR_TOP_MARGIN as i32 / 2,
            ),
            AXIS_STYLE.into_font(),
        ))?;

        let mut chart = ChartBuilder::on(&root)
            .caption(title, CAPTION_STYLE.into_font())
            .margin(CHART_MARGIN)
            .margin_right(
                CHART_MARGIN + COLORBAR_WIDTH + LABEL_AREA_WIDTH + LABEL_AREA_RIGHT_MARGIN,
            ) // make room for colorbar
            .x_label_area_size(AXIS_LABEL_AREA)
            .y_label_area_size(AXIS_LABEL_AREA)
            .build_cartesian_2d(x_range, y_range)?;

        chart
            .configure_mesh()
            .disable_mesh()
            .x_desc(x_label)
            .x_label_style(AXIS_STYLE.into_font())
            .x_labels(dim_x.min(AXIS_LABEL_NUM_MAX))
            .y_desc(y_label)
            .y_label_style(AXIS_STYLE.into_font())
            .y_labels(dim_y.min(AXIS_LABEL_NUM_MAX))
            .draw()?;

        chart.draw_series(data.indexed_iter().map(|((index_x, index_y), &value)| {
            // Map the value to a color
            let color_value = (value - data_min) / (data_range);
            let color: scarlet::color::RGBColor =
                color_map.transform_single(f64::from(color_value));
            let color = RGBColor(
                (color.r * u8::MAX as f64) as u8,
                (color.g * u8::MAX as f64) as u8,
                (color.b * u8::MAX as f64) as u8,
            );
            let start = (
                (index_x as f32).mul_add(x_step, x_offset - x_step / 2.0),
                (index_y as f32).mul_add(y_step, y_offset - y_step / 2.0),
            );
            let end = (
                ((index_x + 1) as f32).mul_add(x_step, x_offset - x_step / 2.0),
                ((index_y + 1) as f32).mul_add(y_step, y_offset - y_step / 2.0),
            );
            Rectangle::new([start, end], color.filled())
        }))?;

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

#[allow(
    clippy::cast_precision_loss,
    clippy::too_many_arguments,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap,
    clippy::cast_lossless
)]
#[tracing::instrument(level = "trace")]
pub fn matrix_angle_plot<A>(
    theta: &ArrayBase<A, Ix2>,
    phi: &ArrayBase<A, Ix2>,
    step: Option<(f32, f32)>,
    offset: Option<(f32, f32)>,
    path: Option<&Path>,
    title: Option<&str>,
    y_label: Option<&str>,
    x_label: Option<&str>,
    resolution: Option<(u32, u32)>,
    flip_axis: Option<(bool, bool)>,
) -> Result<PngBundle>
where
    A: ndarray::Data<Elem = f32>,
{
    trace!("Generating matrix angle plot.");

    if theta.shape() != phi.shape() {
        return Err(anyhow::anyhow!(
            "Theta and phi arrays must have the same shape, but theta is {:?} and phi is {:?}",
            theta.shape(),
            phi.shape()
        ));
    }

    let dim_x = theta.shape()[0];
    let dim_y = theta.shape()[1];

    let (width, height) = resolution.map_or_else(
        || {
            let ratio = (dim_x as f32 / dim_y as f32).clamp(0.1, 10.0);

            if ratio > 1.0 {
                (
                    STANDARD_RESOLUTION.0
                        + AXIS_LABEL_AREA
                        + CHART_MARGIN
                        + 2 * COLORBAR_WIDTH
                        + 2 * LABEL_AREA_WIDTH
                        + 2 * LABEL_AREA_RIGHT_MARGIN,
                    (STANDARD_RESOLUTION.0 as f32 / ratio) as u32
                        + AXIS_LABEL_AREA
                        + CHART_MARGIN
                        + CAPTION_STYLE.1 as u32,
                )
            } else {
                (
                    (STANDARD_RESOLUTION.0 as f32 * ratio) as u32
                        + AXIS_LABEL_AREA
                        + CHART_MARGIN
                        + 2 * COLORBAR_WIDTH
                        + 2 * LABEL_AREA_WIDTH
                        + 2 * LABEL_AREA_RIGHT_MARGIN,
                    STANDARD_RESOLUTION.0 + AXIS_LABEL_AREA + CHART_MARGIN + CAPTION_STYLE.1 as u32,
                )
            }
        },
        |resolution| resolution,
    );

    let mut buffer = allocate_buffer(width, height);

    let (x_step, y_step) = step.map_or((1.0, 1.0), |step| step);

    if x_step <= 0.0 {
        return Err(std::io::Error::new(
            io::ErrorKind::InvalidInput,
            "x_step must be greater than zero",
        )
        .into());
    }
    if y_step <= 0.0 {
        return Err(std::io::Error::new(
            io::ErrorKind::InvalidInput,
            "y_step must be greater than zero",
        )
        .into());
    }

    let (x_offset, y_offset) = offset.map_or((0.0, 0.0), |offset| offset);
    let (flip_x, flip_y) = flip_axis.map_or((false, false), |flip_axis| flip_axis);

    let title = title.unwrap_or("Plot");
    let y_label = y_label.unwrap_or("y");
    let x_label = x_label.unwrap_or("x");

    let x_min = x_offset - x_step / 2.0;
    let x_max = (dim_x as f32).mul_add(x_step, x_offset - x_step / 2.0);
    let y_min = y_offset - y_step / 2.0;
    let y_max = (dim_y as f32).mul_add(y_step, y_offset - y_step / 2.0);

    let x_range = if flip_x { x_max..x_min } else { x_min..x_max };
    let y_range = if flip_y { y_max..y_min } else { y_min..y_max };

    {
        let root = BitMapBackend::with_buffer(&mut buffer[..], (width, height)).into_drawing_area();
        root.fill(&WHITE)?;
        let (root_width, root_height) = root.dim_in_pixel();

        let colorbar_phi_area = root.margin(
            COLORBAR_TOP_MARGIN,
            COLORBAR_BOTTOM_MARGIN,
            root_width - 2 * COLORBAR_WIDTH - 2 * LABEL_AREA_WIDTH - 2 * LABEL_AREA_RIGHT_MARGIN,
            2 * LABEL_AREA_WIDTH + 2 * LABEL_AREA_RIGHT_MARGIN + COLORBAR_WIDTH,
        );

        let (colorbar_phi_width, colorbar_phi_height) = colorbar_phi_area.dim_in_pixel();

        for i in 0..COLORBAR_COLOR_NUMBERS {
            let h = (i as f64 / COLORBAR_COLOR_NUMBERS as f64 + 0.5) % 1.0;
            let v = 0.5;
            let s = 1.0;
            // Map the value to a color
            let color = HSLColor(h, s, v);
            colorbar_phi_area.draw(&Rectangle::new(
                [
                    (0, (i * colorbar_phi_height / COLORBAR_COLOR_NUMBERS) as i32),
                    (
                        colorbar_phi_width as i32,
                        ((i + 1) * colorbar_phi_height / COLORBAR_COLOR_NUMBERS) as i32,
                    ),
                ],
                color.filled(),
            ))?;
        }

        // Drawing labels for the colorbar
        let label_area_phi = root.margin(
            COLORBAR_TOP_MARGIN,
            COLORBAR_BOTTOM_MARGIN,
            root_width - 2 * LABEL_AREA_WIDTH - LABEL_AREA_RIGHT_MARGIN - COLORBAR_WIDTH,
            LABEL_AREA_RIGHT_MARGIN,
        ); // Adjust margins to align with the colorbar
        let num_labels = 4; // Number of labels on the colorbar
        for i in 0..=num_labels {
            label_area_phi.draw(&Text::new(
                format!(
                    "{:.2}",
                    (i as f32 / num_labels as f32).mul_add(-360.0, 360.0)
                ),
                (5, (i * colorbar_phi_height / num_labels) as i32),
                AXIS_STYLE.into_font(),
            ))?;
        }

        // Drawing units for colorbar
        let unit_area_phi = root.margin(
            root_height - colorbar_phi_height - COLORBAR_TOP_MARGIN - COLORBAR_BOTTOM_MARGIN,
            UNIT_AREA_TOP_MARGIN,
            root_width - 2 * COLORBAR_WIDTH - 2 * LABEL_AREA_WIDTH - 2 * LABEL_AREA_RIGHT_MARGIN,
            LABEL_AREA_WIDTH + LABEL_AREA_RIGHT_MARGIN + LABEL_AREA_WIDTH + LABEL_AREA_RIGHT_MARGIN,
        ); // Adjust margins to align with the colorbar
        unit_area_phi.draw(&Text::new(
            "phi [°]",
            (
                COLORBAR_WIDTH as i32 / 2 - AXIS_STYLE.1,
                COLORBAR_TOP_MARGIN as i32 / 2,
            ),
            AXIS_STYLE.into_font(),
        ))?;

        let mut chart = ChartBuilder::on(&root)
            .caption(title, CAPTION_STYLE.into_font())
            .margin(CHART_MARGIN)
            .margin_right(
                CHART_MARGIN
                    + 2 * COLORBAR_WIDTH
                    + 2 * LABEL_AREA_WIDTH
                    + 2 * LABEL_AREA_RIGHT_MARGIN,
            ) // make room for colorbar
            .x_label_area_size(AXIS_LABEL_AREA)
            .y_label_area_size(AXIS_LABEL_AREA)
            .build_cartesian_2d(x_range, y_range)?;

        let colorbar_theta_area = root.margin(
            COLORBAR_TOP_MARGIN,
            COLORBAR_BOTTOM_MARGIN,
            root_width - COLORBAR_WIDTH - LABEL_AREA_WIDTH - LABEL_AREA_RIGHT_MARGIN,
            LABEL_AREA_WIDTH + LABEL_AREA_RIGHT_MARGIN,
        );

        let (colorbar_theta_width, colorbar_theta_height) = colorbar_theta_area.dim_in_pixel();

        for i in 0..COLORBAR_COLOR_NUMBERS {
            let h = 0.5;
            let v = i as f64 / COLORBAR_COLOR_NUMBERS as f64;
            let s = 1.0;
            // Map the value to a color
            let color = HSLColor(h, s, v);
            colorbar_theta_area.draw(&Rectangle::new(
                [
                    (
                        0,
                        (i * colorbar_theta_height / COLORBAR_COLOR_NUMBERS) as i32,
                    ),
                    (
                        colorbar_theta_width as i32,
                        ((i + 1) * colorbar_theta_height / COLORBAR_COLOR_NUMBERS) as i32,
                    ),
                ],
                color.filled(),
            ))?;
        }

        // Drawing labels for the colorbar
        let label_area_theta = root.margin(
            COLORBAR_TOP_MARGIN,
            COLORBAR_BOTTOM_MARGIN,
            root_width - LABEL_AREA_WIDTH - LABEL_AREA_RIGHT_MARGIN,
            LABEL_AREA_RIGHT_MARGIN,
        ); // Adjust margins to align with the colorbar
        let num_labels = 4; // Number of labels on the colorbar
        for i in 0..=num_labels {
            label_area_theta.draw(&Text::new(
                format!(
                    "{:.2}",
                    (i as f32 / num_labels as f32).mul_add(-180.0, 180.0)
                ),
                (5, (i * colorbar_theta_height / num_labels) as i32),
                AXIS_STYLE.into_font(),
            ))?;
        }

        // Drawing units for colorbar
        let unit_area_theta = root.margin(
            root_height - colorbar_theta_height - COLORBAR_TOP_MARGIN - COLORBAR_BOTTOM_MARGIN,
            UNIT_AREA_TOP_MARGIN,
            root_width - COLORBAR_WIDTH - LABEL_AREA_WIDTH - LABEL_AREA_RIGHT_MARGIN,
            LABEL_AREA_WIDTH + LABEL_AREA_RIGHT_MARGIN,
        ); // Adjust margins to align with the colorbar
        unit_area_theta.draw(&Text::new(
            "theta [°]",
            (
                COLORBAR_WIDTH as i32 / 2 - AXIS_STYLE.1,
                COLORBAR_TOP_MARGIN as i32 / 2,
            ),
            AXIS_STYLE.into_font(),
        ))?;

        chart
            .configure_mesh()
            .disable_mesh()
            .x_desc(x_label)
            .x_label_style(AXIS_STYLE.into_font())
            .x_labels(dim_x.min(AXIS_LABEL_NUM_MAX))
            .y_desc(y_label)
            .y_label_style(AXIS_STYLE.into_font())
            .y_labels(dim_y.min(AXIS_LABEL_NUM_MAX))
            .draw()?;

        chart.draw_series(theta.indexed_iter().map(|((index_x, index_y), &theta)| {
            let h = (phi[(index_x, index_y)] + PI) / (2.0 * PI);
            let v = theta / PI;
            let s = 1.0;
            // Map the value to a color
            let color = HSLColor(h as f64, s, v as f64);
            let start = (
                (index_x as f32).mul_add(x_step, x_offset - x_step / 2.0),
                (index_y as f32).mul_add(y_step, y_offset - y_step / 2.0),
            );
            let end = (
                ((index_x + 1) as f32).mul_add(x_step, x_offset - x_step / 2.0),
                ((index_y + 1) as f32).mul_add(y_step, y_offset - y_step / 2.0),
            );
            Rectangle::new([start, end], color.filled())
        }))?;

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
#[cfg(test)]
mod test {

    use ndarray::Array2;

    use super::*;
    use crate::tests::{clean_files, setup_folder};
    const COMMON_PATH: &str = "tests/vis/plotting/png/matrix";

    #[test]
    #[allow(clippy::cast_precision_loss)]
    fn test_matrix_plot_high() -> Result<()> {
        let path = Path::new(COMMON_PATH);
        setup_folder(path.to_path_buf())?;
        let files = vec![path.join("matrix_plot_high.png")];
        clean_files(&files)?;

        let mut data = Array2::zeros((4, 8));

        for x in 0..4 {
            for y in 0..8 {
                data[(x, y)] = ((x + 1) + (y * 4)) as f32;
            }
        }

        matrix_plot(
            &data,
            None,
            None,
            None,
            Some(files[0].as_path()),
            None,
            None,
            None,
            None,
            None,
            None,
        )?;

        assert!(files[0].is_file());
        Ok(())
    }

    #[test]
    #[allow(clippy::cast_precision_loss)]
    fn test_matrix_plot_wide() -> Result<()> {
        let path = Path::new(COMMON_PATH);
        setup_folder(path.to_path_buf())?;
        let files = vec![path.join("matrix_plot_wide.png")];
        clean_files(&files)?;

        let mut data = Array2::zeros((8, 4));

        for x in 0..8 {
            for y in 0..4 {
                data[(x, y)] = ((x + 1) + (y * 8)) as f32;
            }
        }

        matrix_plot(
            &data,
            None,
            None,
            None,
            Some(files[0].as_path()),
            None,
            None,
            None,
            None,
            None,
            None,
        )?;

        assert!(files[0].is_file());
        Ok(())
    }

    #[test]
    #[allow(clippy::cast_precision_loss)]
    fn test_matrix_plot_single_row() -> Result<()> {
        let path = Path::new(COMMON_PATH);
        setup_folder(path.to_path_buf())?;
        let files = vec![path.join("matrix_plot_single_row.png")];
        clean_files(&files)?;

        let mut data = Array2::zeros((8, 1));

        for x in 0..8 {
            for y in 0..1 {
                data[(x, y)] = ((x + 1) + (y * 8)) as f32;
            }
        }

        matrix_plot(
            &data,
            None,
            None,
            None,
            Some(files[0].as_path()),
            None,
            None,
            None,
            None,
            None,
            None,
        )?;

        assert!(files[0].is_file());
        Ok(())
    }

    #[test]
    #[allow(clippy::cast_precision_loss)]
    fn test_matrix_plot_single_column() -> Result<()> {
        let path = Path::new(COMMON_PATH);
        setup_folder(path.to_path_buf())?;
        let files = vec![path.join("matrix_plot_single_column.png")];
        clean_files(&files)?;

        let mut data = Array2::zeros((1, 8));

        for x in 0..1 {
            for y in 0..8 {
                data[(x, y)] = ((x + 1) + (y * 8)) as f32;
            }
        }

        matrix_plot(
            &data,
            None,
            None,
            None,
            Some(files[0].as_path()),
            None,
            None,
            None,
            None,
            None,
            None,
        )?;

        assert!(files[0].is_file());
        Ok(())
    }

    #[test]
    #[allow(clippy::cast_precision_loss)]
    fn test_matrix_plot_large() -> Result<()> {
        let path = Path::new(COMMON_PATH);
        setup_folder(path.to_path_buf())?;
        let files = vec![path.join("matrix_plot_large.png")];
        clean_files(&files)?;

        let mut data = Array2::zeros((1000, 1000));

        for x in 0..1000 {
            for y in 0..1000 {
                data[(x, y)] = ((x + 1) + (y * 1000)) as f32;
            }
        }

        matrix_plot(
            &data,
            None,
            None,
            None,
            Some(files[0].as_path()),
            None,
            None,
            None,
            None,
            None,
            None,
        )?;

        assert!(files[0].is_file());
        Ok(())
    }

    #[test]
    #[allow(clippy::cast_precision_loss)]
    fn test_matrix_plot_custom_labels() -> Result<()> {
        let path = Path::new(COMMON_PATH);
        setup_folder(path.to_path_buf())?;
        let files = vec![path.join("matrix_plot_custom_lables.png")];
        clean_files(&files)?;

        let data = Array2::zeros((4, 4));

        matrix_plot(
            &data,
            None,
            None,
            None,
            Some(files[0].as_path()),
            Some("Custom Title"),
            Some("Custom X"),
            Some("Custom Y"),
            Some("Custom Unit"),
            None,
            None,
        )?;

        assert!(files[0].is_file());
        Ok(())
    }

    #[test]
    #[allow(clippy::cast_precision_loss)]
    fn test_matrix_plot_custom_range() -> Result<()> {
        let path = Path::new(COMMON_PATH);
        setup_folder(path.to_path_buf())?;
        let files = vec![path.join("matrix_plot_custom_range.png")];
        clean_files(&files)?;

        let mut data = Array2::zeros((4, 4));
        data[(0, 0)] = 5.0;

        matrix_plot(
            &data,
            Some((0.0, 10.0)),
            None,
            None,
            Some(files[0].as_path()),
            None,
            None,
            None,
            None,
            None,
            None,
        )?;

        assert!(files[0].is_file());
        Ok(())
    }

    #[test]
    #[allow(clippy::cast_precision_loss)]
    fn test_matrix_plot_custom_step() -> Result<()> {
        let path = Path::new(COMMON_PATH);
        setup_folder(path.to_path_buf())?;
        let files = vec![path.join("matrix_plot_custom_step.png")];
        clean_files(&files)?;

        let mut data = Array2::zeros((4, 4));
        data[(0, 0)] = 5.0;

        matrix_plot(
            &data,
            None,
            Some((0.25, 0.25)),
            None,
            Some(files[0].as_path()),
            None,
            None,
            None,
            None,
            None,
            None,
        )?;

        assert!(files[0].is_file());
        Ok(())
    }

    #[test]
    #[allow(clippy::cast_precision_loss)]
    fn test_matrix_plot_custom_offset() -> Result<()> {
        let path = Path::new(COMMON_PATH);
        setup_folder(path.to_path_buf())?;
        let files = vec![path.join("matrix_plot_custom_offset.png")];
        clean_files(&files)?;

        let mut data = Array2::zeros((4, 4));
        data[(0, 0)] = 5.0;

        matrix_plot(
            &data,
            None,
            None,
            Some((10.0, 100.0)),
            Some(files[0].as_path()),
            None,
            None,
            None,
            None,
            None,
            None,
        )?;

        assert!(files[0].is_file());
        Ok(())
    }

    #[test]
    #[allow(clippy::cast_precision_loss)]
    fn test_matrix_plot_invalid_step() -> anyhow::Result<()> {
        let path = Path::new(COMMON_PATH);
        setup_folder(path.to_path_buf())?;
        let files = vec![path.join("matrix_plot_invalid_step.png")];
        clean_files(&files)?;

        let mut data = Array2::zeros((4, 4));
        data[(0, 0)] = 5.0;

        let results = matrix_plot(
            &data,
            None,
            Some((0.0, 1.0)),
            None,
            Some(files[0].as_path()),
            None,
            None,
            None,
            None,
            None,
            None,
        );

        assert!(results.is_err());
        assert!(!files[0].is_file());
        Ok(())
    }
}
