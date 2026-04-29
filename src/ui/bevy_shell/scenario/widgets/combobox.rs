//! `ComboBox` widget — dropdown selection control.

use bevy::{prelude::*, ui::UiGlobalTransform};

use super::{param_id::ParamId, slider::ParamValueDisplay, Disabled};
use crate::ui::colors;

// ── Components ────────────────────────────────────────────────────────────────

/// Marker on the combobox button node.
#[derive(Component, Debug, Clone)]
pub struct ComboBoxWidget {
    pub param_id: ParamId,
    pub options: Vec<String>,
    pub selected_index: usize,
    pub open: bool,
    /// Entity of the root-level dropdown overlay for this combo box.
    pub dropdown_entity: Entity,
}

/// Marker on the combobox display text inside the button.
#[derive(Component, Debug)]
pub struct ComboBoxDisplay {
    pub param_id: ParamId,
}

/// Marker on a combobox dropdown option button.
#[derive(Component, Debug, Clone)]
pub struct ComboBoxOption {
    pub param_id: ParamId,
    pub option_index: usize,
}

/// Marker for the combobox dropdown overlay container.
#[derive(Component, Debug)]
pub struct ComboBoxDropdown {
    pub param_id: ParamId,
}

// ── Spawn ─────────────────────────────────────────────────────────────────────

/// Spawns a combo box control into the given parent.
///
/// The dropdown overlay is spawned as a **root-level** entity (no parent)
/// via `commands.commands()` so it escapes all `overflow: clip()` containers
/// and always renders on top. The despawn pass must separately remove all
/// [`ComboBoxDropdown`] entities.
#[tracing::instrument(skip_all)]
pub fn spawn_combobox(
    commands: &mut ChildSpawnerCommands,
    param_id: ParamId,
    options: Vec<String>,
    selected_index: usize,
) {
    let selected_text = options.get(selected_index).cloned().unwrap_or_default();

    // Build option list rotated so the selected item is first, then the rest
    // in order. This ensures the dropdown always opens downward with the current
    // value at the top (aligned to the button).
    let rotated: Vec<(usize, &String)> = options
        .iter()
        .enumerate()
        .cycle()
        .skip(selected_index)
        .take(options.len())
        .collect();

    // Spawn dropdown as a root-level node so it escapes `overflow: clip()`.
    let dropdown_entity = commands
        .commands()
        .spawn((
            ComboBoxDropdown { param_id },
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(0.0),
                left: Val::Px(0.0),
                min_width: Val::Px(140.0),
                flex_direction: FlexDirection::Column,
                display: Display::None,
                border: UiRect::all(Val::Px(1.0)),
                border_radius: BorderRadius::all(Val::Px(4.0)),
                ..default()
            },
            ZIndex(2000),
            BackgroundColor(colors::BG1),
            BorderColor::all(colors::BG3),
        ))
        .with_children(|dropdown| {
            for (i, option) in rotated {
                dropdown
                    .spawn((
                        ComboBoxOption {
                            param_id,
                            option_index: i,
                        },
                        Button,
                        Node {
                            padding: UiRect::axes(Val::Px(8.0), Val::Px(6.0)),
                            width: Val::Percent(100.0),
                            ..default()
                        },
                        BackgroundColor(colors::BG1),
                    ))
                    .with_children(|opt| {
                        opt.spawn((
                            Text::new(option.clone()),
                            TextFont {
                                font_size: 11.0,
                                ..default()
                            },
                            TextColor(colors::FG0),
                        ));
                    });
            }
        })
        .id();

    commands
        .spawn((
            ComboBoxWidget {
                param_id,
                options,
                selected_index,
                open: false,
                dropdown_entity,
            },
            Button,
            Node {
                min_width: Val::Px(120.0),
                height: Val::Px(24.0),
                padding: UiRect::axes(Val::Px(8.0), Val::Px(2.0)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                border: UiRect::all(Val::Px(1.0)),
                border_radius: BorderRadius::all(Val::Px(3.0)),
                ..default()
            },
            BackgroundColor(colors::BG2),
            BorderColor::all(colors::BG3),
        ))
        .with_children(|btn| {
            btn.spawn((
                ComboBoxDisplay { param_id },
                Text::new(selected_text),
                TextFont {
                    font_size: 11.0,
                    ..default()
                },
                TextColor(colors::FG0),
            ));
            btn.spawn((
                Text::new("v"),
                TextFont {
                    font_size: 10.0,
                    ..default()
                },
                TextColor(colors::GREY1),
            ));
        });
}

