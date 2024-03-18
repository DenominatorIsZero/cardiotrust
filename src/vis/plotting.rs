pub mod line;
pub mod matrix;
pub mod matrix_old;

use plotters::style::{
    colors::{BLUE, CYAN, GREEN, MAGENTA, RED, YELLOW},
    RGBColor,
};
use tracing::trace;

const STANDARD_RESOLUTION: (u32, u32) = (800, 600);
const X_MARGIN: f32 = 0.0;
const Y_MARGIN: f32 = 0.1;
const CAPTION_STYLE: (&str, i32) = ("Arial", 30);
const AXIS_STYLE: (&str, i32) = ("Arial", 20);

const COLORBAR_WIDTH: u32 = 100;
const COLORBAR_TOP_MARGIN: u32 = 60;
const COLORBAR_BOTTOM_MARGIN: u32 = 75;
const COLORBAR_COLOR_NUMBERS: u32 = 100;

const LABEL_AREA_WIDTH: u32 = 40;
const LABEL_AREA_RIGHT_MARGIN: u32 = 10;

const UNIT_AREA_TOP_MARGIN: u32 = 25;

const CHART_MARGIN: u32 = 25;

const AXIS_LABEL_AREA: u32 = 75;
const AXIS_LABEL_NUM_MAX: usize = 10;

const LEGEND_PATH_LENGTH: i32 = 20;
const LEGEND_OPACITY: f64 = 0.8;

const COLORS: [RGBColor; 6] = [BLUE, GREEN, RED, CYAN, MAGENTA, YELLOW];

/// Allocates a buffer for storing pixel data for an image of the given width and height.
///
/// The buffer is allocated as a `Vec<u8>` with 3 bytes per pixel (for RGB color). The size of the
/// buffer is calculated from the width and height.
///
/// This function is used to allocate image buffers before rendering to them for plotting.
#[tracing::instrument(level = "trace")]
fn allocate_buffer(width: u32, height: u32) -> Vec<u8> {
    trace!("Allocating buffer.");
    let buffer: Vec<u8> = vec![0; width as usize * height as usize * 3];
    buffer
}
