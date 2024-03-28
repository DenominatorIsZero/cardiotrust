use gif::{Encoder, Frame, Repeat};

use ndarray::{ArrayBase, Axis, Ix3};
use ndarray_stats::QuantileExt;

use std::fs::File;

use std::{error::Error, path::Path};
use tracing::trace;

use crate::vis::plotting::gif::DEFAULT_TIME_PER_FRAME_MS;
use crate::vis::plotting::png::matrix::matrix_plot;

use super::GifBundle;

#[allow(
    clippy::too_many_arguments,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::cast_lossless
)]
#[tracing::instrument(level = "trace", skip(data))]
pub(crate) fn matrix_over_slices_plot<A>(
    data: &ArrayBase<A, Ix3>,
    axis: Option<Axis>,
    range: Option<(f32, f32)>,
    step: Option<(f32, f32)>,
    offset: Option<(f32, f32)>,
    path: Option<&Path>,
    title: Option<&str>,
    y_label: Option<&str>,
    x_label: Option<&str>,
    unit: Option<&str>,
    resolution: Option<(u32, u32)>,
    flip_axis: Option<(bool, bool)>,
    time_per_frame_ms: Option<u32>,
) -> Result<GifBundle, Box<dyn Error>>
where
    A: ndarray::Data<Elem = f32>,
{
    trace!("Generating matrix over slices plot.");

    let time_per_frame_ms = time_per_frame_ms.unwrap_or(DEFAULT_TIME_PER_FRAME_MS);

    let axis = axis.unwrap_or(Axis(2));

    let default_title = format!("Matrix over slices. {axis:?}");
    let title = title.unwrap_or(default_title.as_str());

    if time_per_frame_ms < 1 {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "time per frame must be positive",
        )));
    }

    if axis.index() > 2 {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "axis must be 0, 1 or 2",
        )));
    }

    let num_slices = data.shape()[axis.index()];

    let mut frames: Vec<Vec<u8>> = Vec::with_capacity(num_slices);

    let mut width = 0;
    let mut height = 0;

    let range = range.map_or_else(
        || {
            let min = data.min().unwrap();
            let max = data.max().unwrap();
            (*min, *max)
        },
        |range| range,
    );

    for slice in 0..num_slices {
        let title = format!("{title}, Slice: {slice}");
        let frame = matrix_plot(
            &data.index_axis(axis, slice),
            Some(range),
            step,
            offset,
            path,
            Some(title.as_str()),
            y_label,
            x_label,
            unit,
            resolution,
            flip_axis,
        )?;
        frames.push(frame.data);

        width = frame.width;
        height = frame.height;
    }

    let time_per_frame = time_per_frame_ms as u16 / 10;

    if let Some(path) = path {
        let mut file = File::create(path)?;
        let mut encoder = Encoder::new(&mut file, width as u16, height as u16, &[])?;
        encoder.set_repeat(Repeat::Infinite)?;

        for frame in &frames {
            let mut frame = Frame::from_rgb(width as u16, height as u16, frame);
            frame.delay = time_per_frame;
            encoder.write_frame(&frame)?;
        }
    }

    Ok(GifBundle {
        data: frames,
        width,
        height,
        fps: 100 / time_per_frame as u32,
    })
}

#[cfg(test)]
mod test {

    use std::path::Path;

    use ndarray::Array3;

    use crate::tests::clean_files;
    use crate::tests::setup_folder;

    use super::*;

    const COMMON_PATH: &str = "tests/vis/plotting/gif/matrix";

    #[test]
    fn test_matrix_over_slices_plot_valid_input() {
        let data = Array3::<f32>::zeros((10, 10, 10));
        let result = matrix_over_slices_plot(
            &data, None, None, None, None, None, None, None, None, None, None, None, None,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_matrix_over_slices_plot_invalid_axis() {
        let data = Array3::<f32>::zeros((10, 10, 10));
        let result = matrix_over_slices_plot(
            &data,
            Some(Axis(4)),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_matrix_over_slices_plot_zero_time_per_frame() {
        let data = Array3::<f32>::zeros((10, 10, 10));
        let result = matrix_over_slices_plot(
            &data,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(0),
        );
        assert!(result.is_err());
    }

    #[test]
    #[allow(clippy::cast_precision_loss)]
    fn test_matrix_over_slices_plot_with_path() {
        let path = Path::new(COMMON_PATH);
        setup_folder(path.to_path_buf());
        let files = vec![path.join("test_matrix_over_slices_plot_with_path.gif")];
        clean_files(&files);

        let mut data = Array3::<f32>::zeros((10, 10, 10));
        for x in 0..10 {
            for y in 0..10 {
                for z in 0..10 {
                    data[(x, y, z)] = x as f32 + y as f32 + z as f32;
                }
            }
        }
        let result = matrix_over_slices_plot(
            &data,
            None,
            None,
            None,
            None,
            Some(&files[0]),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        );
        assert!(result.is_ok());
    }
}
