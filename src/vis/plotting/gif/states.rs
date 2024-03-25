use bevy::time;
use gif::{Encoder, Frame, Repeat};
use ndarray::{Array2, Axis};
use ndarray_stats::QuantileExt;
use plotters::prelude::*;
use std::fs::File;
use web_sys::console::time_end;

use std::{error::Error, path::Path};
use tracing::trace;

use crate::vis::plotting::gif::{DEFAULT_FPS, DEFAULT_PLAYBACK_SPEED};
use crate::vis::plotting::png::states::states_spherical_plot;
use crate::vis::plotting::StateSphericalPlotMode;
use crate::{
    core::{
        data::shapes::{ArraySystemStatesSpherical, ArraySystemStatesSphericalMax},
        model::spatial::voxels::{VoxelNumbers, VoxelPositions},
    },
    vis::plotting::{
        png::matrix::{matrix_angle_plot, matrix_plot},
        PlotSlice,
    },
};

#[allow(clippy::too_many_arguments)]
#[tracing::instrument(level = "trace")]
pub(crate) fn states_spherical_plot_over_time(
    states: &ArraySystemStatesSpherical,
    states_max: &ArraySystemStatesSphericalMax,
    voxel_positions_mm: &VoxelPositions,
    voxel_size_mm: f32,
    sample_rate_hz: f32,
    voxel_numbers: &VoxelNumbers,
    path: Option<&Path>,
    slice: Option<PlotSlice>,
    mode: Option<StateSphericalPlotMode>,
    playback_speed: Option<f32>,
    fps: Option<u32>,
) -> Result<(Vec<Vec<u8>>, (u32, u32)), Box<dyn Error>> {
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

    let sample_number = states.magnitude.shape()[0] as usize;
    let image_number = (fps as f32 / playback_speed) as usize;
    let sample_step = sample_number / image_number as usize;

    let mut frames: Vec<Vec<u8>> = Vec::with_capacity(image_number);

    let time_indices: Vec<usize> = (0..sample_number).step_by(sample_step).collect();

    let mut width = 0;
    let mut height = 0;

    let range = match mode {
        Some(StateSphericalPlotMode::ABS) => Some((0.0, *states_max.magnitude.max_skipnan())),
        Some(StateSphericalPlotMode::ANGLE) => None,
        _ => None,
    };

    for time_index in time_indices {
        let (buffer, (this_width, this_height)) = states_spherical_plot(
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
        frames.push(buffer);

        width = this_width;
        height = this_height;
    }

    if let Some(path) = path {
        let mut file = File::create(path)?;
        let mut encoder = Encoder::new(&mut file, width as u16, height as u16, &[])?;
        encoder.set_repeat(Repeat::Infinite)?;

        for frame in &frames {
            let mut frame = Frame::from_rgb(width as u16, height as u16, &frame);
            frame.delay = (100.0 / fps as f32) as u16;
            encoder.write_frame(&frame)?;
        }
    }

    Ok((frames, (width, height)))
}

#[cfg(test)]
mod test {

    use std::path::Path;
    use std::path::PathBuf;

    use crate::core::{config::simulation::Simulation as SimulationConfig, data::Data};

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
    fn test_states_abs_default() {
        setup();
        let files = vec![Path::new(COMMON_PATH).join("test_states_abs_default.gif")];
        clean(&files);

        let mut simulation_config = SimulationConfig::default();
        simulation_config.model.pathological = true;
        let data = Data::from_simulation_config(&simulation_config)
            .expect("Model parameters to be valid.");

        states_spherical_plot_over_time(
            &data.simulation.as_ref().unwrap().system_states_spherical,
            &data
                .simulation
                .as_ref()
                .unwrap()
                .system_states_spherical_max,
            &data.get_model().spatial_description.voxels.positions_mm,
            data.get_model().spatial_description.voxels.size_mm,
            simulation_config.sample_rate_hz,
            &data.get_model().spatial_description.voxels.numbers,
            Some(files[0].as_path()),
            Some(PlotSlice::Z(0)),
            Some(StateSphericalPlotMode::ABS),
            Some(0.2),
            Some(10),
        )
        .unwrap();

        assert!(files[0].is_file());
    }

    #[test]
    #[allow(clippy::cast_precision_loss)]
    fn test_states_angle_default() {
        setup();
        let files = vec![Path::new(COMMON_PATH).join("test_states_angle_default.gif")];
        clean(&files);

        let mut simulation_config = SimulationConfig::default();
        simulation_config.model.pathological = true;
        let data = Data::from_simulation_config(&simulation_config)
            .expect("Model parameters to be valid.");

        states_spherical_plot_over_time(
            &data.simulation.as_ref().unwrap().system_states_spherical,
            &data
                .simulation
                .as_ref()
                .unwrap()
                .system_states_spherical_max,
            &data.get_model().spatial_description.voxels.positions_mm,
            data.get_model().spatial_description.voxels.size_mm,
            simulation_config.sample_rate_hz,
            &data.get_model().spatial_description.voxels.numbers,
            Some(files[0].as_path()),
            Some(PlotSlice::Z(0)),
            Some(StateSphericalPlotMode::ANGLE),
            Some(0.2),
            Some(10),
        )
        .unwrap();

        assert!(files[0].is_file());
    }
}
