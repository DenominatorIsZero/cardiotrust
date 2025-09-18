use std::{error::Error, fs::File, io::BufWriter, path::Path};

use gif::{Encoder, Frame, Repeat};
use ndarray_stats::QuantileExt;
use tracing::trace;

use super::GifBundle;
use crate::{
    core::{
        data::shapes::{SystemStatesSpherical, SystemStatesSphericalMax},
        model::spatial::voxels::{VoxelNumbers, VoxelPositions},
    },
    vis::plotting::{
        gif::{DEFAULT_FPS, DEFAULT_PLAYBACK_SPEED},
        png::states::states_spherical_plot,
        PlotSlice, StateSphericalPlotMode,
    },
};

#[allow(
    clippy::too_many_arguments,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)]
#[tracing::instrument(level = "trace")]
pub(crate) fn states_spherical_plot_over_time(
    states: &SystemStatesSpherical,
    states_max: &SystemStatesSphericalMax,
    voxel_positions_mm: &VoxelPositions,
    voxel_size_mm: f32,
    sample_rate_hz: f32,
    voxel_numbers: &VoxelNumbers,
    path: Option<&Path>,
    slice: Option<PlotSlice>,
    mode: Option<StateSphericalPlotMode>,
    playback_speed: Option<f32>,
    fps: Option<u32>,
) -> Result<GifBundle, Box<dyn Error>> {
    trace!("Generating spherixal state plot over time");

    let playback_speed = playback_speed.unwrap_or(DEFAULT_PLAYBACK_SPEED);
    let fps = fps.unwrap_or(DEFAULT_FPS);

    if playback_speed <= 0.0 {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "playback speed must be greater than 0",
        )));
    }

    if fps == 0 {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "fps must be greater than 0",
        )));
    }

    if sample_rate_hz <= 0.0 {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "sample rate must be greater than 0",
        )));
    }

    let sample_number = states.magnitude.shape()[0];
    let image_number = (fps as f32 / playback_speed) as usize;
    let sample_step = sample_number / image_number;

    let mut frames: Vec<Vec<u8>> = Vec::with_capacity(image_number);

    let time_indices: Vec<usize> = (0..sample_number).step_by(sample_step).collect();

    let mut width = 0;
    let mut height = 0;

    let range = match mode {
        Some(StateSphericalPlotMode::ABS) => Some((0.0, *states_max.magnitude.max_skipnan())),
        _ => None,
    };

    for time_index in time_indices {
        let frame = states_spherical_plot(
            states,
            states_max,
            voxel_positions_mm,
            voxel_size_mm,
            voxel_numbers,
            None,
            slice,
            mode,
            Some(time_index),
            range,
        )?;
        frames.push(frame.data);

        width = frame.width;
        height = frame.height;
    }

    if let Some(path) = path {
        let mut file = BufWriter::new(File::create(path)?);
        let mut encoder = Encoder::new(&mut file, width as u16, height as u16, &[])?;
        encoder.set_repeat(Repeat::Infinite)?;

        for frame in &frames {
            let mut frame = Frame::from_rgb(width as u16, height as u16, frame);
            frame.delay = (100.0 / fps as f32) as u16;
            encoder.write_frame(&frame)?;
        }
    }

    Ok(GifBundle {
        data: frames,
        width,
        height,
        fps,
    })
}

#[cfg(test)]
mod test {
    use std::path::Path;

    use super::*;
    use crate::{
        core::{config::simulation::Simulation as SimulationConfig, data::Data},
        tests::{clean_files, setup_folder},
    };

    const COMMON_PATH: &str = "tests/vis/plotting/gif/states";

    #[test]
    #[ignore = "expensive integration test"]
    #[allow(clippy::cast_precision_loss)]
    fn test_states_abs_default() -> anyhow::Result<()> {
        let path = Path::new(COMMON_PATH);
        setup_folder(path.to_path_buf());
        let files = vec![path.join("states_abs_default.gif")];
        clean_files(&files);

        let mut simulation_config = SimulationConfig::default();
        simulation_config.model.common.pathological = true;
        let data = Data::from_simulation_config(&simulation_config)
            .map_err(|e| anyhow::anyhow!("Failed to create simulation data for GIF states test: {}", e))?;

        states_spherical_plot_over_time(
            &data.simulation.system_states_spherical,
            &data.simulation.system_states_spherical_max,
            &data
                .simulation
                .model
                .spatial_description
                .voxels
                .positions_mm,
            data.simulation.model.spatial_description.voxels.size_mm,
            simulation_config.sample_rate_hz,
            &data.simulation.model.spatial_description.voxels.numbers,
            Some(files[0].as_path()),
            Some(PlotSlice::Z(0)),
            Some(StateSphericalPlotMode::ABS),
            Some(0.2),
            Some(10),
        )
        .map_err(|e| anyhow::anyhow!("Failed to generate spherical states GIF for test: {}", e))?;

        assert!(files[0].is_file());
        Ok(())
    }

    #[test]
    #[ignore = "expensive integration test"]
    #[allow(clippy::cast_precision_loss)]
    fn test_states_angle_default() -> anyhow::Result<()> {
        let path = Path::new(COMMON_PATH);
        setup_folder(path.to_path_buf());
        let files = vec![path.join("states_angle_default.gif")];
        clean_files(&files);

        let mut simulation_config = SimulationConfig::default();
        simulation_config.model.common.pathological = true;
        let data = Data::from_simulation_config(&simulation_config)
            .map_err(|e| anyhow::anyhow!("Failed to create simulation data for GIF states angle test: {}", e))?;

        states_spherical_plot_over_time(
            &data.simulation.system_states_spherical,
            &data.simulation.system_states_spherical_max,
            &data
                .simulation
                .model
                .spatial_description
                .voxels
                .positions_mm,
            data.simulation.model.spatial_description.voxels.size_mm,
            simulation_config.sample_rate_hz,
            &data.simulation.model.spatial_description.voxels.numbers,
            Some(files[0].as_path()),
            Some(PlotSlice::Z(0)),
            Some(StateSphericalPlotMode::ANGLE),
            Some(0.2),
            Some(10),
        )
        .map_err(|e| anyhow::anyhow!("Failed to generate spherical states angle GIF for test: {}", e))?;

        assert!(files[0].is_file());
        Ok(())
    }
}
