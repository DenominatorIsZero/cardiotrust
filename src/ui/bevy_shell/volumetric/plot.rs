use anyhow::Result;
use ndarray::{Array1, ArrayView1};

use crate::vis::plotting::png::line::line_plot;

pub(super) const PLOT_IMAGE_MARGIN_PX: u32 = 12;
pub(super) const PLOT_CURSOR_WIDTH_PX: f32 = 2.0;

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

#[tracing::instrument(level = "trace", skip_all)]
pub(super) fn build_signal_plot_image(
    signal: ArrayView1<'_, f32>,
    sample_rate_hz: f32,
    request: &PlotImageRequest,
) -> Result<PlotImage> {
    let width = request.width.max(1);
    let height = request.height.max(1);
    let x = Array1::linspace(0.0, signal.len() as f32 / sample_rate_hz, signal.len());
    let bundle = line_plot(
        Some(&x),
        vec![&signal],
        None,
        None,
        None,
        None,
        None,
        Some((width, height)),
    )?;

    let mut rgba = Vec::with_capacity((bundle.width * bundle.height * 4) as usize);
    for chunk in bundle.data.chunks_exact(3) {
        rgba.extend_from_slice(chunk);
        rgba.push(255);
    }

    Ok(PlotImage {
        rgba,
        width: bundle.width,
        height: bundle.height,
        chart_left_px: PLOT_IMAGE_MARGIN_PX as f32,
        chart_right_px: (bundle.width.saturating_sub(PLOT_IMAGE_MARGIN_PX)) as f32,
    })
}

#[tracing::instrument(level = "trace")]
pub(super) fn cursor_left_px(sample_index: usize, sample_count: usize, plot: &PlotImage) -> f32 {
    if sample_count <= 1 {
        return plot.chart_left_px;
    }

    let chart_width = (plot.chart_right_px - plot.chart_left_px).max(PLOT_CURSOR_WIDTH_PX);
    let x = plot.chart_left_px + sample_index as f32 / (sample_count - 1) as f32 * chart_width;
    (x - PLOT_CURSOR_WIDTH_PX * 0.5).clamp(
        0.0,
        (plot.width as f32 - PLOT_CURSOR_WIDTH_PX).max(0.0),
    )
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
