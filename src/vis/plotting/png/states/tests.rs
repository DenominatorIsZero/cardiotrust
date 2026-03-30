use std::path::Path;

use anyhow::Result;

use super::{states_plot, states_spherical_plot};
use crate::{
    core::{config::simulation::Simulation as SimulationConfig, data::Data},
    tests::{clean_files, setup_folder},
    vis::plotting::{PlotSlice, StatePlotMode, StateSphericalPlotMode},
};
const COMMON_PATH: &str = "tests/vis/plotting/png/states";

#[test]
#[allow(clippy::cast_precision_loss)]
fn test_states_plot_default() -> Result<()> {
    let path = Path::new(COMMON_PATH);
    setup_folder(path.to_path_buf())?;
    let files = vec![path.join("states_default.png")];
    clean_files(&files)?;

    let mut simulation_config = SimulationConfig::default();
    simulation_config.model.common.pathological = true;
    let data = Data::from_simulation_config(&simulation_config)?;

    states_plot(
        &data.simulation.system_states,
        &data
            .simulation
            .model
            .spatial_description
            .voxels
            .positions_mm,
        data.simulation.model.spatial_description.voxels.size_mm,
        &data.simulation.model.spatial_description.voxels.numbers,
        files[0].as_path(),
        Some(PlotSlice::Z(0)),
        Some(StatePlotMode::X),
        350,
    )?;

    assert!(files[0].is_file());
    Ok(())
}

#[test]
#[allow(clippy::cast_precision_loss)]
fn test_states_plot_x_slice() -> Result<()> {
    let path = Path::new(COMMON_PATH);
    setup_folder(path.to_path_buf())?;
    let files = vec![path.join("states_x_slice.png")];
    clean_files(&files)?;

    let mut simulation_config = SimulationConfig::default();
    simulation_config.model.common.pathological = true;
    let data = Data::from_simulation_config(&simulation_config)?;

    states_plot(
        &data.simulation.system_states,
        &data
            .simulation
            .model
            .spatial_description
            .voxels
            .positions_mm,
        data.simulation.model.spatial_description.voxels.size_mm,
        &data.simulation.model.spatial_description.voxels.numbers,
        files[0].as_path(),
        Some(PlotSlice::X(10)),
        Some(StatePlotMode::X),
        350,
    )?;

    assert!(files[0].is_file());
    Ok(())
}

#[test]
#[allow(clippy::cast_precision_loss)]
fn test_states_plot_y_slice() -> Result<()> {
    let path = Path::new(COMMON_PATH);
    setup_folder(path.to_path_buf())?;
    let files = vec![path.join("states_y_slice.png")];
    clean_files(&files)?;

    let mut simulation_config = SimulationConfig::default();
    simulation_config.model.common.pathological = true;
    let data = Data::from_simulation_config(&simulation_config)?;
    states_plot(
        &data.simulation.system_states,
        &data
            .simulation
            .model
            .spatial_description
            .voxels
            .positions_mm,
        data.simulation.model.spatial_description.voxels.size_mm,
        &data.simulation.model.spatial_description.voxels.numbers,
        files[0].as_path(),
        Some(PlotSlice::Y(5)),
        Some(StatePlotMode::X),
        350,
    )?;

    assert!(files[0].is_file());
    Ok(())
}

#[test]
#[allow(clippy::cast_precision_loss)]
fn test_states_plot_in_y() -> Result<()> {
    let path = Path::new(COMMON_PATH);
    setup_folder(path.to_path_buf())?;
    let files = vec![path.join("states_in_y.png")];
    clean_files(&files)?;

    let mut simulation_config = SimulationConfig::default();
    simulation_config.model.common.pathological = true;
    let data = Data::from_simulation_config(&simulation_config)?;

    states_plot(
        &data.simulation.system_states,
        &data
            .simulation
            .model
            .spatial_description
            .voxels
            .positions_mm,
        data.simulation.model.spatial_description.voxels.size_mm,
        &data.simulation.model.spatial_description.voxels.numbers,
        files[0].as_path(),
        Some(PlotSlice::Z(0)),
        Some(StatePlotMode::Y),
        350,
    )?;

    assert!(files[0].is_file());
    Ok(())
}

