//! Simulation tab content for the scenario editor.
//!
//! Spawns three collapsible sections:
//! - Core Setup (Sample Rate, Duration)
//! - Sensor Configuration (Geometry, Motion, 3D, Origin, Sensors-per-axis, etc.)
//! - Measurement Data (Covariance Mean, Std)

#![allow(clippy::cast_precision_loss)]

use bevy::prelude::*;

use super::{
    sections::spawn_section,
    widgets::{
        spawn_checkbox, spawn_combobox, spawn_number_input, spawn_param_row, spawn_slider,
        spawn_xyz_group, ParamId, SliderValueInput,
    },
    ScenarioViewState, SectionId,
};
use crate::core::{
    config::model::{SensorArrayGeometry, SensorArrayMotion},
    scenario::Scenario,
};

/// Spawns all Simulation tab sections into `parent`.
#[tracing::instrument(skip_all)]
pub fn spawn_simulation_tab(
    commands: &mut Commands,
    parent: Entity,
    scenario: &Scenario,
    view_state: &ScenarioViewState,
) {
    let sim = &scenario.config.simulation;
    let common = &sim.model.common;

    // ── Core Setup ────────────────────────────────────────────────────────────
    let body = spawn_section(
        commands,
        parent,
        SectionId::CoreSetup,
        "Core Setup",
        view_state,
    );

    // Sample Rate — range 100–10 000 Hz
    const SR_MIN: f32 = 100.0;
    const SR_MAX: f32 = 10_000.0;
    let (slot, vc) = spawn_param_row_into(
        commands,
        body,
        "Sample Rate",
        "Hz",
        "The sample rate of the simulation in Hz. Default: 2000.0 Hz.",
        &format!("{:.0}", sim.sample_rate_hz),
        ParamId::SampleRate,
    );
    commands.entity(vc).insert(SliderValueInput {
        param_id: ParamId::SampleRate,
        min: SR_MIN,
        max: SR_MAX,
        log_scale: false,
        focused: false,
        input_buffer: String::new(),
    });
    commands.entity(slot).with_children(|ctrl| {
        spawn_slider(
            ctrl,
            ParamId::SampleRate,
            SR_MIN,
            SR_MAX,
            sim.sample_rate_hz,
            false,
        );
    });

    // Duration — range 0.1–10 s
    const DUR_MIN: f32 = 0.1;
    const DUR_MAX: f32 = 10.0;
    let (slot, vc) = spawn_param_row_into(
        commands,
        body,
        "Duration",
        "s",
        "The duration of the simulation in seconds. Default: 1.0 s.",
        &format!("{:.1}", sim.duration_s),
        ParamId::Duration,
    );
    commands.entity(vc).insert(SliderValueInput {
        param_id: ParamId::Duration,
        min: DUR_MIN,
        max: DUR_MAX,
        log_scale: false,
        focused: false,
        input_buffer: String::new(),
    });
    commands.entity(slot).with_children(|ctrl| {
        spawn_slider(
            ctrl,
            ParamId::Duration,
            DUR_MIN,
            DUR_MAX,
            sim.duration_s,
            false,
        );
    });

    // ── Sensor Configuration ──────────────────────────────────────────────────
    let body = spawn_section(
        commands,
        parent,
        SectionId::SensorConfiguration,
        "Sensor Configuration",
        view_state,
    );

    // Sensor Geometry combo
    let geom_options = vec![
        "Cube".to_string(),
        "Sparse Cube".to_string(),
        "Cylinder".to_string(),
    ];
    let geom_idx = match common.sensor_array_geometry {
        SensorArrayGeometry::Cube => 0,
        SensorArrayGeometry::SparseCube => 1,
        SensorArrayGeometry::Cylinder => 2,
    };
    let (slot, _) = spawn_param_row_into(
        commands,
        body,
        "Sensor Geometry",
        "",
        "The spatial geometry of the sensor array.",
        geom_options[geom_idx].as_str(),
        ParamId::SensorGeometry,
    );
    commands.entity(slot).with_children(|ctrl| {
        spawn_combobox(ctrl, ParamId::SensorGeometry, geom_options, geom_idx);
    });

    // Sensor Motion combo
    let motion_options = vec!["Static".to_string(), "Grid".to_string()];
    let motion_idx = match common.sensor_array_motion {
        SensorArrayMotion::Static => 0,
        SensorArrayMotion::Grid => 1,
    };
    let (slot, _) = spawn_param_row_into(
        commands,
        body,
        "Sensor Motion",
        "",
        "Whether the sensor array is static or moving along a grid.",
        motion_options[motion_idx].as_str(),
        ParamId::SensorMotion,
    );
    commands.entity(slot).with_children(|ctrl| {
        spawn_combobox(ctrl, ParamId::SensorMotion, motion_options, motion_idx);
    });

    // 3D Sensors checkbox
    let (slot, _) = spawn_param_row_into(
        commands,
        body,
        "3D Sensors",
        "",
        "Whether to use 3D sensors or not. Default: true.",
        if common.three_d_sensors { "on" } else { "off" },
        ParamId::ThreeDSensors,
    );
    commands.entity(slot).with_children(|ctrl| {
        spawn_checkbox(ctrl, ParamId::ThreeDSensors, common.three_d_sensors);
    });

    // Array Origin XYZ
    let origin = common.sensor_array_origin_mm;
    let (slot, _) = spawn_param_row_into(
        commands,
        body,
        "Array Origin",
        "mm",
        "The origin of the sensor array in body coordinate system (mm).",
        &format!("({:.0},{:.0},{:.0})", origin[0], origin[1], origin[2]),
        ParamId::ArrayOriginX,
    );
    commands.entity(slot).with_children(|ctrl| {
        spawn_xyz_group(ctrl, ParamId::ArrayOriginX, origin);
    });

    // Sensors per axis (Cube/SparseCube only — conditional visibility)
    let spa = common.sensors_per_axis;
    let (slot, _) = spawn_param_row_into(
        commands,
        body,
        "Sensors per Axis",
        "",
        "The number of sensors per axis (Cube geometry only).",
        &format!("({},{},{})", spa[0], spa[1], spa[2]),
        ParamId::SensorsPerAxis,
    );
    // Tag it for conditional visibility
    commands.entity(slot).insert(SensorsPerAxisControl);
    commands.entity(slot).with_children(|ctrl| {
        ctrl.spawn(Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(8.0),
            ..default()
        })
        .with_children(|row| {
            spawn_number_input(row, ParamId::SensorsPerAxis, 0, spa[0] as f32, "X");
            spawn_number_input(row, ParamId::SensorsPerAxis, 1, spa[1] as f32, "Y");
            spawn_number_input(row, ParamId::SensorsPerAxis, 2, spa[2] as f32, "Z");
        });
    });

    // Size / Radius / Count inputs (geometry-dependent)
    match common.sensor_array_geometry {
        SensorArrayGeometry::Cube | SensorArrayGeometry::SparseCube => {
            let size = common.sensor_array_size_mm;
            let (slot, _) = spawn_param_row_into(
                commands,
                body,
                "Array Size",
                "mm",
                "The overall size of the sensor array in mm.",
                &format!("({:.0},{:.0},{:.0})", size[0], size[1], size[2]),
                ParamId::SensorArraySizeX,
            );
            commands.entity(slot).with_children(|ctrl| {
                spawn_xyz_group(ctrl, ParamId::SensorArraySizeX, size);
            });
        }
        SensorArrayGeometry::Cylinder => {
            let radius = common.sensor_array_radius_mm;
            let (slot, _) = spawn_param_row_into(
                commands,
                body,
                "Array Radius",
                "mm",
                "The radius of the cylindrical sensor array.",
                &format!("{radius:.0}"),
                ParamId::SensorRadius,
            );
            commands.entity(slot).with_children(|ctrl| {
                spawn_number_input(ctrl, ParamId::SensorRadius, 0, radius, "");
            });
        }
    }

    // Number of sensors (SparseCube and Cylinder only)
    if matches!(
        common.sensor_array_geometry,
        SensorArrayGeometry::SparseCube | SensorArrayGeometry::Cylinder
    ) {
        let n = common.number_of_sensors as f32;
        let (slot, _) = spawn_param_row_into(
            commands,
            body,
            "Number of Sensors",
            "",
            "The number of sensors used.",
            &format!("{n:.0}"),
            ParamId::NumberOfSensors,
        );
        commands.entity(slot).with_children(|ctrl| {
            spawn_number_input(ctrl, ParamId::NumberOfSensors, 0, n, "");
        });
    }

    // Motion Range / Steps (Grid motion only)
    if matches!(common.sensor_array_motion, SensorArrayMotion::Grid) {
        let range = common.sensor_array_motion_range_mm;
        let (slot, _) = spawn_param_row_into(
            commands,
            body,
            "Motion Range",
            "mm",
            "The maximum offset of the grid along each axis.",
            &format!("({:.0},{:.0},{:.0})", range[0], range[1], range[2]),
            ParamId::MotionRangeX,
        );
        commands.entity(slot).with_children(|ctrl| {
            spawn_xyz_group(ctrl, ParamId::MotionRangeX, range);
        });

        let steps = common.sensor_array_motion_steps;
        let steps_f = [steps[0] as f32, steps[1] as f32, steps[2] as f32];
        let (slot, _) = spawn_param_row_into(
            commands,
            body,
            "Motion Steps",
            "",
            "The number of grid steps along each axis.",
            &format!("({},{},{})", steps[0], steps[1], steps[2]),
            ParamId::MotionStepsX,
        );
        commands.entity(slot).with_children(|ctrl| {
            spawn_xyz_group(ctrl, ParamId::MotionStepsX, steps_f);
        });
    }

    // ── Measurement Data ──────────────────────────────────────────────────────
    let body = spawn_section(
        commands,
        parent,
        SectionId::MeasurementData,
        "Measurement Data",
        view_state,
    );

    // Covariance Mean (log scale)
    let cov_mean = common.measurement_covariance_mean;
    let (slot, vc) = spawn_param_row_into(
        commands,
        body,
        "Covariance Mean",
        "",
        "The mean of the measurement noise covariance matrix.",
        &format!("{cov_mean:.2e}"),
        ParamId::CovarianceMean,
    );
    commands.entity(vc).insert(SliderValueInput {
        param_id: ParamId::CovarianceMean,
        min: 1e-6,
        max: 1.0,
        log_scale: true,
        focused: false,
        input_buffer: String::new(),
    });
    commands.entity(slot).with_children(|ctrl| {
        spawn_slider(ctrl, ParamId::CovarianceMean, 1e-6, 1.0, cov_mean, true);
    });

    // Covariance Std
    let cov_std = common.measurement_covariance_std;
    let (slot, vc) = spawn_param_row_into(
        commands,
        body,
        "Covariance Std",
        "",
        "Standard deviation of the measurement noise covariance diagonal.",
        &format!("{cov_std:.4}"),
        ParamId::CovarianceStd,
    );
    commands.entity(vc).insert(SliderValueInput {
        param_id: ParamId::CovarianceStd,
        min: 0.0,
        max: 1.0,
        log_scale: false,
        focused: false,
        input_buffer: String::new(),
    });
    commands.entity(slot).with_children(|ctrl| {
        spawn_slider(ctrl, ParamId::CovarianceStd, 0.0, 1.0, cov_std, false);
    });
}

/// Convenience wrapper: spawns a param row as a child of `parent` and returns
/// `(control_slot, value_container)`.
#[tracing::instrument(skip_all)]
fn spawn_param_row_into(
    commands: &mut Commands,
    parent: Entity,
    label: &str,
    unit: &str,
    tooltip: &str,
    value_text: &str,
    param_id: ParamId,
) -> (Entity, Entity) {
    let mut slot = Entity::PLACEHOLDER;
    let mut vc = Entity::PLACEHOLDER;
    commands.entity(parent).with_children(|p| {
        (slot, vc) = spawn_param_row(p, label, unit, tooltip, value_text, param_id);
    });
    (slot, vc)
}

/// Marker for the sensors-per-axis row (used for conditional visibility).
#[derive(Component, Debug)]
pub struct SensorsPerAxisControl;