// ── Systems ───────────────────────────────────────────────────────────────────

/// Handles combobox button click to open/close the root-level dropdown overlay.
///
/// When opening, reads the button's `UiGlobalTransform` (physical pixels) and
/// converts to logical pixels to position the dropdown just below the button.
#[tracing::instrument(skip_all)]
pub fn handle_combobox_click(
    windows: Query<&Window>,
    mut combos: Query<
        (
            &mut ComboBoxWidget,
            &Interaction,
            &UiGlobalTransform,
            &ComputedNode,
        ),
        (With<Button>, Changed<Interaction>, Without<Disabled>),
    >,
    mut dropdowns: Query<&mut Node, With<ComboBoxDropdown>>,
) {
    let scale = windows.single().map(Window::scale_factor).unwrap_or(1.0);

    // Collect updates first to avoid simultaneous mutable borrows.
    let mut updates: Vec<(Entity, bool, f32, f32)> = Vec::new();

    for (mut combo, interaction, ui_transform, computed) in &mut combos {
        if *interaction != Interaction::Pressed {
            continue;
        }
        combo.open = !combo.open;

        // UiGlobalTransform origin is the node CENTER (physical pixels).
        // Derive the top-left corner by subtracting half the physical size.
        let center = ui_transform.affine().translation;
        let half_size = computed.size() * 0.5;
        let phys_top_left = center - half_size;

        // The dropdown list is rotated so the selected option is always first,
        // so just align the dropdown top-left to the button top-left (minus 1px
        // to absorb the dropdown's own border).
        let logical_left = phys_top_left.x / scale - 1.0;
        let logical_top = phys_top_left.y / scale - 1.0;

        updates.push((combo.dropdown_entity, combo.open, logical_left, logical_top));
    }

    for (dd_entity, open, left, top) in updates {
        if let Ok(mut node) = dropdowns.get_mut(dd_entity) {
            if open {
                node.left = Val::Px(left);
                node.top = Val::Px(top);
                node.display = Display::Flex;
            } else {
                node.display = Display::None;
            }
        }
    }
}

/// Handles selection of a combobox option.
#[tracing::instrument(skip_all)]
pub fn handle_combobox_option_click(
    options: Query<(&ComboBoxOption, &Interaction), (With<Button>, Changed<Interaction>)>,
    mut combo_states: Query<&mut ComboBoxWidget>,
    mut dropdowns: Query<&mut Node, With<ComboBoxDropdown>>,
    mut displays: Query<(&ComboBoxDisplay, &mut Text)>,
    mut param_displays: Query<(&ParamValueDisplay, &mut Text), Without<ComboBoxDisplay>>,
) {
    for (opt, interaction) in &options {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let param_id = opt.param_id;
        let option_index = opt.option_index;

        for mut c in &mut combo_states {
            if c.param_id == param_id {
                c.selected_index = option_index;
                c.open = false;
                let selected_text = c.options.get(option_index).cloned().unwrap_or_default();

                // Update display text
                for (d, mut text) in &mut displays {
                    if d.param_id == param_id {
                        text.0 = selected_text.clone();
                    }
                }

                // Update right-column value display
                for (d, mut text) in &mut param_displays {
                    if d.param_id == param_id {
                        text.0 = selected_text.clone();
                    }
                }

                // Hide the root-level dropdown overlay by entity.
                if let Ok(mut node) = dropdowns.get_mut(c.dropdown_entity) {
                    node.display = Display::None;
                }
            }
        }
    }
}

/// Updates combobox display text to match current selection.
#[tracing::instrument(skip_all)]
pub fn update_combobox_display(
    combos: Query<&ComboBoxWidget, Changed<ComboBoxWidget>>,
    mut displays: Query<(&ComboBoxDisplay, &mut Text)>,
) {
    for combo in &combos {
        let text = combo
            .options
            .get(combo.selected_index)
            .cloned()
            .unwrap_or_default();
        for (display, mut t) in &mut displays {
            if display.param_id == combo.param_id {
                t.0 = text.clone();
            }
        }
    }
}
