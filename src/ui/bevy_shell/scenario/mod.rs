//! Scenario editor view — Bevy-native tabbed layout.
//!
//! Implements the full scenario configuration editor as a Bevy-native UI
//! component that mounts into the existing content slot infrastructure.
//!
//! # Module layout
//!
//! ```text
//! scenario/
//!   mod.rs       — ScenarioViewPlugin, state types, spawn/despawn
//!   header.rs    — Scenario header bar (ID, status, buttons, comment)
//!   tabs.rs      — Tab bar (Simulation / Algorithm / Model)
//!   sections.rs  — Collapsible section widget
//!   widgets.rs   — Slider, ComboBox, Checkbox, NumberInput, XYZ, TextInput, Tooltip
//!   simulation.rs — Simulation tab content
//!   algorithm.rs — Algorithm tab content
//!   model.rs     — Model tab content
//! ```

pub mod algorithm;
pub mod header;
pub mod model;
pub mod sections;
pub mod simulation;
pub mod tabs;
pub mod widgets;

use std::collections::HashMap;

use bevy::prelude::*;

use self::{
    algorithm::spawn_algorithm_tab,
    header::{
        handle_comment_input, handle_copy_button, handle_delete_confirm, handle_delete_dismiss,
        handle_save_button, handle_schedule_button, spawn_header_bar, update_header_bar,
    },
    model::spawn_model_tab,
    sections::handle_section_header_click,
    simulation::spawn_simulation_tab,
    tabs::{handle_tab_click, spawn_tab_bar, update_tab_visuals},
    widgets::{
        handle_checkbox_click, handle_combobox_click, handle_combobox_option_click,
        handle_number_input, handle_slider_drag, handle_slider_value_input, handle_text_input,
        handle_tooltip_click, handle_tooltip_hover, hide_tooltip_on_other_click,
        spawn_tooltip_overlay, update_combobox_display, update_number_inputs, update_slider_fills,
        TooltipState,
    },
};
use crate::{
    core::scenario::events::{
        handle_copy_scenario, handle_delete_scenario, CopyScenarioMessage, DeleteScenarioMessage,
    },
    ui::{bevy_shell::content_area::ContentSlot, UiState},
    ScenarioList, SelectedSenario,
};

// ── State types ───────────────────────────────────────────────────────────────

/// Which top-level tab is currently active in the scenario editor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScenarioTab {
    Simulation,
    Algorithm,
    Model,
}

impl Default for ScenarioTab {
    #[tracing::instrument(level = "trace")]
    fn default() -> Self {
        Self::Simulation
    }
}

/// Stable identifier for a collapsible section.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SectionId {
    // Simulation tab
    CoreSetup,
    SensorConfiguration,
    MeasurementData,
    // Algorithm tab
    AlgorithmSettings,
    OptimizerSettings,
    RegularizationSettings,
    MetricsSettings,
    // Model tab
    HeartGeometry,
    FunctionalSettings,
    PropagationVelocity,
    HandcraftedModel,
    MriModel,
}

/// Resource that persists the UI state of the scenario editor within a session.
#[derive(Resource, Debug)]
pub struct ScenarioViewState {
    /// The currently active top-level tab.
    pub active_tab: ScenarioTab,
    /// Per-section collapse state (`true` = collapsed).
    pub section_collapsed: HashMap<SectionId, bool>,
}

