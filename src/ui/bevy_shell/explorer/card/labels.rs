//! Systems for updating card label text spans (hover, active border, name/id, highlights).

use bevy::prelude::*;

use super::{
    components::{
        CardEditMode, CardIdLabel, CardMatchHighlight, CardNameLabel, LabelSpanHighlight,
        LabelSpanPrefix, LabelSpanSuffix, ScenarioCard,
    },
    spawn::truncate_str,
};
use crate::{ui::colors, ScenarioList, SelectedSenario};

/// Changes card background to `BG2` on hover, restoring `BG1` otherwise.
/// Skips disabled / active cards.
#[tracing::instrument(skip_all)]
pub fn update_card_hover(
    selected: Res<SelectedSenario>,
    mut cards: Query<(&ScenarioCard, &Interaction, &mut BackgroundColor)>,
) {
    for (card, interaction, mut bg) in &mut cards {
        let is_active = selected.index == Some(card.index);
        if is_active {
            continue; // active-card styling is handled by `update_active_card_border`
        }
        let target = match interaction {
            Interaction::Hovered => colors::BG2,
            _ => colors::BG1,
        };
        bg.set_if_neq(BackgroundColor(target));
    }
}

/// Applies orange 2 px border on the active-scenario card; removes it from others.
#[tracing::instrument(skip_all)]
pub fn update_active_card_border(
    selected: Res<SelectedSenario>,
    mut cards: Query<(&ScenarioCard, &mut BorderColor, &mut BackgroundColor)>,
) {
    for (card, mut border, mut bg) in &mut cards {
        if selected.index == Some(card.index) {
            border.set_if_neq(BorderColor::all(colors::ORANGE));
            bg.set_if_neq(BackgroundColor(colors::BG2));
        } else {
            border.set_if_neq(BorderColor::all(Color::NONE));
        }
    }
}

/// Updates `CardNameLabel` and `CardIdLabel` prefix spans to reflect current scenario
/// data, shows the edit-mode draft when applicable, and gives a rename hint.
///
/// Highlight splitting is handled separately by `update_card_label_highlights`.
#[allow(clippy::type_complexity)]
#[tracing::instrument(skip_all)]
pub fn update_card_labels(
    edit_mode: Res<CardEditMode>,
    selected: Res<SelectedSenario>,
    scenario_list: Res<ScenarioList>,
    search: Res<super::super::toolbar::SearchQuery>,
    name_labels: Query<(&CardNameLabel, &Children), Without<CardIdLabel>>,
    id_labels: Query<(&CardIdLabel, &Children), Without<CardNameLabel>>,
    mut prefix_spans: Query<&mut TextSpan, With<LabelSpanPrefix>>,
    mut prefix_colors: Query<&mut TextColor, With<LabelSpanPrefix>>,
    mut highlight_spans: Query<&mut TextSpan, (With<LabelSpanHighlight>, Without<LabelSpanPrefix>)>,
    mut suffix_spans: Query<
        &mut TextSpan,
        (
            With<LabelSpanSuffix>,
            Without<LabelSpanPrefix>,
            Without<LabelSpanHighlight>,
        ),
    >,
) {
    if !edit_mode.is_changed()
        && !scenario_list.is_changed()
        && !selected.is_changed()
        && !search.is_changed()
    {
        return;
    }

    for (label, children) in &name_labels {
        let Some(entry) = scenario_list.entries.get(label.index) else {
            continue;
        };
        let is_editing = edit_mode.editing_index == Some(label.index);
        let is_selected = selected.index == Some(label.index);

        let (display, color) = if is_editing {
            (format!("{}|", edit_mode.draft), colors::ORANGE)
        } else if is_selected {
            let comment = &entry.scenario.comment;
            let id = entry.scenario.get_id();
            let base = if comment.is_empty() {
                truncate_str(id, 28)
            } else {
                truncate_str(comment, 28)
            };
            (format!("{base}  [rename]"), colors::FG0)
        } else {
            let comment = &entry.scenario.comment;
            let id = entry.scenario.get_id();
            let text = if comment.is_empty() {
                truncate_str(id, 36)
            } else {
                truncate_str(comment, 36)
            };
            (text, colors::FG0)
        };

        // Write to prefix span and clear highlight/suffix so update_card_label_highlights
        // can take over when a query is active.
        for &child in children {
            if let Ok(mut span) = prefix_spans.get_mut(child) {
                span.0.clone_from(&display);
            }
            if let Ok(mut col) = prefix_colors.get_mut(child) {
                col.0 = color;
            }
            if let Ok(mut span) = highlight_spans.get_mut(child) {
                span.0.clear();
            }
            if let Ok(mut span) = suffix_spans.get_mut(child) {
                span.0.clear();
            }
        }
    }

    for (label, children) in &id_labels {
        let Some(entry) = scenario_list.entries.get(label.index) else {
            continue;
        };
        let text = truncate_str(entry.scenario.get_id(), 32);

        for &child in children {
            if let Ok(mut span) = prefix_spans.get_mut(child) {
                span.0.clone_from(&text);
            }
            if let Ok(mut span) = highlight_spans.get_mut(child) {
                span.0.clear();
            }
            if let Ok(mut span) = suffix_spans.get_mut(child) {
                span.0.clear();
            }
        }
    }
}

