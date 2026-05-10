//! Algorithm tab content for the scenario editor.
//!
//! Spawns four collapsible sections:
//! - Algorithm Settings (type, epochs, batch size, freeze flags)
//! - Optimizer Settings (optimizer, learning rate, LR schedule)
//! - Regularization Settings (threshold, strength)
//! - Metrics Settings (snapshot interval)

#![allow(clippy::cast_precision_loss)]

use bevy::prelude::*;

use super::{
    sections::spawn_section,
    widgets::{
        spawn_checkbox, spawn_combobox, spawn_param_row, spawn_slider, ParamId, SliderValueInput,
    },
    ScenarioViewState, SectionId,
};
use crate::core::{
    algorithm::refinement::Optimizer, config::algorithm::AlgorithmType, scenario::Scenario,
};

/// Spawns all Algorithm tab sections into `parent`.
#[tracing::instrument(skip_all)]
pub fn spawn_algorithm_tab(
    commands: &mut Commands,
    parent: Entity,
    scenario: &Scenario,
    view_state: &ScenarioViewState,
) {
    let algo = &scenario.config.algorithm;

    // ── Algorithm Settings ────────────────────────────────────────────────────
    let body = spawn_section(
        commands,
        parent,
        SectionId::AlgorithmSettings,
        "Algorithm Settings",
        view_state,
    );

    // Algorithm type combo
    #[cfg(feature = "native")]
    let algo_options = vec![
        "ModelBased".to_string(),
        "ModelBasedGPU".to_string(),
        "PseudoInverse".to_string(),
    ];
    #[cfg(not(feature = "native"))]
    let algo_options = vec![
        "ModelBased".to_string(),
        "PseudoInverse".to_string(),
    ];
    let algo_idx = match algo.algorithm_type {
        AlgorithmType::ModelBased => 0,
        #[cfg(feature = "native")]
        AlgorithmType::ModelBasedGPU => 1,
        AlgorithmType::PseudoInverse => {
            #[cfg(feature = "native")]
            {
                2
            }
            #[cfg(not(feature = "native"))]
            {
                1
            }
        }
    };
    let (slot, _) = spawn_param_row_into(
        commands,
        body,
        "Algorithm Type",
        "",
        "The inverse problem algorithm to use.",
        algo_options[algo_idx].as_str(),
        ParamId::AlgorithmType,
    );
    commands.entity(slot).with_children(|ctrl| {
        spawn_combobox(ctrl, ParamId::AlgorithmType, algo_options, algo_idx);
    });

    // Epochs
    let epochs = algo.epochs as f32;
    let (slot, vc) = spawn_param_row_into(
        commands,
        body,
        "Epochs",
        "",
        "Number of training epochs.",
        &format!("{epochs:.0}"),
        ParamId::Epochs,
    );
    attach_slider_input(commands, vc, ParamId::Epochs, 1.0, 5000.0, false);
    commands.entity(slot).with_children(|ctrl| {
        spawn_slider(ctrl, ParamId::Epochs, 1.0, 5000.0, epochs, false);
    });

    // Batch Size
    let batch = algo.batch_size as f32;
    let (slot, vc) = spawn_param_row_into(
        commands,
        body,
        "Batch Size",
        "",
        "Number of samples per batch (0 = full batch).",
        &format!("{batch:.0}"),
        ParamId::BatchSize,
    );
    attach_slider_input(commands, vc, ParamId::BatchSize, 0.0, 1000.0, false);
    commands.entity(slot).with_children(|ctrl| {
        spawn_slider(ctrl, ParamId::BatchSize, 0.0, 1000.0, batch, false);
    });

    // Freeze Gains checkbox
    let (slot, _) = spawn_param_row_into(
        commands,
        body,
        "Freeze Gains",
        "",
        "When checked, the gain parameters are not updated during training.",
        if algo.freeze_gains { "on" } else { "off" },
        ParamId::FreezeGains,
    );
    commands.entity(slot).with_children(|ctrl| {
        spawn_checkbox(ctrl, ParamId::FreezeGains, algo.freeze_gains);
    });

    // Freeze Delays checkbox
    let (slot, _) = spawn_param_row_into(
        commands,
        body,
        "Freeze Delays",
        "",
        "When checked, the delay parameters are not updated during training.",
        if algo.freeze_delays { "on" } else { "off" },
        ParamId::FreezeDelays,
    );
    commands.entity(slot).with_children(|ctrl| {
        spawn_checkbox(ctrl, ParamId::FreezeDelays, algo.freeze_delays);
    });

    // ── Optimizer Settings ────────────────────────────────────────────────────
    let body = spawn_section(
        commands,
        parent,
        SectionId::OptimizerSettings,
        "Optimizer Settings",
        view_state,
    );

    // Optimizer combo
    let opt_options = vec!["SGD".to_string(), "Adam".to_string()];
    let opt_idx = match algo.optimizer {
        Optimizer::Sgd => 0,
        Optimizer::Adam => 1,
    };
    let (slot, _) = spawn_param_row_into(
        commands,
        body,
        "Optimizer",
        "",
        "Gradient descent optimizer.",
        opt_options[opt_idx].as_str(),
        ParamId::OptimizerType,
    );
    commands.entity(slot).with_children(|ctrl| {
        spawn_combobox(ctrl, ParamId::OptimizerType, opt_options, opt_idx);
    });

    // Learning Rate (log scale)
    let lr = algo.learning_rate;
    let (slot, vc) = spawn_param_row_into(
        commands,
        body,
        "Learning Rate",
        "",
        "The step size for gradient descent. Use log scale.",
        &format!("{lr:.4}"),
        ParamId::LearningRate,
    );
    attach_slider_input(commands, vc, ParamId::LearningRate, 0.001, 10000.0, true);
    commands.entity(slot).with_children(|ctrl| {
        spawn_slider(ctrl, ParamId::LearningRate, 0.001, 10000.0, lr, true);
    });

    // LR Reduction Interval
    let lr_interval = algo.learning_rate_reduction_interval as f32;
    let (slot, vc) = spawn_param_row_into(
        commands,
        body,
        "LR Reduction Interval",
        "epochs",
        "Reduce the learning rate every N epochs (0 = disabled).",
        &format!("{lr_interval:.0}"),
        ParamId::LrReductionInterval,
    );
    attach_slider_input(
        commands,
        vc,
        ParamId::LrReductionInterval,
        0.0,
        500.0,
        false,
    );
    commands.entity(slot).with_children(|ctrl| {
        spawn_slider(
            ctrl,
            ParamId::LrReductionInterval,
            0.0,
            500.0,
            lr_interval,
            false,
        );
    });

    // LR Reduction Factor
    let lr_factor = algo.learning_rate_reduction_factor;
    let (slot, vc) = spawn_param_row_into(
        commands,
        body,
        "LR Reduction Factor",
        "",
        "Multiply the learning rate by this factor at each reduction interval.",
        &format!("{lr_factor:.3}"),
        ParamId::LrReductionFactor,
    );
    attach_slider_input(commands, vc, ParamId::LrReductionFactor, 0.0, 1.0, false);
    commands.entity(slot).with_children(|ctrl| {
        spawn_slider(ctrl, ParamId::LrReductionFactor, 0.0, 1.0, lr_factor, false);
    });

    // ── Regularization Settings ───────────────────────────────────────────────
    let body = spawn_section(
        commands,
        parent,
        SectionId::RegularizationSettings,
        "Regularization Settings",
        view_state,
    );

    // Threshold
    let threshold = algo.maximum_regularization_threshold;
    let (slot, vc) = spawn_param_row_into(
        commands,
        body,
        "Reg. Threshold",
        "",
        "Maximum regularization threshold.",
        &format!("{threshold:.3}"),
        ParamId::MaxRegThreshold,
    );
    attach_slider_input(commands, vc, ParamId::MaxRegThreshold, 0.0, 5.0, false);
    commands.entity(slot).with_children(|ctrl| {
        spawn_slider(ctrl, ParamId::MaxRegThreshold, 0.0, 5.0, threshold, false);
    });

    // Strength
    let strength = algo.maximum_regularization_strength;
    let (slot, vc) = spawn_param_row_into(
        commands,
        body,
        "Reg. Strength",
        "",
        "Maximum regularization strength.",
        &format!("{strength:.3}"),
        ParamId::MaxRegStrength,
    );
    attach_slider_input(commands, vc, ParamId::MaxRegStrength, 0.0, 100.0, false);
    commands.entity(slot).with_children(|ctrl| {
        spawn_slider(ctrl, ParamId::MaxRegStrength, 0.0, 100.0, strength, false);
    });

    // ── Metrics Settings ──────────────────────────────────────────────────────
    let body = spawn_section(
        commands,
        parent,
        SectionId::MetricsSettings,
        "Metrics Settings",
        view_state,
    );

    // Snapshot Interval
    let snap = algo.snapshots_interval as f32;
    let (slot, vc) = spawn_param_row_into(
        commands,
        body,
        "Snapshot Interval",
        "epochs",
        "Save a model snapshot every N epochs (0 = disabled).",
        &format!("{snap:.0}"),
        ParamId::SnapshotInterval,
    );
    attach_slider_input(commands, vc, ParamId::SnapshotInterval, 0.0, 500.0, false);
    commands.entity(slot).with_children(|ctrl| {
        spawn_slider(ctrl, ParamId::SnapshotInterval, 0.0, 500.0, snap, false);
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
