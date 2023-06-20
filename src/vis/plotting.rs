use ndarray::{arr1, s, Array1, Array2, Array3};
use ndarray_stats::QuantileExt;
use plotly::{
    color::NamedColor,
    common::{ColorScale, Mode},
    layout::Axis,
    HeatMap, Layout, Plot, Scatter,
};

use crate::core::{
    data::shapes::ArraySystemStates,
    model::{functional::allpass::shapes::ArrayActivationTime, spatial::voxels::VoxelType},
};

pub fn standard_time_plot(
    y: &Array1<f32>,
    sample_rate_hz: f32,
    file_name: &str,
    title: &str,
    y_label: &str,
) {
    let t = Array1::from_vec(
        (0..y.shape()[0])
            .map(|i| i as f32 / sample_rate_hz)
            .collect(),
    );
    let t_min = *t.min_skipnan();
    let t_max = *t.max_skipnan();
    let mut y_min = *y.min_skipnan();
    let mut y_max = *y.max_skipnan();
    let y_range = y_max - y_min;
    let y_margin = 0.1;
    y_min = y_min - y_margin * y_range;
    y_max = y_max + y_margin * y_range;

    let mut plot = Plot::new();
    let trace = Scatter::from_array(t, y.clone()).mode(Mode::Lines);
    plot.add_trace(trace);
    let layout = Layout::new()
        .title(title.into())
        .x_axis(
            Axis::new()
                .title("t [s]".into())
                .range(vec![t_min, t_max])
                .show_spikes(true),
        )
        .y_axis(Axis::new().title(y_label.into()).range(vec![y_min, y_max]));
    plot.set_layout(layout);

    let width = 800;
    let height = 600;
    let scale = 1.0;
    save_plot(file_name, plot, width, height, scale);
}

pub fn plot_state_xyz(
    system_states: &ArraySystemStates,
    state_index: usize,
    sample_rate_hz: f32,
    file_name: &str,
    title: &str,
) {
    let x = system_states.values.slice(s![.., state_index]).to_owned();
    let y = system_states
        .values
        .slice(s![.., state_index + 1])
        .to_owned();
    let z = system_states
        .values
        .slice(s![.., state_index + 2])
        .to_owned();
    let t = Array1::from_vec(
        (0..y.shape()[0])
            .map(|i| i as f32 / sample_rate_hz)
            .collect(),
    );

    let mut xyz_min = *arr1(&[*x.min_skipnan(), *y.min_skipnan(), *z.min_skipnan()]).min_skipnan();
    let mut xyz_max = *arr1(&[*x.max_skipnan(), *y.max_skipnan(), *z.max_skipnan()]).max_skipnan();
    let xyz_range = xyz_max - xyz_min;
    let xyz_margin = 0.1;
    xyz_min = xyz_min - xyz_margin * xyz_range;
    xyz_max = xyz_max + xyz_margin * xyz_range;

    let t_min = *t.min_skipnan();
    let t_max = *t.max_skipnan();

    let mut plot = Plot::new();
    let trace_x = Scatter::from_array(t.clone(), x)
        .mode(Mode::Lines)
        .line(
            plotly::common::Line::new()
                .color(NamedColor::SkyBlue)
                .width(2 as f64)
                .dash(plotly::common::DashType::Solid),
        )
        .name("x");
    let trace_y = Scatter::from_array(t.clone(), y)
        .mode(Mode::Lines)
        .line(
            plotly::common::Line::new()
                .color(NamedColor::Orange)
                .width(2 as f64)
                .dash(plotly::common::DashType::Dot),
        )
        .name("y");
    let trace_z = Scatter::from_array(t.clone(), z)
        .mode(Mode::Lines)
        .line(
            plotly::common::Line::new()
                .color(NamedColor::SeaGreen)
                .width(2 as f64)
                .dash(plotly::common::DashType::Dash),
        )
        .name("z");
    plot.add_trace(trace_x);
    plot.add_trace(trace_y);
    plot.add_trace(trace_z);
    let layout = Layout::new()
        .title(title.into())
        .x_axis(Axis::new().title("t [s]".into()).range(vec![t_min, t_max]))
        .y_axis(
            Axis::new()
                .title("j [A/mm^2]".into())
                .range(vec![xyz_min, xyz_max]),
        );
    plot.set_layout(layout);

    let width = 800;
    let height = 600;
    let scale = 1.0;
    save_plot(file_name, plot, width, height, scale);
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
