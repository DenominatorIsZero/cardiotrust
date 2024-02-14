use bevy::prelude::*;

#[allow(clippy::module_name_repetitions)]
#[derive(Resource, Debug)]
pub struct VisOptions {
    pub playbackspeed: f32,
    pub mode: VisMode,
    pub relative_coloring: bool,
}

impl Default for VisOptions {
    fn default() -> Self {
        Self {
            playbackspeed: 0.1,
            mode: VisMode::SimulationVoxelTypes,
            relative_coloring: true,
        }
    }
}

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
