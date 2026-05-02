use std::sync::{mpsc, Mutex};

use bevy::prelude::*;

use crate::vis::options::ColorMode;

pub(super) const DEFAULT_PLOT_HEIGHT: f32 = 250.0;
pub(super) const COLLAPSED_PLOT_HEIGHT: f32 = 30.0;
pub(super) const MIN_PLOT_HEIGHT: f32 = 160.0;
pub(super) const MAX_PLOT_HEIGHT_RATIO: f32 = 0.55;
pub(super) const CONTROL_PANEL_BREAKPOINT: f32 = 1000.0;
pub(super) const DEFAULT_PANEL_WIDTH: f32 = 280.0;
pub(super) const DEFAULT_FULLSCREEN_CAMERA_TRANSLATION: Vec3 =
    Vec3::new(-300.962, -859.492, 653.266);
pub(super) const DEFAULT_CAMERA_ROTATION: Quat =
    Quat::from_xyzw(0.450_315, -0.047_057, -0.253_808, 0.854_742);
pub(super) const PLOT_HANDLE_HEIGHT: f32 = 18.0;
pub(super) const PANEL_SECTION_GAP: f32 = 12.0;
pub(super) const PANEL_ROW_GAP: f32 = 10.0;
pub(super) const PANEL_CARD_RADIUS: f32 = 10.0;
pub(super) const PANEL_CARD_BG: Color = Color::srgba(0.157, 0.157, 0.157, 0.72);
pub(super) const TAB_STRIP_WIDTH: f32 = 172.0;
pub(super) const TAB_BUTTON_HEIGHT: f32 = 42.0;
pub(super) const TAB_STRIP_HALF_HEIGHT: f32 = 105.0;
pub(super) const PANEL_RIGHT_OFFSET: f32 = TAB_STRIP_WIDTH + 24.0;
pub(super) const SMALL_ACTION_BUTTON_SIZE: f32 = 28.0;
use super::plot::{PlotImage, PlotImageRequest};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) enum VolumetricSection {
    VoxelColoring,
    Visibility,
    CuttingPlane,
    SensorBracket,
}

impl VolumetricSection {
    #[tracing::instrument(level = "trace")]
    pub(super) fn title(self) -> &'static str {
        match self {
            Self::VoxelColoring => "Voxel Coloring",
            Self::Visibility => "Visibility",
            Self::CuttingPlane => "Cutting Plane",
            Self::SensorBracket => "Sensor Bracket",
        }
    }

    #[tracing::instrument(level = "trace")]
    pub(super) fn sidebar_icon(self) -> &'static str {
        match self {
            Self::VoxelColoring => "[C]",
            Self::Visibility => "[V]",
            Self::CuttingPlane => "[P]",
            Self::SensorBracket => "[B]",
        }
    }
}

#[derive(Resource, Debug)]
pub(super) struct VolumetricViewState {
    pub(super) active_section: Option<VolumetricSection>,
    pub(super) fullscreen: bool,
    pub(super) plot_height: f32,
    pub(super) plot_collapsed: bool,
    pub(super) last_expanded_plot_height: f32,
    pub(super) color_mode_open: bool,
}

impl Default for VolumetricViewState {
    #[tracing::instrument(level = "trace")]
    fn default() -> Self {
        Self {
            active_section: None,
            fullscreen: false,
            plot_height: DEFAULT_PLOT_HEIGHT,
            plot_collapsed: false,
            last_expanded_plot_height: DEFAULT_PLOT_HEIGHT,
            color_mode_open: false,
        }
    }
}

#[derive(Resource, Default)]
pub(super) struct ScreenshotDialogReceiver(
    pub(super) Option<Mutex<mpsc::Receiver<Option<std::path::PathBuf>>>>,
);

#[derive(Resource, Default, Debug, Clone)]
pub(super) struct PlotImageState {
    pub(super) request: Option<PlotImageRequest>,
    pub(super) image: Option<PlotImage>,
    pub(super) handle: Option<Handle<Image>>,
}

#[derive(Component, Debug)]
pub(super) struct VolumetricViewRoot;

#[derive(Component, Debug)]
pub(super) struct ViewportHost;

