//! Commits widget values back to the Scenario data model.
//!
//! UI widgets (checkboxes, sliders, combos, etc.) update their own component
//! state on interaction but never write back to the [`Scenario`] struct. This
//! module fills that gap: it queries changed widgets each frame and applies
//! their values to the in-memory Scenario so that Save serialises the correct
//! configuration.

use bevy::prelude::*;

use super::{
    widgets::{
        checkbox::CheckboxWidget, combobox::ComboBoxWidget, number_input::NumberInputWidget,
        slider::SliderWidget, text_input::TextInputWidget, ParamId,
    },
    ScenarioTab, ScenarioViewState,
};
use crate::{
    core::{
        algorithm::refinement::Optimizer,
        config::{
            algorithm::AlgorithmType,
            model::{ControlFunction, Model, Mri, SensorArrayGeometry, SensorArrayMotion},
        },
    },
    Scenario, ScenarioList, SelectedSenario,
};

/// Runs every frame to push widget values into the Scenario.
#[tracing::instrument(skip_all)]
pub fn commit_widget_changes(
    checkboxes: Query<&CheckboxWidget, Changed<CheckboxWidget>>,
    sliders: Query<&SliderWidget, Changed<SliderWidget>>,
    combos: Query<&ComboBoxWidget, Changed<ComboBoxWidget>>,
    number_inputs: Query<&NumberInputWidget, Changed<NumberInputWidget>>,
    text_inputs: Query<&TextInputWidget, Changed<TextInputWidget>>,
    mut scenario_list: ResMut<ScenarioList>,
    selected: Res<SelectedSenario>,
    view_state: Res<ScenarioViewState>,
) {
    let Some(index) = selected.index else {
        return;
    };
    let Some(entry) = scenario_list.entries.get_mut(index) else {
        return;
    };
    let scenario = &mut entry.scenario;

    for cb in &checkboxes {
        apply_checkbox(scenario, cb.param_id, cb.checked, &view_state);
    }
    for slider in &sliders {
        apply_slider(scenario, slider.param_id, slider.current, &view_state);
    }
    for combo in &combos {
        apply_combobox(scenario, combo.param_id, combo.selected_index, &view_state);
    }
    for input in &number_inputs {
        apply_number_input(
            scenario,
            input.param_id,
            input.axis_index,
            input.value,
            &view_state,
        );
    }
    for input in &text_inputs {
        apply_text_input(scenario, input.param_id, &input.content, &view_state);
    }
}

// ── Route by ParamId ──────────────────────────────────────────────────────────

fn apply_checkbox(scenario: &mut Scenario, id: ParamId, checked: bool, state: &ScenarioViewState) {
    let model = model_mut(scenario, id, state);
    match id {
        ParamId::Pathological => model.common.pathological = checked,
        ParamId::ThreeDSensors => model.common.three_d_sensors = checked,
        ParamId::FreezeGains => scenario.config.algorithm.freeze_gains = checked,
        ParamId::FreezeDelays => scenario.config.algorithm.freeze_delays = checked,
        ParamId::IncludeAtrium => {
            if let Some(hc) = model.handcrafted.as_mut() {
                hc.include_atrium = checked;
            }
        }
        ParamId::IncludeAv => {
            if let Some(hc) = model.handcrafted.as_mut() {
                hc.include_av = checked;
            }
        }
        ParamId::IncludeHps => {
            if let Some(hc) = model.handcrafted.as_mut() {
                hc.include_hps = checked;
            }
        }
        _ => {}
    }
}

