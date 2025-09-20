use std::{fs::File, io::BufWriter, path::Path};

use gif::{Encoder, Frame, Repeat};
use ndarray::Axis;
use tracing::trace;

use super::GifBundle;
use crate::{
    core::model::spatial::voxels::{VoxelPositions, VoxelTypes},
    vis::plotting::{gif::_DEFAULT_TIME_PER_FRAME_MS, png::voxel_type::voxel_type_plot, PlotSlice},
};

#[allow(
    clippy::too_many_arguments,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::cast_lossless
)]
#[tracing::instrument(level = "trace", skip_all)]
pub(crate) fn voxel_types_over_slices_plot(
    types: &VoxelTypes,
    voxel_positions_mm: &VoxelPositions,
    voxel_size_mm: f32,
    axis: Option<Axis>,
    path: Option<&Path>,
    time_per_frame_ms: Option<u32>,
) -> anyhow::Result<GifBundle>
where
{
    trace!("Generating voxel_types over slices plot.");

    let time_per_frame_ms = time_per_frame_ms.unwrap_or(_DEFAULT_TIME_PER_FRAME_MS);

    let axis = axis.unwrap_or(Axis(2));

    if time_per_frame_ms < 1 {
        return Err(anyhow::anyhow!("Time per frame must be positive"));
    }

    if axis.index() > 2 {
        return Err(anyhow::anyhow!("Axis must be 0, 1 or 2"));
    }

    let num_slices = types.shape()[axis.index()];

    let mut frames: Vec<Vec<u8>> = Vec::with_capacity(num_slices);

    let mut width = 0;
    let mut height = 0;

    for slice in 0..num_slices {
        let slice = match axis {
            Axis(0) => Some(PlotSlice::X(slice)),
            Axis(1) => Some(PlotSlice::Y(slice)),
            Axis(2) => Some(PlotSlice::Z(slice)),
            _ => unreachable!(),
        };
        let frame = voxel_type_plot(types, voxel_positions_mm, voxel_size_mm, None, slice)?;
        frames.push(frame.data);

        width = frame.width;
        height = frame.height;
    }

    let time_per_frame = time_per_frame_ms as u16 / 10;

    if let Some(path) = path {
        let mut file = BufWriter::new(File::create(path)?);
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