/// Applies fuzzy-match highlight coloring to card label spans.
///
/// Runs after `update_card_labels`. When `CardMatchHighlight` has a range,
/// splits the prefix text at byte offsets and distributes across spans.
#[allow(clippy::type_complexity)]
#[tracing::instrument(skip_all)]
pub fn update_card_label_highlights(
    search: Res<super::super::toolbar::SearchQuery>,
    name_labels: Query<(&CardNameLabel, &CardMatchHighlight, &Children), Without<CardIdLabel>>,
    id_labels: Query<(&CardIdLabel, &CardMatchHighlight, &Children), Without<CardNameLabel>>,
    scenario_list: Res<ScenarioList>,
    edit_mode: Res<CardEditMode>,
    selected: Res<SelectedSenario>,
    mut prefix_spans: Query<&mut TextSpan, With<LabelSpanPrefix>>,
    mut highlight_spans: Query<&mut TextSpan, (With<LabelSpanHighlight>, Without<LabelSpanPrefix>)>,
    mut suffix_spans: Query<
        &mut TextSpan,
        (
            With<LabelSpanSuffix>,
            Without<LabelSpanPrefix>,
            Without<LabelSpanHighlight>,
        ),
    >,
) {
    if search.0.is_empty() {
        // No query — spans already written correctly by update_card_labels
        return;
    }

    for (label, highlight, children) in &name_labels {
        // Don't interfere with inline-edit display
        if edit_mode.editing_index == Some(label.index) {
            continue;
        }

        let Some(entry) = scenario_list.entries.get(label.index) else {
            continue;
        };

        let comment = &entry.scenario.comment;
        let id = entry.scenario.get_id();
        let is_selected = selected.index == Some(label.index);
        let max_chars = if is_selected { 28 } else { 36 };
        let full_text = if comment.is_empty() {
            truncate_str(id, max_chars)
        } else {
            truncate_str(comment, max_chars)
        };

        apply_highlight_to_spans(
            &full_text,
            highlight.name_range,
            children,
            &mut prefix_spans,
            &mut highlight_spans,
            &mut suffix_spans,
        );
    }

    for (label, highlight, children) in &id_labels {
        let Some(entry) = scenario_list.entries.get(label.index) else {
            continue;
        };
        let full_text = truncate_str(entry.scenario.get_id(), 32);

        apply_highlight_to_spans(
            &full_text,
            highlight.id_range,
            children,
            &mut prefix_spans,
            &mut highlight_spans,
            &mut suffix_spans,
        );
    }
}

/// Splits `text` at `range` and writes prefix/highlight/suffix spans accordingly.
#[allow(clippy::type_complexity)]
#[tracing::instrument(level = "trace", skip_all)]
fn apply_highlight_to_spans(
    text: &str,
    range: Option<(usize, usize)>,
    children: &Children,
    prefix_spans: &mut Query<&mut TextSpan, With<LabelSpanPrefix>>,
    highlight_spans: &mut Query<
        &mut TextSpan,
        (With<LabelSpanHighlight>, Without<LabelSpanPrefix>),
    >,
    suffix_spans: &mut Query<
        &mut TextSpan,
        (
            With<LabelSpanSuffix>,
            Without<LabelSpanPrefix>,
            Without<LabelSpanHighlight>,
        ),
    >,
) {
    let Some((start, end)) = range else {
        // No match — full text in prefix, clear others (already done by update_card_labels)
        return;
    };

    // Clamp to valid byte boundaries within text
    let start = start.min(text.len());
    let end = end.min(text.len());

    let prefix_text = &text[..start];
    let highlight_text = &text[start..end];
    let suffix_text = &text[end..];

    for &child in children {
        if let Ok(mut span) = prefix_spans.get_mut(child) {
            span.0 = prefix_text.to_string();
        }
        if let Ok(mut span) = highlight_spans.get_mut(child) {
            span.0 = highlight_text.to_string();
        }
        if let Ok(mut span) = suffix_spans.get_mut(child) {
            span.0 = suffix_text.to_string();
        }
    }
}
