use bevy::{math::vec3, prelude::*};
use bevy_aabb_instancing::{Cuboid, CuboidMaterialId, Cuboids, VertexPullingRenderPlugin};
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use ndarray::{arr1, s, Array1, Array2};
use ndarray_stats::QuantileExt;
use scarlet::{
    color::RGBColor,
    colormap::{ColorMap, ListedColorMap},
};

use crate::{
    core::{model::spatial::voxels::VoxelType, scenario::Scenario},
    ui::UiState,
    ScenarioList, SelectedSenario,
};
#[allow(clippy::module_name_repetitions)]
#[derive(Resource, Debug)]
pub struct VisOptions {
    pub playbackspeed: f32,
    pub mode: VisMode,
}

impl Default for VisOptions {
    fn default() -> Self {
        Self {
            playbackspeed: 0.1,
            mode: VisMode::SimulationVoxelTypes,
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