impl Default for ScenarioViewState {
    #[tracing::instrument(level = "trace")]
    fn default() -> Self {
        let mut section_collapsed = HashMap::new();
        // First section of each tab is expanded; rest are collapsed.
        section_collapsed.insert(SectionId::CoreSetup, false);
        section_collapsed.insert(SectionId::SensorConfiguration, true);
        section_collapsed.insert(SectionId::MeasurementData, true);
        section_collapsed.insert(SectionId::AlgorithmSettings, false);
        section_collapsed.insert(SectionId::OptimizerSettings, true);
        section_collapsed.insert(SectionId::RegularizationSettings, true);
        section_collapsed.insert(SectionId::MetricsSettings, true);
        section_collapsed.insert(SectionId::HeartGeometry, false);
        section_collapsed.insert(SectionId::FunctionalSettings, true);
        section_collapsed.insert(SectionId::PropagationVelocity, true);
        section_collapsed.insert(SectionId::HandcraftedModel, true);
        section_collapsed.insert(SectionId::MriModel, true);
        Self {
            active_tab: ScenarioTab::default(),
            section_collapsed,
        }
    }
}

// ── Marker components ─────────────────────────────────────────────────────────

/// Marker for the root node of the entire Scenario view.
#[derive(Component, Debug)]
pub struct ScenarioViewRoot;

// ── Plugin ────────────────────────────────────────────────────────────────────

/// Plugin that owns the Bevy-native Scenario editor view.
#[derive(Debug)]
pub struct ScenarioViewPlugin;

impl Plugin for ScenarioViewPlugin {
    #[tracing::instrument(level = "info", skip(app))]
    fn build(&self, app: &mut App) {
        // State
        app.init_resource::<ScenarioViewState>();
        app.init_resource::<TooltipState>();

        // Backend messages for Copy and Delete operations
        app.add_message::<CopyScenarioMessage>()
            .add_message::<DeleteScenarioMessage>()
            .add_systems(Update, (handle_copy_scenario, handle_delete_scenario));

        // Spawn / despawn
        app.add_systems(OnEnter(UiState::Scenario), spawn_scenario_view)
            .add_systems(OnExit(UiState::Scenario), despawn_scenario_view);

        // Per-frame update systems — split across two add_systems calls to stay
        // within Bevy's 20-element tuple limit for IntoSystemConfigs.
        let scenario_condition = in_state(UiState::Scenario);
        app.add_systems(
            Update,
            (
                update_header_bar,
                update_tab_visuals,
                update_slider_fills,
                update_number_inputs,
                update_combobox_display,
                handle_tab_click,
                handle_section_header_click,
                handle_slider_drag,
                handle_checkbox_click,
                handle_combobox_click,
                handle_combobox_option_click,
            )
                .run_if(scenario_condition.clone()),
        );
        app.add_systems(
            Update,
            (
                handle_number_input,
                handle_slider_value_input,
                handle_text_input,
                handle_tooltip_hover,
                handle_tooltip_click,
                hide_tooltip_on_other_click,
                handle_comment_input,
                handle_save_button,
                handle_schedule_button,
                handle_copy_button,
                handle_delete_confirm,
                handle_delete_dismiss,
                apply_disabled_state,
            )
                .run_if(scenario_condition),
        );
    }
}

// ── Spawn / despawn ───────────────────────────────────────────────────────────

/// Spawns the Scenario editor root node as a child of [`ContentSlot`].
#[tracing::instrument(skip_all)]
fn spawn_scenario_view(
    mut commands: Commands,
    content_slots: Query<Entity, With<ContentSlot>>,
    scenario_list: Res<ScenarioList>,
    selected: Res<SelectedSenario>,
    mut view_state: ResMut<ScenarioViewState>,
) {
    let Ok(slot) = content_slots.single() else {
        return;
    };

    // Reset state for fresh session
    *view_state = ScenarioViewState::default();

    let Some(index) = selected.index else {
        return;
    };
    let Some(entry) = scenario_list.entries.get(index) else {
        return;
    };
    let scenario = &entry.scenario;

    let root = commands
        .spawn((
            ScenarioViewRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                overflow: Overflow::clip(),
                ..default()
            },
            BackgroundColor(crate::ui::colors::BG0),
        ))
        .id();

    commands.entity(slot).add_child(root);

    // Spawn header bar
    spawn_header_bar(&mut commands, root, scenario);

    // Spawn tab bar
    let (tab_bar, sim_body, algo_body, model_body) =
        spawn_tab_bar(&mut commands, root, &view_state);

    // Spawn Simulation tab content
    spawn_simulation_tab(&mut commands, sim_body, scenario, &view_state);

    // Spawn Algorithm tab content
    spawn_algorithm_tab(&mut commands, algo_body, scenario, &view_state);

    // Spawn Model tab content
    spawn_model_tab(&mut commands, model_body, scenario, &view_state);

    // Spawn shared tooltip overlay (on top of everything)
    spawn_tooltip_overlay(&mut commands, root);

    let _ = tab_bar;
}

