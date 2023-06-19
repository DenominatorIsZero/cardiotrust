use ndarray::{Array1, Array2, Array3};
use ndarray_stats::QuantileExt;
use plotly::{
    common::{ColorScale, Mode},
    layout::Axis,
    HeatMap, Layout, Plot, Scatter,
};

use crate::core::model::{
    functional::allpass::shapes::ArrayActivationTime, spatial::voxels::VoxelType,
};

pub fn standard_time_plot(
    y: &Array1<f32>,
    sample_rate_hz: f32,
    file_name: &str,
    title: &str,
    y_label: &str,
) {
    let x = Array1::from_vec(
        (0..y.shape()[0])
            .map(|i| i as f32 / sample_rate_hz)
            .collect(),
    );
    let x_min = *x.min_skipnan();
    let x_max = *x.max_skipnan();
    let mut y_min = *y.min_skipnan();
    let mut y_max = *y.max_skipnan();
    let y_range = y_max - y_min;
    let y_margin = 0.1;
    y_min = y_min - y_margin * y_range;
    y_max = y_max + y_margin * y_range;

    let mut plot = Plot::new();
    let trace = Scatter::from_array(x, y.clone()).mode(Mode::Lines);
    plot.add_trace(trace);
    let layout = Layout::new()
        .title(title.into())
        .x_axis(
            Axis::new()
                .title("t [s]".into())
                .range(vec![x_min, x_max])
                .show_spikes(true),
        )
        .y_axis(Axis::new().title(y_label.into()).range(vec![y_min, y_max]));
    plot.set_layout(layout);
    plot.write_html(format!("{file_name}.html"));
    plot.write_image(
        format!("{file_name}.png"),
        plotly::ImageFormat::PNG,
        800,
        600,
        1.0,
    );
    plot.write_image(
        format!("{file_name}.svg"),
        plotly::ImageFormat::SVG,
        800,
        600,
        1.0,
    );
    plot.write_image(
        format!("{file_name}.pdf"),
        plotly::ImageFormat::PDF,
        800,
        600,
        1.0,
    );
}

pub fn plot_voxel_types(types: &Array3<VoxelType>, file_name: &str, title: &str) {
    let mut z: Vec<Vec<i32>> = Vec::new();
    for y in 0..types.shape()[1] {
        let mut row: Vec<i32> = Vec::new();
        for x in 0..types.shape()[0] {
            row.push(types[(x, y, 0)] as i32);
        }
        z.push(row);
    }
    let mut row: Vec<i32> = Vec::new();
    for x in 0..types.shape()[0] {
        if x < 7 {
            row.push(x as i32);
        } else {
            row.push(0)
        }
    }
    z.push(row);

    let trace = HeatMap::new_z(z).color_scale(ColorScale::Palette(
        plotly::common::ColorScalePalette::Earth,
    ));
    let mut plot = Plot::new();

    let width = (500.0 * types.shape()[0] as f32 / types.shape()[1] as f32) as usize + 175;
    let height = (500.0 * types.shape()[1] as f32 / types.shape()[0] as f32) as usize;

    let layout = Layout::new()
        .title(title.into())
        .x_axis(
            Axis::new()
                .title("x".into())
                .range(vec![0, types.shape()[0] - 1]),
        )
        .y_axis(
            Axis::new()
                .title("y".into())
                .range(vec![types.shape()[1] - 1, 0])
                .anchor("x"),
        )
        .height(height)
        .width(width);

    plot.add_trace(trace);
    plot.set_layout(layout);

    save_plot(file_name, plot, width, height, 1.0);
}

pub fn plot_activation_time(activation_times: &ArrayActivationTime, file_name: &str, title: &str) {
    let times = &activation_times.values;
    let mut z: Vec<Vec<f32>> = Vec::new();
    for y in 0..times.shape()[1] {
        let mut row: Vec<f32> = Vec::new();
        for x in 0..times.shape()[0] {
            row.push(times[(x, y, 0)].unwrap_or(-1.0));
        }
        z.push(row);
    }

    let trace =
        HeatMap::new_z(z).color_scale(ColorScale::Palette(plotly::common::ColorScalePalette::Jet));
    let mut plot = Plot::new();

    let width = (500.0 * times.shape()[0] as f32 / times.shape()[1] as f32) as usize + 175;
    let height = (500.0 * times.shape()[1] as f32 / times.shape()[0] as f32) as usize;

    let layout = Layout::new()
        .title(title.into())
        .x_axis(
            Axis::new()
                .title("x".into())
                .range(vec![0, times.shape()[0] - 1]),
        )
        .y_axis(
            Axis::new()
                .title("y".into())
                .range(vec![times.shape()[1] - 1, 0])
                .anchor("x"),
        )
        .height(height)
        .width(width);

    plot.add_trace(trace);
    plot.set_layout(layout);

    save_plot(file_name, plot, width, height, 1.0);
}

pub fn plot_matrix(matrix: &Array2<f32>, file_name: &str, title: &str) {
    let mut z: Vec<Vec<f32>> = Vec::new();
    for y in 0..matrix.shape()[1] {
        let mut row: Vec<f32> = Vec::new();
        for x in 0..matrix.shape()[0] {
            row.push(matrix[(x, y)]);
        }
        z.push(row);
    }

    let trace =
        HeatMap::new_z(z).color_scale(ColorScale::Palette(plotly::common::ColorScalePalette::RdBu));
    let mut plot = Plot::new();

    let width = (500.0 * matrix.shape()[0] as f32 / matrix.shape()[1] as f32) as usize + 175;
    let height = (500.0 * matrix.shape()[1] as f32 / matrix.shape()[0] as f32) as usize;

    let layout = Layout::new()
        .title(title.into())
        .x_axis(
            Axis::new()
                .title("Axis 1".into())
                .range(vec![-0.5, matrix.shape()[0] as f32 - 0.5]),
        )
        .y_axis(
            Axis::new()
                .title("Axis 2".into())
                .range(vec![-0.5, matrix.shape()[1] as f32 - 0.5]),
        )
        .height(height)
        .width(width);

    plot.add_trace(trace);
    plot.set_layout(layout);

    save_plot(file_name, plot, width, height, 1.0);
}

fn save_plot(file_name: &str, plot: Plot, width: usize, height: usize, scale: f64) {
    plot.write_html(format!("{file_name}.html"));
    plot.write_image(
        format!("{file_name}.png"),
        plotly::ImageFormat::PNG,
        width,
        height,
        scale,
    );
    plot.write_image(
        format!("{file_name}.svg"),
        plotly::ImageFormat::SVG,
        width,
        height,
        scale,
    );
    plot.write_image(
        format!("{file_name}.pdf"),
        plotly::ImageFormat::PDF,
        width,
        height,
        scale,
    );
}