fn apply_slider(scenario: &mut Scenario, id: ParamId, value: f32, state: &ScenarioViewState) {
    match id {
        // Simulation
        ParamId::SampleRate => scenario.config.simulation.sample_rate_hz = value,
        ParamId::Duration => scenario.config.simulation.duration_s = value,
        // Algorithm
        ParamId::Epochs => scenario.config.algorithm.epochs = value as usize,
        ParamId::BatchSize => scenario.config.algorithm.batch_size = value as usize,
        ParamId::LearningRate => scenario.config.algorithm.learning_rate = value,
        ParamId::LrReductionInterval => {
            scenario.config.algorithm.learning_rate_reduction_interval = value as usize;
        }
        ParamId::LrReductionFactor => {
            scenario.config.algorithm.learning_rate_reduction_factor = value;
        }
        ParamId::MaxRegThreshold => {
            scenario.config.algorithm.maximum_regularization_threshold = value;
        }
        ParamId::MaxRegStrength => {
            scenario.config.algorithm.maximum_regularization_strength = value;
        }
        ParamId::SnapshotInterval => {
            scenario.config.algorithm.snapshots_interval = value as usize;
        }
        // Model — shared between Ground Truth and Initial Model
        ParamId::VoxelSize => model_mut(scenario, id, state).common.voxel_size_mm = value,
        ParamId::CurrentFactor => {
            model_mut(scenario, id, state).common.current_factor_in_pathology = value;
        }
        // Propagation velocities
        ParamId::PropVelSA => {
            model_mut(scenario, id, state)
                .common
                .propagation_velocities
                .sinoatrial = value;
        }
        ParamId::PropVelAtrium => {
            model_mut(scenario, id, state)
                .common
                .propagation_velocities
                .atrium = value;
        }
        ParamId::PropVelAV => {
            model_mut(scenario, id, state)
                .common
                .propagation_velocities
                .atrioventricular = value;
        }
        ParamId::PropVelHPS => {
            model_mut(scenario, id, state)
                .common
                .propagation_velocities
                .hps = value;
        }
        ParamId::PropVelVentricle => {
            model_mut(scenario, id, state)
                .common
                .propagation_velocities
                .ventricle = value;
        }
        ParamId::PropVelPathological => {
            model_mut(scenario, id, state)
                .common
                .propagation_velocities
                .pathological = value;
        }
        // Handcrafted sliders
        ParamId::AtriumYStart => {
            if let Some(hc) = model_mut(scenario, id, state).handcrafted.as_mut() {
                hc.atrium_y_start_percentage = value;
            }
        }
        ParamId::AvCenterX => {
            if let Some(hc) = model_mut(scenario, id, state).handcrafted.as_mut() {
                hc.av_x_center_percentage = value;
            }
        }
        ParamId::HpsYStop => {
            if let Some(hc) = model_mut(scenario, id, state).handcrafted.as_mut() {
                hc.hps_y_stop_percentage = value;
            }
        }
        ParamId::HpsXStart => {
            if let Some(hc) = model_mut(scenario, id, state).handcrafted.as_mut() {
                hc.hps_x_start_percentage = value;
            }
        }
        ParamId::HpsXStop => {
            if let Some(hc) = model_mut(scenario, id, state).handcrafted.as_mut() {
                hc.hps_x_stop_percentage = value;
            }
        }
        ParamId::HpsYUp => {
            if let Some(hc) = model_mut(scenario, id, state).handcrafted.as_mut() {
                hc.hps_y_up_percentage = value;
            }
        }
        ParamId::PathXStart => {
            if let Some(hc) = model_mut(scenario, id, state).handcrafted.as_mut() {
                hc.pathology_x_start_percentage = value;
            }
        }
        ParamId::PathXStop => {
            if let Some(hc) = model_mut(scenario, id, state).handcrafted.as_mut() {
                hc.pathology_x_stop_percentage = value;
            }
        }
        ParamId::PathYStart => {
            if let Some(hc) = model_mut(scenario, id, state).handcrafted.as_mut() {
                hc.pathology_y_start_percentage = value;
            }
        }
        ParamId::PathYStop => {
            if let Some(hc) = model_mut(scenario, id, state).handcrafted.as_mut() {
                hc.pathology_y_stop_percentage = value;
            }
        }
        // Covariance
        ParamId::CovarianceMean => {
            scenario
                .config
                .simulation
                .model
                .common
                .measurement_covariance_mean = value;
        }
        ParamId::CovarianceStd => {
            scenario
                .config
                .simulation
                .model
                .common
                .measurement_covariance_std = value;
        }
        _ => {}
    }
}

