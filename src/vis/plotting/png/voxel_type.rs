use ndarray::Axis;
use plotters::prelude::*;
use scarlet::colormap::ListedColorMap;
use std::{error::Error, path::Path};
use strum::IntoEnumIterator;
use tracing::trace;

use crate::{
    core::model::spatial::voxels::{VoxelPositions, VoxelType, VoxelTypes},
    vis::{
        heart::type_to_color,
        plotting::{
            allocate_buffer, PlotSlice, AXIS_LABEL_AREA, AXIS_LABEL_NUM_MAX, CHART_MARGIN,
            COLORBAR_BOTTOM_MARGIN, COLORBAR_TOP_MARGIN, COLORBAR_WIDTH, LABEL_AREA_RIGHT_MARGIN,
            LABEL_AREA_WIDTH,
        },
    },
};

use crate::vis::plotting::{AXIS_STYLE, CAPTION_STYLE, STANDARD_RESOLUTION};

use super::PngBundle;

#[allow(
    clippy::cast_precision_loss,
    clippy::too_many_arguments,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap,
    clippy::cast_lossless
)]
#[tracing::instrument(level = "trace")]
pub fn voxel_type_plot(
    types: &VoxelTypes,
    voxel_positions_mm: &VoxelPositions,
    voxel_size_mm: f32,
    path: Option<&Path>,
    slice: Option<PlotSlice>,
) -> Result<PngBundle, Box<dyn Error>> {
    trace!("Generating voxel type plot.");

    let slice = slice.unwrap_or(PlotSlice::Z(0));

    let (data, offset, title, x_label, y_label, flip_axis) = match slice {
        PlotSlice::X(index) => {
            let data = types.values.index_axis(Axis(0), index);
            let offset = (
                voxel_positions_mm.values[(0, 0, 0, 1)],
                voxel_positions_mm.values[(0, 0, 0, 2)],
            );
            let x = voxel_positions_mm.values[(index, 0, 0, 0)];
            let title = format!("Voxel types x-index = {index}, x = {x} mm");
            let x_label = Some("y [mm]");
            let y_label = Some("z [mm]");
            let flip_axis = (true, false);

            (data, offset, title, x_label, y_label, flip_axis)
        }
        PlotSlice::Y(index) => {
            let data = types.values.index_axis(Axis(1), index);
            let offset = (
                voxel_positions_mm.values[(0, 0, 0, 0)],
                voxel_positions_mm.values[(0, 0, 0, 2)],
            );
            let y = voxel_positions_mm.values[(0, index, 0, 1)];
            let title = format!("Voxel types y-index = {index}, y = {y} mm");
            let x_label = Some("x [mm]");
            let y_label = Some("z [mm]");
            let flip_axis = (false, false);

            (data, offset, title, x_label, y_label, flip_axis)
        }
        PlotSlice::Z(index) => {
            let data = types.values.index_axis(Axis(2), index);
            let offset = (
                voxel_positions_mm.values[(0, 0, 0, 0)],
                voxel_positions_mm.values[(0, 0, 0, 1)],
            );
            let z = voxel_positions_mm.values[(0, 0, index, 2)];
            let title = format!("Voxel types z-index = {index}, z = {z} mm");
            let x_label = Some("x [mm]");
            let y_label = Some("y [mm]");
            let flip_axis = (false, false);

            (data, offset, title, x_label, y_label, flip_axis)
        }
    };

    let dim_x = data.shape()[0];
    let dim_y = data.shape()[1];

    let (width, height) = {
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
    };

    let mut buffer = allocate_buffer(width, height);

    let (x_step, y_step) = (voxel_size_mm, voxel_size_mm);

    let (x_offset, y_offset) = offset;
    let (flip_x, flip_y) = flip_axis;

    let y_label = y_label.unwrap_or("y");
    let x_label = x_label.unwrap_or("x");

    let x_min = x_offset - x_step / 2.0;
    let x_max = (dim_x as f32).mul_add(x_step, x_offset - x_step / 2.0);
    let y_min = y_offset - y_step / 2.0;
    let y_max = (dim_y as f32).mul_add(y_step, y_offset - y_step / 2.0);

    let x_range = if flip_x { x_max..x_min } else { x_min..x_max };
    let y_range = if flip_y { y_max..y_min } else { y_min..y_max };

    let _color_map = ListedColorMap::viridis();

    {
        let root = BitMapBackend::with_buffer(&mut buffer[..], (width, height)).into_drawing_area();
        root.fill(&WHITE)?;
        let (root_width, _root_height) = root.dim_in_pixel();

        let legend_area = root.margin(
            COLORBAR_TOP_MARGIN,
            COLORBAR_BOTTOM_MARGIN,
            root_width - COLORBAR_WIDTH - LABEL_AREA_WIDTH - LABEL_AREA_RIGHT_MARGIN,
            LABEL_AREA_WIDTH + LABEL_AREA_RIGHT_MARGIN,
        );

        let (legend_width, legend_height) = legend_area.dim_in_pixel();

        let num_types = VoxelType::iter().count() as u32;
        let single_space = (legend_height / (2 * num_types - 1)) as i32;

        for (i, voxel_type) in VoxelType::iter().enumerate() {
            let color = type_to_color(voxel_type);
            let color = color.as_rgba_u8();
            let color = RGBColor(color[0], color[1], color[2]);
            let start = (
                legend_width as i32 / 2 - single_space / 2,
                i as i32 * (single_space + single_space),
            );
            let end = (
                legend_width as i32 / 2 + single_space / 2,
                i as i32 * (single_space + single_space) + single_space,
            );
            legend_area.draw(&Rectangle::new([start, end], color.filled()))?;
            legend_area.draw(&Rectangle::new([start, end], BLACK))?;
            legend_area.draw(&Text::new(
                format!("{voxel_type:?}"),
                (
                    legend_width as i32 / 2 + single_space * 2 / 3,
                    i as i32 * (single_space + single_space) + single_space / 2 - AXIS_STYLE.1 / 2,
                ),
                AXIS_STYLE.into_font(),
            ))?;
        }

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
            let color = type_to_color(value);
            let color = color.as_rgba_u8();
            let color = RGBColor(color[0], color[1], color[2]);
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

    use crate::{
        core::{config::simulation::Simulation as SimulationConfig, data::Data},
        tests::{clean_files, setup_folder},
    };

    use super::*;
    const COMMON_PATH: &str = "tests/vis/plotting/png/voxel_types";

    #[test]
    #[allow(clippy::cast_precision_loss)]
    fn test_voxel_type_plot_default() {
        let path = Path::new(COMMON_PATH);
        setup_folder(path.to_path_buf());
        let files = vec![path.join("types_default.png")];
        clean_files(&files);

        let mut simulation_config = SimulationConfig::default();
        simulation_config.model.common.pathological = true;
        let data = Data::from_simulation_config(&simulation_config)
            .expect("Model parameters to be valid.");

        voxel_type_plot(
            data.get_voxel_types(),
            &data.get_model().spatial_description.voxels.positions_mm,
            data.get_model().spatial_description.voxels.size_mm,
            Some(files[0].as_path()),
            None,
        )
        .unwrap();

        assert!(files[0].is_file());
    }

    #[test]
    #[allow(clippy::cast_precision_loss)]
    fn test_voxel_type_plot_x_slice() {
        let path = Path::new(COMMON_PATH);
        setup_folder(path.to_path_buf());
        let files = vec![path.join("types_x_slice.png")];
        clean_files(&files);

        let mut simulation_config = SimulationConfig::default();
        simulation_config.model.common.pathological = true;
        let data = Data::from_simulation_config(&simulation_config)
            .expect("Model parameters to be valid.");

        voxel_type_plot(
            data.get_voxel_types(),
            &data.get_model().spatial_description.voxels.positions_mm,
            data.get_model().spatial_description.voxels.size_mm,
            Some(files[0].as_path()),
            Some(PlotSlice::X(10)),
        )
        .unwrap();

        assert!(files[0].is_file());
    }

    #[test]
    #[allow(clippy::cast_precision_loss)]
    fn test_voxel_type_plot_y_slice() {
        let path = Path::new(COMMON_PATH);
        setup_folder(path.to_path_buf());
        let files = vec![path.join("types_y_slice.png")];
        clean_files(&files);

        let mut simulation_config = SimulationConfig::default();
        simulation_config.model.common.pathological = true;
        let data = Data::from_simulation_config(&simulation_config)
            .expect("Model parameters to be valid.");

        voxel_type_plot(
            data.get_voxel_types(),
            &data.get_model().spatial_description.voxels.positions_mm,
            data.get_model().spatial_description.voxels.size_mm,
            Some(files[0].as_path()),
            Some(PlotSlice::Y(5)),
        )
        .unwrap();

        assert!(files[0].is_file());
    }
}