#[derive(Component, Debug)]
pub(super) struct OverlayTabStrip;

#[derive(Component, Debug)]
pub(super) struct OverlayPanelHost;

#[derive(Component, Debug)]
pub(super) struct OverlayPanelTitle;

#[derive(Component, Debug, Clone, Copy)]
pub(super) struct SectionPanel {
    pub(super) section: VolumetricSection,
}

#[derive(Component, Debug)]
pub(super) struct PlotContainer;

#[derive(Component, Debug)]
pub(super) struct PlotCanvas;

#[derive(Component, Debug, Clone, Copy)]
pub(super) struct PlotImageNode;

#[derive(Component, Debug)]
pub(super) struct PlotCursor;

#[derive(Component, Debug)]
pub(super) struct PlotEmptyLabel;

#[derive(Component, Debug)]
pub(super) struct PlotResizeHandle;

#[derive(Component, Debug)]
pub(super) struct PlotCollapseButton;

#[derive(Component, Debug)]
pub(super) struct PlotCollapseLabel;

#[derive(Component, Debug)]
pub(super) struct PlotStatusLabel;

#[derive(Component, Debug)]
pub(super) struct ToolbarContainer;

#[derive(Component, Debug)]
pub(super) struct ToolbarResetCameraButton;

#[derive(Component, Debug)]
pub(super) struct ToolbarFullscreenButton;

#[derive(Component, Debug)]
pub(super) struct ToolbarFullscreenLabel;

#[derive(Component, Debug)]
pub(super) struct ToolbarScreenshotButton;

#[derive(Component, Debug)]
pub(super) struct ToolbarScreenshotLabel;

#[derive(Component, Debug, Clone, Copy)]
pub(super) struct SectionTabButton {
    pub(super) section: VolumetricSection,
}

#[derive(Component, Debug)]
pub(super) struct SectionTabLabel {
    pub(super) section: VolumetricSection,
}

#[derive(Component, Debug)]
pub(super) struct BlocksCameraMotion;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum StepDirection {
    Decrease,
    Increase,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum VisibilityTarget {
    Heart,
    CuttingPlane,
    Sensors,
    SensorBracket,
    Torso,
    Room,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ControlValueKind {
    ColorMode,
    RelativeColoring,
    PlaybackSpeed,
    ManualMode,
    Sample,
    Beat,
    Sensor,
    Visibility(VisibilityTarget),
    CuttingPlaneEnabled,
    CuttingPlanePosition(usize),
    CuttingPlaneNormal(usize),
    CuttingPlaneOpacity,
    SensorBracketOffset(usize),
    SensorBracketRadius,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ControlAction {
    CycleColorMode(StepDirection),
    ToggleRelativeColoring,
    StepPlaybackSpeed(StepDirection),
    ToggleManualMode,
    StepSample(StepDirection),
    StepBeat(StepDirection),
    StepSensor(StepDirection),
    ToggleVisibility(VisibilityTarget),
    ToggleCuttingPlaneEnabled,
    StepCuttingPlanePosition {
        axis: usize,
        direction: StepDirection,
    },
    StepCuttingPlaneNormal {
        axis: usize,
        direction: StepDirection,
    },
    StepCuttingPlaneOpacity(StepDirection),
    StepSensorBracketOffset {
        axis: usize,
        direction: StepDirection,
    },
    StepSensorBracketRadius(StepDirection),
}

#[derive(Component, Debug, Clone, Copy)]
pub(super) struct ControlValueText {
    pub(super) kind: ControlValueKind,
}

#[derive(Component, Debug, Clone, Copy)]
pub(super) struct ControlActionButton {
    pub(super) action: ControlAction,
}

#[derive(Component, Debug)]
pub(super) struct ColorModeButton;

#[derive(Component, Debug)]
pub(super) struct ColorModeButtonLabel;

#[derive(Component, Debug)]
pub(super) struct ColorModeChevron;

#[derive(Component, Debug)]
pub(super) struct ColorModeDropdown;

#[derive(Component, Debug, Clone)]
pub(super) struct ColorModeOptionButton {
    pub(super) mode: ColorMode,
}
