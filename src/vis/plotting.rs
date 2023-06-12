use ndarray::{Array1, Array3, ArrayBase, Dim, ViewRepr};
use plotly::{
    common::{ColorScale, ColorScaleElement, ColorScalePalette, Mode, Title},
    contour::Contours,
    layout::Axis,
    Contour, HeatMap, Layout, Plot, Scatter,
};

use crate::core::model::spatial::voxels::VoxelType;

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
    let x_min = *x
        .iter()
        .reduce(|min, e| if e < min { e } else { min })
        .unwrap_or(&f32::MAX);
    let x_max = *x
        .iter()
        .reduce(|max, e| if e > max { e } else { max })
        .unwrap_or(&f32::MIN);
    let mut y_min = *y
        .iter()
        .reduce(|min, e| if e < min { e } else { min })
        .unwrap_or(&f32::MAX);
    let mut y_max = *y
        .iter()
        .reduce(|max, e| if e > max { e } else { max })
        .unwrap_or(&f32::MIN);
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
