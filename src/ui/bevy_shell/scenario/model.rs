//! Model tab content for the scenario editor.
//!
//! Spawns five collapsible sections:
//! - Heart Geometry (voxel size, heart offset, heart size)
//! - Functional Settings (control function, pathological, current factor)
//! - Propagation Velocity (per-tissue sliders)
//! - Handcrafted Model (conditional on model type)
//! - MRI Model (conditional on model type)

#![allow(clippy::cast_precision_loss)]

use bevy::prelude::*;

use super::{
    sections::spawn_section,
    widgets::{
        spawn_checkbox, spawn_combobox, spawn_param_row, spawn_slider, spawn_text_input,
        spawn_xyz_group, ParamId, SliderValueInput,
    },
    ScenarioViewState, SectionId,
};
use crate::core::{config::model::ControlFunction, scenario::Scenario};

// ── Marker components ─────────────────────────────────────────────────────────

/// Marker for the Handcrafted section container (for conditional visibility).
#[derive(Component, Debug)]
pub struct HandcraftedSectionContainer;

/// Marker for the MRI section container (for conditional visibility).
#[derive(Component, Debug)]
pub struct MriSectionContainer;

// ── Spawn ─────────────────────────────────────────────────────────────────────

/// Spawns all Model tab sections into `parent`.
#[tracing::instrument(skip_all)]
pub fn spawn_model_tab(
    commands: &mut Commands,
    parent: Entity,
    scenario: &Scenario,
    view_state: &ScenarioViewState,
) {
    let sim_model = &scenario.config.simulation.model;
    let common = &sim_model.common;
    let is_handcrafted = sim_model.handcrafted.is_some();

    // ── Heart Geometry ────────────────────────────────────────────────────────
    let body = spawn_section(
        commands,
        parent,
        SectionId::HeartGeometry,
        "Heart Geometry",
        view_state,
    );

    // Voxel Size
    let voxel = common.voxel_size_mm;
    let (slot, vc) = spawn_param_row_into(
        commands,
        body,
        "Voxel Size",
        "mm",
        "The desired size of the voxels in mm.",
        &format!("{voxel:.1}"),
        ParamId::VoxelSize,
    );
    attach_slider_input(commands, vc, ParamId::VoxelSize, 1.0, 10.0, false);
    commands.entity(slot).with_children(|ctrl| {
        spawn_slider(ctrl, ParamId::VoxelSize, 1.0, 10.0, voxel, false);
    });

    // Heart Offset XYZ
    let offset = common.heart_offset_mm;
    let (slot, _) = spawn_param_row_into(
        commands,
        body,
        "Heart Offset",
        "mm",
        "The offset of the heart in the body coordinate system (mm).",
        &format!("({:.0},{:.0},{:.0})", offset[0], offset[1], offset[2]),
        ParamId::HeartOffsetX,
    );
    commands.entity(slot).with_children(|ctrl| {
        spawn_xyz_group(ctrl, ParamId::HeartOffsetX, offset);
    });

    // Heart Size XYZ (Handcrafted only)
    if let Some(handcrafted) = sim_model.handcrafted.as_ref() {
        let size = handcrafted.heart_size_mm;
        let (slot, _) = spawn_param_row_into(
            commands,
            body,
            "Heart Size",
            "mm",
            "The overall size of the heart in mm (Handcrafted model only).",
            &format!("({:.0},{:.0},{:.0})", size[0], size[1], size[2]),
            ParamId::HeartSizeX,
        );
        commands.entity(slot).with_children(|ctrl| {
            spawn_xyz_group(ctrl, ParamId::HeartSizeX, size);
        });
    }

    // ── Functional Settings ───────────────────────────────────────────────────
    let body = spawn_section(
        commands,
        parent,
        SectionId::FunctionalSettings,
        "Functional Settings",
        view_state,
    );

    // Control Function combo
    let cf_options = vec![
        "Ohara".to_string(),
        "Triangle".to_string(),
        "Ramp".to_string(),
    ];
    let cf_idx = match common.control_function {
        ControlFunction::Ohara => 0,
        ControlFunction::Triangle => 1,
        ControlFunction::Ramp => 2,
    };
    let (slot, _) = spawn_param_row_into(
        commands,
        body,
        "Control Function",
        "",
        "The cardiac action potential control function.",
        cf_options[cf_idx].as_str(),
        ParamId::ControlFunction,
    );
    commands.entity(slot).with_children(|ctrl| {
        spawn_combobox(ctrl, ParamId::ControlFunction, cf_options, cf_idx);
    });

    // Pathological checkbox
    let (slot, _) = spawn_param_row_into(
        commands,
        body,
        "Pathological",
        "",
        "Whether to include pathological tissue in the model.",
        if common.pathological { "on" } else { "off" },
        ParamId::Pathological,
    );
    commands.entity(slot).with_children(|ctrl| {
        spawn_checkbox(ctrl, ParamId::Pathological, common.pathological);
    });

    // Current Factor
    let cf = common.current_factor_in_pathology;
    let (slot, vc) = spawn_param_row_into(
        commands,
        body,
        "Current Factor",
        "",
        "The current factor applied in pathological tissue.",
        &format!("{cf:.3}"),
        ParamId::CurrentFactor,
    );
    attach_slider_input(commands, vc, ParamId::CurrentFactor, 0.0, 2.0, false);
    commands.entity(slot).with_children(|ctrl| {
        spawn_slider(ctrl, ParamId::CurrentFactor, 0.0, 2.0, cf, false);
    });

    // ── Propagation Velocity ──────────────────────────────────────────────────
    let body = spawn_section(
        commands,
        parent,
        SectionId::PropagationVelocity,
        "Propagation Velocity",
        view_state,
    );

    let pv = &common.propagation_velocities;
    let velocity_params = [
        (
            "SA Node",
            "m/s",
            pv.sinoatrial,
            ParamId::PropVelSA,
            "Propagation velocity in the sinoatrial node.",
        ),
        (
            "Atrium",
            "m/s",
            pv.atrium,
            ParamId::PropVelAtrium,
            "Propagation velocity in the atrium.",
        ),
        (
            "AV Node",
            "m/s",
            pv.atrioventricular,
            ParamId::PropVelAV,
            "Propagation velocity in the atrioventricular node.",
        ),
        (
            "HPS",
            "m/s",
            pv.hps,
            ParamId::PropVelHPS,
            "Propagation velocity in the His-Purkinje system.",
        ),
        (
            "Ventricle",
            "m/s",
            pv.ventricle,
            ParamId::PropVelVentricle,
            "Propagation velocity in the ventricle.",
        ),
        (
            "Pathological",
            "m/s",
            pv.pathological,
            ParamId::PropVelPathological,
            "Propagation velocity in pathological tissue.",
        ),
    ];

    for (label, unit, value, param_id, tooltip) in velocity_params {
        let (slot, vc) = spawn_param_row_into(
            commands,
            body,
            label,
            unit,
            tooltip,
            &format!("{value:.3}"),
            param_id,
        );
        attach_slider_input(commands, vc, param_id, 0.001, 10.0, true);
        commands.entity(slot).with_children(|ctrl| {
            spawn_slider(ctrl, param_id, 0.001, 10.0, value, true);
        });
    }

    // ── Handcrafted Model (conditional) ──────────────────────────────────────
    let hc_display = if is_handcrafted {
        Display::Flex
    } else {
        Display::None
    };
    let hc_container = commands
        .spawn((
            HandcraftedSectionContainer,
            Node {
                flex_direction: FlexDirection::Column,
                width: Val::Percent(100.0),
                display: hc_display,
                ..default()
            },
        ))
        .id();
    commands.entity(parent).add_child(hc_container);

    let body = spawn_section(
        commands,
        hc_container,
        SectionId::HandcraftedModel,
        "Handcrafted Model",
        view_state,
    );

    if let Some(hc) = sim_model.handcrafted.as_ref() {
        // SA center
        let (slot, _) = spawn_param_row_into(
            commands,
            body,
            "SA Node Center",
            "",
            "SA node center as percentage of heart size.",
            &format!(
                "({:.2},{:.2})",
                hc.sa_x_center_percentage, hc.sa_y_center_percentage
            ),
            ParamId::SaCenterX,
        );
        commands.entity(slot).with_children(|ctrl| {
            ctrl.spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(8.0),
                ..default()
            })
            .with_children(|row| {
                use super::widgets::spawn_number_input;
                // Both use SaCenterX so the composite display groups them together.
                spawn_number_input(row, ParamId::SaCenterX, 0, hc.sa_x_center_percentage, "X");
                spawn_number_input(row, ParamId::SaCenterX, 1, hc.sa_y_center_percentage, "Y");
            });
        });

        // AV / HPS includes
        let (slot, _) = spawn_param_row_into(
            commands,
            body,
            "Include Atrium",
            "",
            "Whether to include atrium tissue.",
            if hc.include_atrium { "on" } else { "off" },
            ParamId::IncludeAtrium,
        );
        commands.entity(slot).with_children(|ctrl| {
            spawn_checkbox(ctrl, ParamId::IncludeAtrium, hc.include_atrium);
        });

        let (slot, _) = spawn_param_row_into(
            commands,
            body,
            "Include AV",
            "",
            "Whether to include AV node tissue.",
            if hc.include_av { "on" } else { "off" },
            ParamId::IncludeAv,
        );
        commands.entity(slot).with_children(|ctrl| {
            spawn_checkbox(ctrl, ParamId::IncludeAv, hc.include_av);
        });

        let (slot, _) = spawn_param_row_into(
            commands,
            body,
            "Include HPS",
            "",
            "Whether to include His-Purkinje system.",
            if hc.include_hps { "on" } else { "off" },
            ParamId::IncludeHps,
        );
        commands.entity(slot).with_children(|ctrl| {
            spawn_checkbox(ctrl, ParamId::IncludeHps, hc.include_hps);
        });

        // Atrium Y start
        let (slot, vc) = spawn_param_row_into(
            commands,
            body,
            "Atrium Y Start",
            "%",
            "Atrium start position as a Y percentage.",
            &format!("{:.2}", hc.atrium_y_start_percentage),
            ParamId::AtriumYStart,
        );
        attach_slider_input(commands, vc, ParamId::AtriumYStart, 0.0, 1.0, false);
        commands.entity(slot).with_children(|ctrl| {
            spawn_slider(
                ctrl,
                ParamId::AtriumYStart,
                0.0,
                1.0,
                hc.atrium_y_start_percentage,
                false,
            );
        });

        // AV center X
        let (slot, vc) = spawn_param_row_into(
            commands,
            body,
            "AV Center X",
            "%",
            "AV node X center as percentage.",
            &format!("{:.2}", hc.av_x_center_percentage),
            ParamId::AvCenterX,
        );
        attach_slider_input(commands, vc, ParamId::AvCenterX, 0.0, 1.0, false);
        commands.entity(slot).with_children(|ctrl| {
            spawn_slider(
                ctrl,
                ParamId::AvCenterX,
                0.0,
                1.0,
                hc.av_x_center_percentage,
                false,
            );
        });

        // HPS parameters
        let hps_params = [
            (
                "HPS Y Stop",
                "%",
                hc.hps_y_stop_percentage,
                ParamId::HpsYStop,
            ),
            (
                "HPS X Start",
                "%",
                hc.hps_x_start_percentage,
                ParamId::HpsXStart,
            ),
            (
                "HPS X Stop",
                "%",
                hc.hps_x_stop_percentage,
                ParamId::HpsXStop,
            ),
            ("HPS Y Up", "%", hc.hps_y_up_percentage, ParamId::HpsYUp),
        ];
        for (label, unit, value, param_id) in hps_params {
            let (slot, vc) = spawn_param_row_into(
                commands,
                body,
                label,
                unit,
                &format!("HPS {label} parameter."),
                &format!("{value:.2}"),
                param_id,
            );
            attach_slider_input(commands, vc, param_id, 0.0, 1.0, false);
            commands.entity(slot).with_children(|ctrl| {
                spawn_slider(ctrl, param_id, 0.0, 1.0, value, false);
            });
        }

        // Pathology region
        let path_params = [
            (
                "Pathology X Start",
                "%",
                hc.pathology_x_start_percentage,
                ParamId::PathXStart,
            ),
            (
                "Pathology X Stop",
                "%",
                hc.pathology_x_stop_percentage,
                ParamId::PathXStop,
            ),
            (
                "Pathology Y Start",
                "%",
                hc.pathology_y_start_percentage,
                ParamId::PathYStart,
            ),
            (
                "Pathology Y Stop",
                "%",
                hc.pathology_y_stop_percentage,
                ParamId::PathYStop,
            ),
        ];
        for (label, unit, value, param_id) in path_params {
            let (slot, vc) = spawn_param_row_into(
                commands,
                body,
                label,
                unit,
                &format!("{label} boundary of pathological region."),
                &format!("{value:.2}"),
                param_id,
            );
            attach_slider_input(commands, vc, param_id, 0.0, 1.0, false);
            commands.entity(slot).with_children(|ctrl| {
                spawn_slider(ctrl, param_id, 0.0, 1.0, value, false);
            });
        }
    }

    // ── MRI Model (conditional) ───────────────────────────────────────────────
    let mri_display = if is_handcrafted {
        Display::None
    } else {
        Display::Flex
    };
    let mri_container = commands
        .spawn((
            MriSectionContainer,
            Node {
                flex_direction: FlexDirection::Column,
                width: Val::Percent(100.0),
                display: mri_display,
                ..default()
            },
        ))
        .id();
    commands.entity(parent).add_child(mri_container);

    let body = spawn_section(
        commands,
        mri_container,
        SectionId::MriModel,
        "MRI Model",
        view_state,
    );

    let mri_path = sim_model.mri.as_ref().map_or_else(
        || "assets/segmentation.nii".to_string(),
        |m| m.path.to_string_lossy().to_string(),
    );

    let (slot, _) = spawn_param_row_into(
        commands,
        body,
        "MRI Path",
        "",
        "Path to the MRI segmentation file (.nii).",
        &mri_path,
        ParamId::MriPath,
    );
    commands.entity(slot).with_children(|ctrl| {
        spawn_text_input(ctrl, ParamId::MriPath, &mri_path);
    });
}

/// Attaches a [`SliderValueInput`] component to a value-container entity.
#[tracing::instrument(skip_all)]
fn attach_slider_input(
    commands: &mut Commands,
    vc: Entity,
    param_id: ParamId,
    min: f32,
    max: f32,
    log_scale: bool,
) {
    commands.entity(vc).insert(SliderValueInput {
        param_id,
        min,
        max,
        log_scale,
        focused: false,
        input_buffer: String::new(),
    });
}

/// Convenience wrapper.
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
