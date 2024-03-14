use bevy::prelude::*;

/// Options for visualization behaviour.
///
/// `playbackspeed` is the speed of the animation.
///
/// `mode` determines what data is visualized.
///
/// `relative_coloring` determines whether the coloring is relative to the
/// maximum value in the data.
#[allow(clippy::module_name_repetitions)]
#[derive(Resource, Debug)]
pub struct VisOptions {
    pub playbackspeed: f32,
    pub mode: VisMode,
    pub relative_coloring: bool,
}

impl Default for VisOptions {
    #[tracing::instrument(level = "debug")]
    fn default() -> Self {
        debug!("Initializing default visualization options.");
        Self {
            playbackspeed: 0.1,
            mode: VisMode::SimulationVoxelTypes,
            relative_coloring: true,
        }
    }
}

/// `VisMode` is an enum representing the different visualization modes.
///
/// `EstimationVoxelTypes` visualizes the voxel types from the estimation.
/// `SimulationVoxelTypes` visualizes the voxel types from the simulation.
/// `EstimatedCdeNorm` visualizes the estimated CDE over time
/// `SimulatedCdeNorm` visualizes the simulated CDE over time
/// `EstimatedCdeMax` visualizes the estimated CDE maximum values.
/// `SimulatedCdeMax` visualizes the simulated CDE maximum values.
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum VisMode {
    EstimationVoxelTypes,
    SimulationVoxelTypes,
    EstimatedCdeNorm,
    SimulatedCdeNorm,
    EstimatedCdeMax,
    SimulatedCdeMax,
}
