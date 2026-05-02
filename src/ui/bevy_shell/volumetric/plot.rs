use anyhow::Result;
use bevy::prelude::Color as BevyColor;
use ndarray::{Array1, ArrayView1};
use plotters::{coord::types::RangedCoordf32, prelude::*};

use crate::ui::colors;

pub(super) const PLOT_IMAGE_MARGIN_PX: u32 = 18;
pub(super) const PLOT_CURSOR_WIDTH_PX: f32 = 2.0;
const PLOT_LEFT_LABEL_AREA_PX: u32 = 64;
const PLOT_BOTTOM_LABEL_AREA_PX: u32 = 26;
const PLOT_TOP_MARGIN_PX: u32 = 12;
const PLOT_RIGHT_MARGIN_PX: u32 = 12;

#[derive(Debug, Clone, PartialEq)]
pub(super) struct PlotImageRequest {
    pub(super) width: u32,
    pub(super) height: u32,
    pub(super) beat_index: usize,
    pub(super) sensor_index: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct PlotImage {
    pub(super) rgba: Vec<u8>,
    pub(super) width: u32,
    pub(super) height: u32,
    pub(super) chart_left_px: f32,
    pub(super) chart_right_px: f32,
}

#[allow(clippy::cast_precision_loss)]
#[tracing::instrument(level = "trace", skip_all)]
pub(super) fn build_signal_plot_image(
    signal: ArrayView1<'_, f32>,
    sample_rate_hz: f32,
    request: &PlotImageRequest,
) -> Result<PlotImage> {
    let width = request.width.max(1);
    let height = request.height.max(1);
    let x = Array1::linspace(0.0, signal.len() as f32 / sample_rate_hz, signal.len());
    let mut rgb = vec![0; (width * height * 3) as usize];
    let x_min = *x.first().unwrap_or(&0.0);
    let x_max = *x.last().unwrap_or(&1.0);
    let mut y_min = signal
        .iter()
        .fold(f32::INFINITY, |acc, value| acc.min(*value));
    let mut y_max = signal
        .iter()
        .fold(f32::NEG_INFINITY, |acc, value| acc.max(*value));
    if !y_min.is_finite() || !y_max.is_finite() {
        y_min = -1.0;
        y_max = 1.0;
    }
    if (y_max - y_min).abs() < f32::EPSILON {
        y_min -= 1.0;
        y_max += 1.0;
    }

    {
        let root = BitMapBackend::with_buffer(&mut rgb, (width, height)).into_drawing_area();
        root.fill(&rgb_color(colors::BG0))?;

        let mut chart = ChartBuilder::on(&root)
            .margin_top(PLOT_TOP_MARGIN_PX)
            .margin_right(PLOT_RIGHT_MARGIN_PX)
            .margin_left(PLOT_IMAGE_MARGIN_PX)
            .margin_bottom(PLOT_IMAGE_MARGIN_PX)
            .x_label_area_size(PLOT_BOTTOM_LABEL_AREA_PX)
            .y_label_area_size(PLOT_LEFT_LABEL_AREA_PX)
            .build_cartesian_2d(x_min..x_max.max(x_min + f32::EPSILON), y_min..y_max)?;

        configure_mesh(&mut chart)?;

        chart.draw_series(LineSeries::new(
            x.iter()
                .zip(signal.iter())
                .map(|(x_value, y_value)| (*x_value, *y_value)),
            &rgb_color(colors::BLUE),
        ))?;

        root.present()?;
    }

    let mut rgba = Vec::with_capacity((width * height * 4) as usize);
    for chunk in rgb.chunks_exact(3) {
        rgba.extend_from_slice(chunk);
        rgba.push(255);
    }

    Ok(PlotImage {
        rgba,
        width,
        height,
        chart_left_px: (PLOT_IMAGE_MARGIN_PX + PLOT_LEFT_LABEL_AREA_PX) as f32,
        chart_right_px: (width.saturating_sub(PLOT_IMAGE_MARGIN_PX + PLOT_RIGHT_MARGIN_PX)) as f32,
    })
}

#[tracing::instrument(level = "trace", skip_all)]
fn configure_mesh<DB: DrawingBackend>(
    chart: &mut ChartContext<'_, DB, Cartesian2d<RangedCoordf32, RangedCoordf32>>,
) -> Result<()>
where
    <DB as DrawingBackend>::ErrorType: 'static,
{
    chart
        .configure_mesh()
        .disable_mesh()
        .bold_line_style(rgb_color(colors::BG3))
        .light_line_style(rgb_color(colors::BG1))
        .axis_style(rgb_color(colors::GREY1))
        .label_style(("Arial", 12).into_font().color(&rgb_color(colors::FG1)))
        .x_desc("Time (s)")
        .y_desc("Bz [pT]")
        .x_labels(6)
        .y_labels(4)
        .draw()?;
    Ok(())
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
#[tracing::instrument(level = "trace")]
fn rgb_color(color: BevyColor) -> RGBColor {
    let srgb = color.to_srgba();
    RGBColor(
        (srgb.red * 255.0).round() as u8,
        (srgb.green * 255.0).round() as u8,
        (srgb.blue * 255.0).round() as u8,
    )
}

#[allow(clippy::cast_precision_loss, clippy::suboptimal_flops)]
#[tracing::instrument(level = "trace")]
pub(super) fn cursor_left_px(sample_index: usize, sample_count: usize, plot: &PlotImage) -> f32 {
    if sample_count <= 1 {
        return plot.chart_left_px;
    }

    let chart_width = (plot.chart_right_px - plot.chart_left_px).max(PLOT_CURSOR_WIDTH_PX);
    let x = plot.chart_left_px + sample_index as f32 / (sample_count - 1) as f32 * chart_width;
    (x - PLOT_CURSOR_WIDTH_PX * 0.5).clamp(0.0, (plot.width as f32 - PLOT_CURSOR_WIDTH_PX).max(0.0))
}

#[tracing::instrument(level = "trace")]
pub(super) fn image_from_plot(plot: &PlotImage) -> bevy::prelude::Image {
    bevy::prelude::Image::new_fill(
        bevy::render::render_resource::Extent3d {
            width: plot.width,
            height: plot.height,
            depth_or_array_layers: 1,
        },
        bevy::render::render_resource::TextureDimension::D2,
        &plot.rgba,
        bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
        bevy::asset::RenderAssetUsages::RENDER_WORLD,
    )
}

#[cfg(test)]
mod tests {
    use ndarray::Array1;

    use super::{build_signal_plot_image, cursor_left_px, PlotImageRequest, PLOT_CURSOR_WIDTH_PX};

    #[test]
    fn builds_rgba_plot_image_for_signal() {
        let signal = Array1::from_vec(vec![0.0, 1.0, 0.0, -1.0, 0.5]);
        let request = PlotImageRequest {
            width: 320,
            height: 120,
            beat_index: 0,
            sensor_index: 0,
        };

        let plot = build_signal_plot_image(signal.view(), 1000.0, &request)
            .expect("plot image should render");

        assert_eq!(plot.width, 320);
        assert_eq!(plot.height, 120);
        assert_eq!(plot.rgba.len(), 320 * 120 * 4);
        assert!(plot.chart_right_px > plot.chart_left_px);
    }

    #[test]
    #[allow(clippy::cast_precision_loss)]
    fn cursor_position_stays_inside_plot_bounds() {
        let signal = Array1::from_vec(vec![0.0, 1.0, 0.0, -1.0, 0.5]);
        let request = PlotImageRequest {
            width: 320,
            height: 120,
            beat_index: 0,
            sensor_index: 0,
        };

        let plot = build_signal_plot_image(signal.view(), 1000.0, &request)
            .expect("plot image should render");

        let left = cursor_left_px(0, signal.len(), &plot);
        let right = cursor_left_px(signal.len() - 1, signal.len(), &plot);

        assert!(left >= 0.0);
        assert!(right <= plot.width as f32 - PLOT_CURSOR_WIDTH_PX);
        assert!(right > left);
    }
}
