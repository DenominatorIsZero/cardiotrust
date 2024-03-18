use ndarray::{ArrayBase, Axis, Ix2};
use ndarray_stats::QuantileExt;
use plotters::prelude::*;
use scarlet::colormap::{ColorMap, ListedColorMap};
use std::{error::Error, io, path::Path};
use tracing::trace;

use crate::{
    core::model::{
        functional::allpass::shapes::ArrayActivationTime, spatial::voxels::VoxelPositions,
    },
    vis::plotting::{
        allocate_buffer, AXIS_LABEL_AREA, AXIS_LABEL_NUM_MAX, CHART_MARGIN, COLORBAR_BOTTOM_MARGIN,
        COLORBAR_COLOR_NUMBERS, COLORBAR_TOP_MARGIN, COLORBAR_WIDTH, LABEL_AREA_RIGHT_MARGIN,
        LABEL_AREA_WIDTH, UNIT_AREA_TOP_MARGIN,
    },
};

use super::{AXIS_STYLE, CAPTION_STYLE, STANDARD_RESOLUTION};

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
#[tracing::instrument(level = "trace")]
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
) -> Result<Vec<u8>, Box<dyn Error>>
where
    A: ndarray::Data<Elem = f32>,
{
    trace!("Generating matrix plot.");

    let dim_x = data.shape()[0];
    let dim_y = data.shape()[1];

    let (width, height) = resolution.map_or_else(
        || {
            let ratio = (dim_x as f32 / dim_y as f32).clamp(0.1, 10.0);

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

    let (x_step, y_step) = step.map_or((1.0, 1.0), |step| step);

    if x_step <= 0.0 {
        return Err(Box::new(std::io::Error::new(
            io::ErrorKind::InvalidInput,
            "x_step must be greater than zero",
        )));
    }
    if y_step <= 0.0 {
        return Err(Box::new(std::io::Error::new(
            io::ErrorKind::InvalidInput,
            "y_step must be greater than zero",
        )));
    }

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

    Ok(buffer)
}

#[derive(Debug, Clone, Copy)]
pub enum PlotSlice {
    X(usize),
    Y(usize),
    Z(usize),
}

/// Plots the activation time for a given slice (x, y or z) of the
/// activation time matrix.
#[tracing::instrument(level = "trace")]
pub(crate) fn activation_time_plot(
    activation_time_ms: &ArrayActivationTime,
    voxel_positions_mm: &VoxelPositions,
    voxel_size_mm: f32,
    path: &Path,
    slice: Option<PlotSlice>,
) -> Result<Vec<u8>, Box<dyn Error>> {
    trace!("Generating activation time plot");
    let slice = slice.unwrap_or(PlotSlice::Z(0));
    let step = Some((voxel_size_mm, voxel_size_mm));

    let (data, offset, title, x_label, y_label, flip_axis) = match slice {
        PlotSlice::X(index) => {
            let data = activation_time_ms
                .values
                .index_axis(Axis(0), index)
                .map(|value| value.unwrap_or(0.0));
            let offset = Some((
                voxel_positions_mm.values[(0, 0, 0, 1)],
                voxel_positions_mm.values[(0, 0, 0, 2)],
            ));
            let x = voxel_positions_mm.values[(index, 0, 0, 0)];
            let title = format!("Activation time x-index = {index}, x = {x} mm");
            let x_label = Some("y [mm]");
            let y_label = Some("z [mm]");
            let flip_axis = Some((false, false));

            (data, offset, title, x_label, y_label, flip_axis)
        }
        PlotSlice::Y(index) => {
            let data = activation_time_ms
                .values
                .index_axis(Axis(1), index)
                .map(|value| value.unwrap_or(0.0));
            let offset = Some((
                voxel_positions_mm.values[(0, 0, 0, 0)],
                voxel_positions_mm.values[(0, 0, 0, 2)],
            ));
            let y = voxel_positions_mm.values[(0, index, 0, 1)];
            let title = format!("Activation time y-index = {index}, y = {y} mm");
            let x_label = Some("x [mm]");
            let y_label = Some("z [mm]");
            let flip_axis = Some((false, false));

            (data, offset, title, x_label, y_label, flip_axis)
        }
        PlotSlice::Z(index) => {
            let data = activation_time_ms
                .values
                .index_axis(Axis(2), index)
                .map(|value| value.unwrap_or(0.0));
            let offset = Some((
                voxel_positions_mm.values[(0, 0, 0, 0)],
                voxel_positions_mm.values[(0, 0, 0, 1)],
            ));
            let z = voxel_positions_mm.values[(0, 0, index, 2)];
            let title = format!("Activation time z-index = {index}, z = {z} mm");
            let x_label = Some("x [mm]");
            let y_label = Some("y [mm]");
            let flip_axis = Some((false, true));

            (data, offset, title, x_label, y_label, flip_axis)
        }
    };

    matrix_plot(
        &data,
        None,
        step,
        offset,
        Some(path),
        Some(title.as_str()),
        y_label,
        x_label,
        Some("[ms]"),
        None,
        flip_axis,
    )
}

#[cfg(test)]
mod test {

    use std::path::PathBuf;

    use ndarray::Array2;

    use crate::core::{config::simulation::Simulation as SimulationConfig, data::Data};

    use super::*;
    const COMMON_PATH: &str = "tests/vis/plotting/matrix";

    #[tracing::instrument(level = "trace")]
    fn setup() {
        if !Path::new(COMMON_PATH).exists() {
            std::fs::create_dir_all(COMMON_PATH).unwrap();
        }
    }

    #[tracing::instrument(level = "trace")]
    fn clean(files: &Vec<PathBuf>) {
        for file in files {
            if file.is_file() {
                std::fs::remove_file(file).unwrap();
            }
        }
    }

    #[test]
    #[allow(clippy::cast_precision_loss)]
    fn test_matrix_plot_high() {
        setup();
        let files = vec![Path::new(COMMON_PATH).join("test_matrix_plot_high.png")];
        clean(&files);

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
        )
        .unwrap();

        assert!(files[0].is_file());
    }

    #[test]
    #[allow(clippy::cast_precision_loss)]
    fn test_matrix_plot_wide() {
        setup();
        let files = vec![Path::new(COMMON_PATH).join("test_matrix_plot_wide.png")];
        clean(&files);

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
        )
        .unwrap();

        assert!(files[0].is_file());
    }

    #[test]
    #[allow(clippy::cast_precision_loss)]
    fn test_matrix_plot_single_row() {
        setup();
        let files = vec![Path::new(COMMON_PATH).join("test_matrix_plot_single_row.png")];
        clean(&files);

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
        )
        .unwrap();

        assert!(files[0].is_file());
    }

    #[test]
    #[allow(clippy::cast_precision_loss)]
    fn test_matrix_plot_single_column() {
        setup();
        let files = vec![Path::new(COMMON_PATH).join("test_matrix_plot_single_column.png")];
        clean(&files);

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
        )
        .unwrap();

        assert!(files[0].is_file());
    }

    #[test]
    #[allow(clippy::cast_precision_loss)]
    fn test_matrix_plot_large() {
        setup();
        let files = vec![Path::new(COMMON_PATH).join("test_matrix_plot_large.png")];
        clean(&files);

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
        )
        .unwrap();

        assert!(files[0].is_file());
    }

    #[test]
    #[allow(clippy::cast_precision_loss)]
    fn test_matrix_plot_custom_labels() {
        setup();
        let files = vec![Path::new(COMMON_PATH).join("test_matrix_plot_custom_lables.png")];
        clean(&files);

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
        )
        .unwrap();

        assert!(files[0].is_file());
    }

    #[test]
    #[allow(clippy::cast_precision_loss)]
    fn test_matrix_plot_custom_range() {
        setup();
        let files = vec![Path::new(COMMON_PATH).join("test_matrix_plot_custom_range.png")];
        clean(&files);

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
        )
        .unwrap();

        assert!(files[0].is_file());
    }

    #[test]
    #[allow(clippy::cast_precision_loss)]
    fn test_matrix_plot_custom_step() {
        setup();
        let files = vec![Path::new(COMMON_PATH).join("test_matrix_plot_custom_step.png")];
        clean(&files);

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
        )
        .unwrap();

        assert!(files[0].is_file());
    }

    #[test]
    #[allow(clippy::cast_precision_loss)]
    fn test_matrix_plot_custom_offset() {
        setup();
        let files = vec![Path::new(COMMON_PATH).join("test_matrix_plot_custom_offset.png")];
        clean(&files);

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
        )
        .unwrap();

        assert!(files[0].is_file());
    }

    #[test]
    #[allow(clippy::cast_precision_loss)]
    fn test_matrix_plot_invalid_step() {
        setup();
        let files = vec![Path::new(COMMON_PATH).join("test_matrix_plot_invalid_step.png")];
        clean(&files);

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
    }

    #[test]
    #[allow(clippy::cast_precision_loss)]
    fn test_activation_time_plot_default() {
        setup();
        let files = vec![Path::new(COMMON_PATH).join("test_activation_time_plot_default.png")];
        clean(&files);

        let mut simulation_config = SimulationConfig::default();
        simulation_config.model.pathological = true;
        let data = Data::from_simulation_config(&simulation_config)
            .expect("Model parameters to be valid.");

        activation_time_plot(
            data.get_activation_time_ms(),
            &data.get_model().spatial_description.voxels.positions_mm,
            data.get_model().spatial_description.voxels.size_mm,
            files[0].as_path(),
            Some(PlotSlice::Z(0)),
        )
        .unwrap();

        assert!(files[0].is_file());
    }

    #[test]
    #[allow(clippy::cast_precision_loss)]
    fn test_activation_time_plot_x_slice() {
        setup();
        let files = vec![Path::new(COMMON_PATH).join("test_activation_time_plot_x_slice.png")];
        clean(&files);

        let mut simulation_config = SimulationConfig::default();
        simulation_config.model.pathological = true;
        let data = Data::from_simulation_config(&simulation_config)
            .expect("Model parameters to be valid.");

        activation_time_plot(
            data.get_activation_time_ms(),
            &data.get_model().spatial_description.voxels.positions_mm,
            data.get_model().spatial_description.voxels.size_mm,
            files[0].as_path(),
            Some(PlotSlice::X(10)),
        )
        .unwrap();

        assert!(files[0].is_file());
    }

    #[test]
    #[allow(clippy::cast_precision_loss)]
    fn test_activation_time_plot_y_slice() {
        setup();
        let files = vec![Path::new(COMMON_PATH).join("test_activation_time_plot_y_slice.png")];
        clean(&files);

        let mut simulation_config = SimulationConfig::default();
        simulation_config.model.pathological = true;
        let data = Data::from_simulation_config(&simulation_config)
            .expect("Model parameters to be valid.");

        activation_time_plot(
            data.get_activation_time_ms(),
            &data.get_model().spatial_description.voxels.positions_mm,
            data.get_model().spatial_description.voxels.size_mm,
            files[0].as_path(),
            Some(PlotSlice::Y(5)),
        )
        .unwrap();

        assert!(files[0].is_file());
    }
}
