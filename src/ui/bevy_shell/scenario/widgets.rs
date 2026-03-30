//! Reusable UI control widgets for the Scenario editor.
//!
//! Each widget is a spawn function plus a marker component and update systems.
//! All widgets check for the [`Disabled`] component before mutating config.

#![allow(clippy::type_complexity, clippy::assigning_clones)]

pub mod checkbox;
pub mod combobox;
pub mod number_input;
pub mod param_id;
pub mod param_row;
pub mod slider;
pub mod text_input;
pub mod tooltip;
pub mod xyz;

// ── Disabled marker ───────────────────────────────────────────────────────────

use bevy::prelude::*;

/// Marker added to all interactive control nodes when the scenario is not in
/// Planning status. Interaction observers check for this and early-return.
#[derive(Component, Debug, Clone, Copy)]
pub struct Disabled;

// ── Re-exports ────────────────────────────────────────────────────────────────

pub use checkbox::{handle_checkbox_click, spawn_checkbox, CheckboxMark, CheckboxWidget};
pub use combobox::{
    handle_combobox_click, handle_combobox_option_click, spawn_combobox, update_combobox_display,
    ComboBoxDisplay, ComboBoxDropdown, ComboBoxOption, ComboBoxWidget,
};
pub use number_input::{
    handle_number_input, spawn_number_input, update_number_inputs, NumberInputDisplay,
    NumberInputWidget,
};
pub use param_id::ParamId;
pub use param_row::{spawn_param_row, TooltipTarget};
pub use slider::{
    handle_slider_drag, handle_slider_value_input, spawn_slider, update_slider_fills,
    ParamValueDisplay, SliderFill, SliderThumb, SliderValueInput, SliderWidget,
};
pub use text_input::{handle_text_input, spawn_text_input, TextInputDisplay, TextInputWidget};
pub use tooltip::{
    handle_tooltip_click, handle_tooltip_hover, hide_tooltip_on_other_click, spawn_tooltip_overlay,
    TooltipButton, TooltipOverlay, TooltipState, TooltipText,
};
pub use xyz::spawn_xyz_group;
