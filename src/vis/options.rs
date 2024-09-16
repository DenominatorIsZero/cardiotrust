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
pub struct ColorOptions {
    pub playbackspeed: f32,
    pub mode: ColorMode,
    pub relative_coloring: bool,
}

impl Default for ColorOptions {
    #[tracing::instrument(level = "debug")]
    fn default() -> Self {
        debug!("Initializing default visualization options.");
        Self {
            playbackspeed: 0.1,
            mode: ColorMode::SimulationVoxelTypes,
            relative_coloring: true,
        }
    }
}

/// `VisMode` is an enum representing the different visualization modes.
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ColorMode {
    EstimationVoxelTypes,
    SimulationVoxelTypes,
    EstimatedCdeNorm,
    SimulatedCdeNorm,
    EstimatedCdeMax,
    SimulatedCdeMax,
    DeltaCdeMax,
    EstimatedActivationTime,
    SimulatedActivationTime,
    DeltaActivationTime,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ColorSource {
    Estimation,
    Simulation,
    Delta,
}

#[allow(clippy::module_name_repetitions, clippy::struct_excessive_bools)]
#[derive(Resource, Debug)]
pub struct VisibilityOptions {
    pub sensors_visible: bool,
    pub sensor_bracket_visible: bool,
    pub heart_visible: bool,
    pub cutting_plane_visible: bool,
}

impl Default for VisibilityOptions {
    #[tracing::instrument(level = "debug")]
    fn default() -> Self {
        debug!("Initializing default visibility options.");
        Self {
            sensors_visible: true,
            sensor_bracket_visible: false,
            heart_visible: true,
            cutting_plane_visible: false,
        }
    }
}