#[test]
#[allow(clippy::cast_precision_loss)]
fn test_states_plot_in_z() -> Result<()> {
    let path = Path::new(COMMON_PATH);
    setup_folder(path.to_path_buf())?;
    let files = vec![path.join("states_in_z.png")];
    clean_files(&files)?;

    let mut simulation_config = SimulationConfig::default();
    simulation_config.model.common.pathological = true;
    let data = Data::from_simulation_config(&simulation_config)?;

    states_plot(
        &data.simulation.system_states,
        &data
            .simulation
            .model
            .spatial_description
            .voxels
            .positions_mm,
        data.simulation.model.spatial_description.voxels.size_mm,
        &data.simulation.model.spatial_description.voxels.numbers,
        files[0].as_path(),
        Some(PlotSlice::Z(0)),
        Some(StatePlotMode::Z),
        350,
    )?;

    assert!(files[0].is_file());
    Ok(())
}

#[test]
#[allow(clippy::cast_precision_loss)]
fn test_states_spherical_plot_abs_z_slice() -> Result<()> {
    let path = Path::new(COMMON_PATH);
    setup_folder(path.to_path_buf())?;
    let files = vec![path.join("states_spherical_abs_z_slice.png")];
    clean_files(&files)?;

    let mut simulation_config = SimulationConfig::default();
    simulation_config.model.common.pathological = true;
    let data = Data::from_simulation_config(&simulation_config)?;

    states_spherical_plot(
        &data.simulation.system_states_spherical,
        &data.simulation.system_states_spherical_max,
        &data
            .simulation
            .model
            .spatial_description
            .voxels
            .positions_mm,
        data.simulation.model.spatial_description.voxels.size_mm,
        &data.simulation.model.spatial_description.voxels.numbers,
        Some(files[0].as_path()),
        Some(PlotSlice::Z(0)),
        Some(StateSphericalPlotMode::ABS),
        Some(350),
        None,
    )?;

    assert!(files[0].is_file());
    Ok(())
}

#[test]
#[allow(clippy::cast_precision_loss)]
fn test_states_spherical_plot_abs_y_slice() -> Result<()> {
    let path = Path::new(COMMON_PATH);
    setup_folder(path.to_path_buf())?;
    let files = vec![path.join("states_spherical_abs_y_slice.png")];
    clean_files(&files)?;

    let mut simulation_config = SimulationConfig::default();
    simulation_config.model.common.pathological = true;
    let data = Data::from_simulation_config(&simulation_config)?;

    states_spherical_plot(
        &data.simulation.system_states_spherical,
        &data.simulation.system_states_spherical_max,
        &data
            .simulation
            .model
            .spatial_description
            .voxels
            .positions_mm,
        data.simulation.model.spatial_description.voxels.size_mm,
        &data.simulation.model.spatial_description.voxels.numbers,
        Some(files[0].as_path()),
        Some(PlotSlice::Y(5)),
        Some(StateSphericalPlotMode::ABS),
        Some(350),
        None,
    )?;

    assert!(files[0].is_file());
    Ok(())
}

#[test]
#[allow(clippy::cast_precision_loss)]
fn test_states_spherical_plot_abs_x_slice() -> Result<()> {
    let path = Path::new(COMMON_PATH);
    setup_folder(path.to_path_buf())?;
    let files = vec![path.join("states_spherical_abs_x_slice.png")];
    clean_files(&files)?;

    let mut simulation_config = SimulationConfig::default();
    simulation_config.model.common.pathological = true;
    let data = Data::from_simulation_config(&simulation_config)?;

    states_spherical_plot(
        &data.simulation.system_states_spherical,
        &data.simulation.system_states_spherical_max,
        &data
            .simulation
            .model
            .spatial_description
            .voxels
            .positions_mm,
        data.simulation.model.spatial_description.voxels.size_mm,
        &data.simulation.model.spatial_description.voxels.numbers,
        Some(files[0].as_path()),
        Some(PlotSlice::X(10)),
        Some(StateSphericalPlotMode::ABS),
        Some(350),
        None,
    )?;

    assert!(files[0].is_file());
    Ok(())
}

#[test]
#[allow(clippy::cast_precision_loss)]
fn test_states_spherical_plot_angle_z_slice() -> Result<()> {
    let path = Path::new(COMMON_PATH);
    setup_folder(path.to_path_buf())?;
    let files = vec![path.join("states_spherical_angle_z_slice.png")];
    clean_files(&files)?;

    let mut simulation_config = SimulationConfig::default();
    simulation_config.model.common.pathological = true;
    let data = Data::from_simulation_config(&simulation_config)?;

    states_spherical_plot(
        &data.simulation.system_states_spherical,
        &data.simulation.system_states_spherical_max,
        &data
            .simulation
            .model
            .spatial_description
            .voxels
            .positions_mm,
        data.simulation.model.spatial_description.voxels.size_mm,
        &data.simulation.model.spatial_description.voxels.numbers,
        Some(files[0].as_path()),
        Some(PlotSlice::Z(0)),
        Some(StateSphericalPlotMode::ANGLE),
        Some(350),
        None,
    )?;

    assert!(files[0].is_file());
    Ok(())
}

