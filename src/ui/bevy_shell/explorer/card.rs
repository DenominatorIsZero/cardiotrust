//! Scenario card widget — spawn and update systems.
//!
//! Each `ScenarioCard` entity is a Bevy UI `Node` tree displaying:
//! status badge · thumbnail area · metrics row · display-name (comment) · id · timestamp.
//!
//! # Interaction model
//!
//! - **Single click** — selects the card (sets `SelectedSenario`); does NOT navigate.
//! - **Click on already-selected card** — enters inline comment/name edit mode.
//! - **Enter / Escape** — commits or cancels the inline edit.
//! - **Double-click (navigate)** — use the sidebar or keyboard shortcut instead.

pub mod components;
pub mod interactions;
pub mod labels;
pub mod spawn;
pub mod sync;

pub use components::{
    CardEditMode, CardIdLabel, CardMatchHighlight, CardNameLabel, CardQuickAction,
    CardQuickActionKind, CardScenarioId, CardThumbnailArea, InlineEditComment, InlineEditName,
    LabelSpanHighlight, LabelSpanPrefix, LabelSpanSuffix, LastCardClick, NewScenarioActionCard,
    ScenarioCard,
};
pub use interactions::{
    create_new_scenario, handle_card_click, handle_card_inline_edit, handle_card_quick_actions,
    handle_new_scenario_card_click,
};
pub use labels::{
    update_active_card_border, update_card_hover, update_card_label_highlights, update_card_labels,
};
pub use spawn::{spawn_card, spawn_new_scenario_action_card};
pub use sync::sync_cards_to_scenarios;
