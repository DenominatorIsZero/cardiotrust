use std::time::{SystemTime, UNIX_EPOCH};

use bevy::{prelude::*, ui::UiGlobalTransform};
use bevy_egui::egui;

use super::types::{ControlValueKind, StepDirection, VisibilityTarget};
use crate::{
    core::scenario::Scenario,
    vis::{
        cutting_plane::CuttingPlaneSettings,
        options::{ColorMode, ColorOptions, VisibilityOptions},
        sample_tracker::SampleTracker,
        sensors::BacketSettings,
    },
};

#[tracing::instrument(level = "trace")]
pub(super) fn color_mode_label(mode: &ColorMode) -> &'static str {
    match mode {
        ColorMode::EstimationVoxelTypes => "Voxel types\n(estimation)",
        ColorMode::SimulationVoxelTypes => "Voxel types\n(simulation)",
        ColorMode::EstimatedCdeNorm => "Cde norm\n(estimation)",
        ColorMode::SimulatedCdeNorm => "Cde norm\n(simulation)",
        ColorMode::EstimatedCdeMax => "Cde max\n(estimation)",
        ColorMode::SimulatedCdeMax => "Cde max\n(simulation)",
        ColorMode::DeltaCdeMax => "Cde max\n(delta)",
        ColorMode::EstimatedActivationTime => "Activation time\n(estimation)",
        ColorMode::SimulatedActivationTime => "Activation time\n(simulation)",
        ColorMode::DeltaActivationTime => "Activation time\n(delta)",
    }
}

#[tracing::instrument(level = "trace")]
pub(super) fn color_mode_menu_label(mode: &ColorMode) -> &'static str {
    match mode {
        ColorMode::EstimationVoxelTypes => "Voxel types (estimation)",
        ColorMode::SimulationVoxelTypes => "Voxel types (simulation)",
        ColorMode::EstimatedCdeNorm => "Cde norm (estimation)",
        ColorMode::SimulatedCdeNorm => "Cde norm (simulation)",
        ColorMode::EstimatedCdeMax => "Cde max (estimation)",
        ColorMode::SimulatedCdeMax => "Cde max (simulation)",
        ColorMode::DeltaCdeMax => "Cde max (delta)",
        ColorMode::EstimatedActivationTime => "Activation time (estimation)",
        ColorMode::SimulatedActivationTime => "Activation time (simulation)",
        ColorMode::DeltaActivationTime => "Activation time (delta)",
    }
}

#[tracing::instrument(level = "trace")]
pub(super) fn default_screenshot_file_name() -> String {
    let timestamp_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0);

    format!("volumetric-screenshot-{timestamp_ms}.png")
}

#[tracing::instrument(level = "trace", skip_all)]
pub(super) fn control_value_text(
    kind: ControlValueKind,
    scenario: Option<&Scenario>,
    sample_tracker: &SampleTracker,
    color_options: &ColorOptions,
    visibility_options: &VisibilityOptions,
    cutting_plane: &CuttingPlaneSettings,
    sensor_bracket_settings: &BacketSettings,
) -> String {
    match kind {
        ControlValueKind::ColorMode => color_mode_label(&color_options.mode).to_string(),
        ControlValueKind::RelativeColoring => on_off(color_options.relative_coloring),
        ControlValueKind::PlaybackSpeed => format!("{:.2}x", color_options.playbackspeed),
        ControlValueKind::ManualMode => on_off(sample_tracker.manual),
        ControlValueKind::Sample => format!(
            "{}/{}",
            sample_tracker.current_sample,
            sample_tracker.max_sample.saturating_sub(1)
        ),
        ControlValueKind::Beat => {
            let beat_max = scenario
                .and_then(|scenario| scenario.results.as_ref())
                .and_then(|results| results.model.as_ref())
                .map_or(0, |model| {
                    model.spatial_description.sensors.array_offsets_mm.shape()[0].saturating_sub(1)
                });
            format!("{}/{}", sample_tracker.selected_beat, beat_max)
        }
        ControlValueKind::Sensor => {
            let sensor_max = scenario
                .and_then(|scenario| scenario.results.as_ref())
                .map_or(0, |results| {
                    results
                        .estimations
                        .measurements
                        .num_sensors()
                        .saturating_sub(1)
                });
            format!("{}/{}", sample_tracker.selected_sensor, sensor_max)
        }
        ControlValueKind::Visibility(target) => on_off(match target {
            VisibilityTarget::Heart => visibility_options.heart,
            VisibilityTarget::CuttingPlane => visibility_options.cutting_plane,
            VisibilityTarget::Sensors => visibility_options.sensors,
            VisibilityTarget::SensorBracket => visibility_options.sensor_bracket,
            VisibilityTarget::Torso => visibility_options.torso,
            VisibilityTarget::Room => visibility_options.room,
        }),
        ControlValueKind::CuttingPlaneEnabled => on_off(cutting_plane.enabled),
        ControlValueKind::CuttingPlanePosition(axis) => {
            format_vec3_axis(cutting_plane.position, axis, 1)
        }
        ControlValueKind::CuttingPlaneNormal(axis) => {
            format_vec3_axis(cutting_plane.normal, axis, 2)
        }
        ControlValueKind::CuttingPlaneOpacity => format!("{:.2}", cutting_plane.opacity),
        ControlValueKind::SensorBracketOffset(axis) => {
            format_vec3_axis(sensor_bracket_settings.offset, axis, 1)
        }
        ControlValueKind::SensorBracketRadius => format!("{:.1}", sensor_bracket_settings.radius),
    }
}