fn apply_combobox(
    scenario: &mut Scenario,
    id: ParamId,
    index: usize,
    state: &ScenarioViewState,
) {
    match id {
        ParamId::ControlFunction => {
            let val = match index {
                0 => ControlFunction::Ohara,
                1 => ControlFunction::Triangle,
                2 => ControlFunction::Ramp,
                _ => return,
            };
            model_mut(scenario, id, state).common.control_function = val;
        }
        ParamId::SensorGeometry => {
            let val = match index {
                0 => SensorArrayGeometry::Cube,
                1 => SensorArrayGeometry::SparseCube,
                2 => SensorArrayGeometry::Cylinder,
                _ => return,
            };
            scenario.config.simulation.model.common.sensor_array_geometry = val;
        }
        ParamId::SensorMotion => {
            let val = match index {
                0 => SensorArrayMotion::Static,
                1 => SensorArrayMotion::Grid,
                _ => return,
            };
            scenario.config.simulation.model.common.sensor_array_motion = val;
        }
        ParamId::AlgorithmType => {
            let val = match index {
                0 => AlgorithmType::ModelBased,
                #[cfg(feature = "native")]
                1 => AlgorithmType::ModelBasedGPU,
                #[cfg(not(feature = "native"))]
                1 => AlgorithmType::PseudoInverse,
                #[cfg(feature = "native")]
                2 => AlgorithmType::PseudoInverse,
                _ => return,
            };
            scenario.config.algorithm.algorithm_type = val;
        }
        ParamId::OptimizerType => {
            let val = match index {
                0 => Optimizer::Sgd,
                1 => Optimizer::Adam,
                _ => return,
            };
            scenario.config.algorithm.optimizer = val;
        }
        _ => {}
    }
}

fn apply_number_input(
    scenario: &mut Scenario,
    id: ParamId,
    axis: usize,
    value: f32,
    state: &ScenarioViewState,
) {
    let idx = axis.min(2);
    match id {
        // Sensor configuration
        ParamId::SensorRadius => {
            scenario.config.simulation.model.common.sensor_array_radius_mm = value;
        }
        ParamId::NumberOfSensors => {
            scenario.config.simulation.model.common.number_of_sensors = value as usize;
        }
        // XYZ groups → array fields
        ParamId::SensorsPerAxis => {
            scenario.config.simulation.model.common.sensors_per_axis[idx] = value as usize;
        }
        ParamId::ArrayOriginX => {
            scenario.config.simulation.model.common.sensor_array_origin_mm[idx] = value;
        }
        ParamId::SensorArraySizeX => {
            scenario.config.simulation.model.common.sensor_array_size_mm[idx] = value;
        }
        ParamId::MotionRangeX => {
            scenario
                .config
                .simulation
                .model
                .common
                .sensor_array_motion_range_mm[idx] = value;
        }
        ParamId::MotionStepsX => {
            scenario
                .config
                .simulation
                .model
                .common
                .sensor_array_motion_steps[idx] = value as usize;
        }
        // Model XYZ groups
        ParamId::HeartOffsetX => {
            model_mut(scenario, id, state).common.heart_offset_mm[idx] = value;
        }
        ParamId::HeartSizeX => {
            if let Some(hc) = model_mut(scenario, id, state).handcrafted.as_mut() {
                hc.heart_size_mm[idx] = value;
            }
        }
        // SA center is a 2-axis group
        ParamId::SaCenterX => {
            let model = model_mut(scenario, id, state);
            if let Some(hc) = model.handcrafted.as_mut() {
                match axis {
                    0 => hc.sa_x_center_percentage = value,
                    1 => hc.sa_y_center_percentage = value,
                    _ => {}
                }
            }
        }
        _ => {}
    }
}

fn apply_text_input(scenario: &mut Scenario, id: ParamId, content: &str, _state: &ScenarioViewState) {
    match id {
        ParamId::MriPath => {
            scenario.config.simulation.model.mri = Some(Mri {
                path: content.into(),
            });
        }
        _ => {}
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Returns a mutable reference to the `Model` targeted by the given param.
///
/// Ground Truth tab → `config.simulation.model`
/// Initial Model tab → `config.algorithm.model`
/// Simulation / Algorithm tabs → `config.simulation.model` (fallback; model
/// params shouldn't appear there, but we pick a safe default).
fn model_mut<'a>(scenario: &'a mut Scenario, _id: ParamId, state: &ScenarioViewState) -> &'a mut Model {
    match state.active_tab {
        ScenarioTab::GroundTruth => &mut scenario.config.simulation.model,
        ScenarioTab::InitialModel => &mut scenario.config.algorithm.model,
        // Simulation and Algorithm tabs don't have model widgets — fallback
        ScenarioTab::Simulation | ScenarioTab::Algorithm => {
            &mut scenario.config.simulation.model
        }
    }
}