/// Despawns all [`ScenarioViewRoot`] entities.
#[tracing::instrument(skip_all)]
fn despawn_scenario_view(
    mut commands: Commands,
    roots: Query<Entity, With<ScenarioViewRoot>>,
    overlays: Query<Entity, With<widgets::TooltipOverlay>>,
    dropdowns: Query<Entity, With<widgets::ComboBoxDropdown>>,
    mut tooltip_state: ResMut<TooltipState>,
) {
    for entity in &roots {
        commands.entity(entity).despawn();
    }
    // TooltipOverlay and ComboBoxDropdown are root-level entities (no parent)
    // and must be despawned separately.
    for entity in &overlays {
        commands.entity(entity).despawn();
    }
    for entity in &dropdowns {
        commands.entity(entity).despawn();
    }
    *tooltip_state = TooltipState::default();
}

// ── Disabled state enforcement ────────────────────────────────────────────────

/// Applies or removes the [`Disabled`] marker on all interactive control nodes
/// depending on the current scenario's planning status.
///
/// When the scenario is NOT in Planning status, all sliders, checkboxes, combo
/// boxes, and number inputs are marked `Disabled` so their interaction observers
/// early-return without mutating config values. Visual muting is applied via
/// the `GREY1` colour on the node's background.
#[tracing::instrument(skip_all)]
fn apply_disabled_state(
    scenario_list: Res<ScenarioList>,
    selected: Res<SelectedSenario>,
    mut commands: Commands,
    // Collect all interactive widget entities
    sliders: Query<(Entity, Option<&widgets::Disabled>), With<widgets::SliderWidget>>,
    checkboxes: Query<(Entity, Option<&widgets::Disabled>), With<widgets::CheckboxWidget>>,
    combos: Query<(Entity, Option<&widgets::Disabled>), With<widgets::ComboBoxWidget>>,
    number_inputs: Query<(Entity, Option<&widgets::Disabled>), With<widgets::NumberInputWidget>>,
    slider_value_inputs: Query<
        (Entity, Option<&widgets::Disabled>),
        With<widgets::SliderValueInput>,
    >,
) {
    if !scenario_list.is_changed() && !selected.is_changed() {
        return;
    }

    let is_planning = selected
        .index
        .and_then(|i| scenario_list.entries.get(i))
        .is_some_and(|entry| {
            matches!(
                entry.scenario.get_status(),
                crate::core::scenario::Status::Planning
            )
        });

    let update = |commands: &mut Commands, entity: Entity, has_disabled: bool| {
        if is_planning && has_disabled {
            commands.entity(entity).remove::<widgets::Disabled>();
        } else if !is_planning && !has_disabled {
            commands.entity(entity).insert(widgets::Disabled);
        }
    };

    for (entity, disabled) in &sliders {
        update(&mut commands, entity, disabled.is_some());
    }
    for (entity, disabled) in &checkboxes {
        update(&mut commands, entity, disabled.is_some());
    }
    for (entity, disabled) in &combos {
        update(&mut commands, entity, disabled.is_some());
    }
    for (entity, disabled) in &number_inputs {
        update(&mut commands, entity, disabled.is_some());
    }
    for (entity, disabled) in &slider_value_inputs {
        update(&mut commands, entity, disabled.is_some());
    }
}
