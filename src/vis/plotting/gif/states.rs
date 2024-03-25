use gif::{Encoder, Frame, Repeat};
use plotters::prelude::*;
use std::fs::File;

fn test_gif() -> Result<(), Box<dyn std::error::Error>> {
    let width = 480;
    let height = 360;
    let frame_count = 100;

    // This vector will hold your frames
    let mut frames: Vec<Vec<u8>> = Vec::new();

    // Generate each frame
    for i in 0..frame_count {
        let mut buf = vec![0; (width * height * 3) as usize]; // RGB buffer for one frame
        {
            // Create a drawing area on this buffer
            let root = BitMapBackend::with_buffer(&mut buf, (width, height)).into_drawing_area();
            root.fill(&WHITE)?;
            // Drawing logic goes here, similar to the previous example
            let mut chart = ChartBuilder::on(&root)
                .caption("Animated Plot", ("sans-serif", 40))
                .margin(5)
                .x_label_area_size(30)
                .y_label_area_size(30)
                .build_cartesian_2d(0f32..10f32, 0f32..10f32)?;

            chart.configure_mesh().draw()?;

            // This is where your plotting logic goes
            // For example, we draw a moving circle
            let x = (i as f32 / frame_count as f32) * 10.0;
            chart.draw_series(PointSeries::of_element(
                vec![(x, 5.0)], // Point's position
                5,              // Diameter
                &RED,           // Color
                &|c, s, st| {
                    return EmptyElement::at(c)    // We place our drawing logic here
                    + Circle::new((0, 0), s, st.filled());
                },
            ))?;
        }
        frames.push(buf);
    }

    // Now encode these frames into a GIF
    let mut image = File::create("animated.gif")?;
    let mut encoder = Encoder::new(&mut image, width as u16, height as u16, &[])?;
    encoder.set_repeat(Repeat::Infinite)?;

    for buf in frames {
        let mut frame = Frame::from_rgb_speed(width as u16, height as u16, &buf, 10);
        frame.delay = 100; // 10 ms per frame
        encoder.write_frame(&frame)?;
    }

    Ok(())
}

#[cfg(test)]
mod test {

    use std::path::Path;
    use std::path::PathBuf;

    use ndarray::Array2;

    use super::*;
    const COMMON_PATH: &str = "tests/vis/plotting/gif/states";

    #[tracing::instrument(level = "trace")]
    fn setup() {
        if !Path::new(COMMON_PATH).exists() {
            std::fs::create_dir_all(COMMON_PATH).unwrap();
        }
    }

    #[tracing::instrument(level = "trace")]
    fn clean(files: &Vec<PathBuf>) {
        for file in files {
            if file.is_file() {
                std::fs::remove_file(file).unwrap();
            }
        }
    }

    #[test]
    #[allow(clippy::cast_precision_loss)]
    fn test_matrix_plot_high() {
        setup();
        let files = vec![Path::new(COMMON_PATH).join("test_states_default.png")];
        clean(&files);

        test_gif().unwrap();

        assert!(files[0].is_file());
    }
}