#[test]
#[allow(clippy::cast_precision_loss)]
fn test_states_spherical_plot_angle_y_slice() -> Result<()> {
    let path = Path::new(COMMON_PATH);
    setup_folder(path.to_path_buf())?;
    let files = vec![path.join("states_spherical_angle_y_slice.png")];
    clean_files(&files)?;

    let mut simulation_config = SimulationConfig::default();
    simulation_config.model.common.pathological = true;
    let data = Data::from_simulation_config(&simulation_config)?;

    states_spherical_plot(
        &data.simulation.system_states_spherical,
        &data.simulation.system_states_spherical_max,
        &data
            .simulation
            .model
            .spatial_description
            .voxels
            .positions_mm,
        data.simulation.model.spatial_description.voxels.size_mm,
        &data.simulation.model.spatial_description.voxels.numbers,
        Some(files[0].as_path()),
        Some(PlotSlice::Y(5)),
        Some(StateSphericalPlotMode::ANGLE),
        Some(350),
        None,
    )?;

    assert!(files[0].is_file());
    Ok(())
}

#[test]
#[allow(clippy::cast_precision_loss)]
fn test_states_spherical_plot_angle_x_slice() -> Result<()> {
    let path = Path::new(COMMON_PATH);
    setup_folder(path.to_path_buf())?;
    let files = vec![path.join("states_spherical_angle_x_slice.png")];
    clean_files(&files)?;

    let mut simulation_config = SimulationConfig::default();
    simulation_config.model.common.pathological = true;
    let data = Data::from_simulation_config(&simulation_config)?;

    states_spherical_plot(
        &data.simulation.system_states_spherical,
        &data.simulation.system_states_spherical_max,
        &data
            .simulation
            .model
            .spatial_description
            .voxels
            .positions_mm,
        data.simulation.model.spatial_description.voxels.size_mm,
        &data.simulation.model.spatial_description.voxels.numbers,
        Some(files[0].as_path()),
        Some(PlotSlice::X(10)),
        Some(StateSphericalPlotMode::ANGLE),
        Some(350),
        None,
    )?;

    assert!(files[0].is_file());
    Ok(())
}

#[test]
#[allow(clippy::cast_precision_loss)]
fn test_states_spherical_plot_abs_max() -> Result<()> {
    let path = Path::new(COMMON_PATH);
    setup_folder(path.to_path_buf())?;
    let files = vec![path.join("states_spherical_abs_max.png")];
    clean_files(&files)?;

    let mut simulation_config = SimulationConfig::default();
    simulation_config.model.common.pathological = true;
    let data = Data::from_simulation_config(&simulation_config)?;

    states_spherical_plot(
        &data.simulation.system_states_spherical,
        &data.simulation.system_states_spherical_max,
        &data
            .simulation
            .model
            .spatial_description
            .voxels
            .positions_mm,
        data.simulation.model.spatial_description.voxels.size_mm,
        &data.simulation.model.spatial_description.voxels.numbers,
        Some(files[0].as_path()),
        Some(PlotSlice::Z(0)),
        Some(StateSphericalPlotMode::ABS),
        None,
        None,
    )?;

    assert!(files[0].is_file());
    Ok(())
}

#[test]
#[allow(clippy::cast_precision_loss)]
fn test_states_spherical_plot_angle_max() -> Result<()> {
    let path = Path::new(COMMON_PATH);
    setup_folder(path.to_path_buf())?;
    let files = vec![path.join("states_spherical_angle_max.png")];
    clean_files(&files)?;

    let mut simulation_config = SimulationConfig::default();
    simulation_config.model.common.pathological = true;
    let data = Data::from_simulation_config(&simulation_config)?;

    states_spherical_plot(
        &data.simulation.system_states_spherical,
        &data.simulation.system_states_spherical_max,
        &data
            .simulation
            .model
            .spatial_description
            .voxels
            .positions_mm,
        data.simulation.model.spatial_description.voxels.size_mm,
        &data.simulation.model.spatial_description.voxels.numbers,
        Some(files[0].as_path()),
        None,
        Some(StateSphericalPlotMode::ANGLE),
        None,
        None,
    )?;

    assert!(files[0].is_file());
    Ok(())
}