#[tracing::instrument(level = "trace")]
fn on_off(value: bool) -> String {
    if value {
        "On".to_string()
    } else {
        "Off".to_string()
    }
}

#[tracing::instrument(level = "trace")]
pub(super) fn cycle_color_mode(mode: &ColorMode, direction: StepDirection) -> ColorMode {
    let modes = all_color_modes();

    let current_index = modes
        .iter()
        .position(|candidate| candidate == mode)
        .unwrap_or(0);
    let next_index = match direction {
        StepDirection::Decrease => current_index.checked_sub(1).unwrap_or(modes.len() - 1),
        StepDirection::Increase => (current_index + 1) % modes.len(),
    };
    modes[next_index].clone()
}

#[tracing::instrument(level = "trace")]
pub(super) fn all_color_modes() -> [ColorMode; 10] {
    [
        ColorMode::EstimationVoxelTypes,
        ColorMode::SimulationVoxelTypes,
        ColorMode::EstimatedCdeNorm,
        ColorMode::SimulatedCdeNorm,
        ColorMode::EstimatedCdeMax,
        ColorMode::SimulatedCdeMax,
        ColorMode::DeltaCdeMax,
        ColorMode::EstimatedActivationTime,
        ColorMode::SimulatedActivationTime,
        ColorMode::DeltaActivationTime,
    ]
}

#[tracing::instrument(level = "trace")]
pub(super) fn step_playback_speed(current: f32, direction: StepDirection) -> f32 {
    let speeds = [0.01, 0.02, 0.05, 0.1, 0.2, 0.5, 1.0];
    let index = speeds
        .iter()
        .position(|speed| (*speed - current).abs() < f32::EPSILON)
        .unwrap_or_else(|| {
            speeds
                .iter()
                .enumerate()
                .min_by(|(_, left), (_, right)| {
                    ((*left - current).abs())
                        .partial_cmp(&((*right - current).abs()))
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .map_or(0, |(index, _)| index)
        });

    let next_index = match direction {
        StepDirection::Decrease => index.saturating_sub(1),
        StepDirection::Increase => (index + 1).min(speeds.len() - 1),
    };
    speeds[next_index]
}

#[tracing::instrument(level = "trace")]
pub(super) fn step_usize(current: usize, direction: StepDirection, max: usize) -> usize {
    match direction {
        StepDirection::Decrease => current.saturating_sub(1),
        StepDirection::Increase => (current + 1).min(max),
    }
}

#[tracing::instrument(level = "trace")]
pub(super) fn step_f32(
    current: f32,
    direction: StepDirection,
    amount: f32,
    min: f32,
    max: f32,
) -> f32 {
    match direction {
        StepDirection::Decrease => (current - amount).max(min),
        StepDirection::Increase => (current + amount).min(max),
    }
}

#[tracing::instrument(level = "trace", skip_all)]
pub(super) fn step_vec3_axis(
    vector: &mut Vec3,
    axis: usize,
    direction: StepDirection,
    amount: f32,
) {
    let component = match axis {
        0 => &mut vector.x,
        1 => &mut vector.y,
        _ => &mut vector.z,
    };
    *component = step_f32(*component, direction, amount, -10_000.0, 10_000.0);
}

#[tracing::instrument(level = "trace")]
pub(super) fn format_vec3_axis(vector: Vec3, axis: usize, precision: usize) -> String {
    let value = match axis {
        0 => vector.x,
        1 => vector.y,
        _ => vector.z,
    };
    format!("{value:.precision$}")
}

#[tracing::instrument(level = "trace", skip_all)]
pub(super) fn logical_rect_from_node(
    transform: &UiGlobalTransform,
    computed: &ComputedNode,
    scale: f32,
) -> egui::Rect {
    let size = computed.size() / scale;
    let center = transform.affine().translation / scale;
    let min = center - size * 0.5;
    egui::Rect::from_min_size(egui::pos2(min.x, min.y), egui::vec2(size.x, size.y))
}
