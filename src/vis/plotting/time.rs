use ndarray::{arr1, s, Array1};
use ndarray_stats::QuantileExt;
use plotly::{color::NamedColor, common::Mode, layout::Axis, Layout, Plot, Scatter};

use crate::core::data::shapes::ArraySystemStates;

use super::save_plot;

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
