//! Components and resources for scenario cards.

use bevy::prelude::*;

// ── Resources ─────────────────────────────────────────────────────────────────

/// Tracks which scenario card (by index) is currently in inline-edit mode, and the
/// draft text being edited.
#[derive(Resource, Debug, Default)]
pub struct CardEditMode {
    /// Index of the scenario being edited, or `None` if not editing.
    pub editing_index: Option<usize>,
    /// Draft text (mirrors the scenario `comment` while the user types).
    pub draft: String,
}

/// Tracks the last click time and card index for double-click detection.
#[derive(Resource, Debug, Default)]
pub struct LastCardClick {
    /// Time (in seconds since startup) of the last click.
    pub time: f64,
    /// Card index that was last clicked.
    pub index: Option<usize>,
}

/// Maximum interval (seconds) between two clicks to count as a double-click.
pub const DOUBLE_CLICK_SECS: f64 = 0.4;

// ── Components ────────────────────────────────────────────────────────────────

/// Marker for a scenario card root node.
#[derive(Component, Debug, Clone)]
pub struct ScenarioCard {
    /// The scenario ID this card represents.
    pub scenario_id: String,
    /// Index into `ScenarioList::entries`.
    pub index: usize,
}

/// Stable scenario-identity label attached to every card root entity.
///
/// Used by `apply_filter_and_sort` to map card entities back to their
/// scenario data without relying on child-order indices.
#[derive(Component, Debug, Clone)]
pub struct CardScenarioId(pub String);

/// Marker for the "New Scenario" action card.
#[derive(Component, Debug)]
pub struct NewScenarioActionCard;

/// Marker for the thumbnail area node inside a card.
#[derive(Component, Debug)]
pub struct CardThumbnailArea {
    pub scenario_id: String,
}

/// Marker for the primary display-name text node (shows comment or ID).
#[derive(Component, Debug)]
pub struct CardNameLabel {
    pub index: usize,
}

/// Marker for the secondary ID text node.
#[derive(Component, Debug)]
pub struct CardIdLabel {
    pub index: usize,
}

/// Marker for the inline-edit name field inside a new scenario card.
#[derive(Component, Debug)]
pub struct InlineEditName;

/// Marker for the inline-edit comment field inside a new scenario card.
#[derive(Component, Debug)]
pub struct InlineEditComment;

/// Quick-action button on a card (delete, copy, schedule).
/// Carries the scenario index so no parent-entity walk is needed.
#[derive(Component, Debug, Clone)]
pub struct CardQuickAction {
    pub index: usize,
    pub kind: CardQuickActionKind,
}

/// The kind of quick action.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CardQuickActionKind {
    Delete,
    Copy,
    Schedule,
}

/// Stores fuzzy-match highlight ranges for a card's name and ID labels.
/// Both fields are byte-offset ranges `(start, end)` into the relevant string.
#[derive(Component, Debug, Clone, Default)]
pub struct CardMatchHighlight {
    pub name_range: Option<(usize, usize)>,
    pub id_range: Option<(usize, usize)>,
}

/// Marker for the prefix `TextSpan` child of a `CardNameLabel` or `CardIdLabel`.
#[derive(Component, Debug)]
pub struct LabelSpanPrefix;

/// Marker for the highlight `TextSpan` child of a `CardNameLabel` or `CardIdLabel`.
#[derive(Component, Debug)]
pub struct LabelSpanHighlight;

/// Marker for the suffix `TextSpan` child of a `CardNameLabel` or `CardIdLabel`.
#[derive(Component, Debug)]
pub struct LabelSpanSuffix;
