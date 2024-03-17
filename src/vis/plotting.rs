pub mod line;
pub mod matrix;

use plotters::style::{
    colors::{BLUE, CYAN, GREEN, MAGENTA, RED, YELLOW},
    RGBColor,
};

const STANDARD_RESOLUTION: (u32, u32) = (800, 600);
const X_MARGIN: f32 = 0.0;
const Y_MARGIN: f32 = 0.1;
const CAPTION_STYLE: (&str, i32) = ("Arial", 30);
const AXIS_STYLE: (&str, i32) = ("Arial", 20);

const COLORS: [RGBColor; 6] = [BLUE, GREEN, RED, CYAN, MAGENTA, YELLOW];
