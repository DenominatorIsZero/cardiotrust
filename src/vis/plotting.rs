use plotly::Plot;

pub mod engiffen;
pub mod matrix;
pub mod time;

fn save_plot(file_name: &str, plot: &Plot, width: usize, height: usize, scale: f64) {
    // plot.write_html(format!("{file_name}.html"));
    plot.write_image(
        format!("{file_name}.png"),
        plotly::ImageFormat::PNG,
        width,
        height,
        scale,
    );
    // plot.write_image(
    //     format!("{file_name}.svg"),
    //     plotly::ImageFormat::SVG,
    //     width,
    //     height,
    //     scale,
    // );
    // plot.write_image(
    //     format!("{file_name}.pdf"),
    //     plotly::ImageFormat::PDF,
    //     width,
    //     height,
    //     scale,
    // );
}
