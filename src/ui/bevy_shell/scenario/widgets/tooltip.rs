//! Tooltip infrastructure — hover-delay popup for `[?]` help buttons.

use bevy::{prelude::*, ui::UiGlobalTransform};

use crate::ui::colors;

// ── Constants ─────────────────────────────────────────────────────────────────

const TOOLTIP_DELAY_SECS: f32 = 1.0;

// ── Resources ─────────────────────────────────────────────────────────────────

/// Tracks which `[?]` button the cursor is currently over and how long it has
/// been there. Drives the hover-delay tooltip display.
#[derive(Resource, Debug, Default)]
pub struct TooltipState {
    /// Entity of the button currently under the cursor, if any.
    pub hovered_entity: Option<Entity>,
    /// Seconds the cursor has been continuously over `hovered_entity`.
    pub hover_secs: f32,
}

// ── Components ────────────────────────────────────────────────────────────────

/// Marker on a `[?]` tooltip button.
#[derive(Component, Debug, Clone)]
pub struct TooltipButton {
    pub text: String,
}

/// Marker for the shared tooltip overlay (absolute-positioned popup).
#[derive(Component, Debug)]
pub struct TooltipOverlay;

/// Marker for the text node inside the tooltip overlay.
#[derive(Component, Debug)]
pub struct TooltipText;

// ── Spawn ─────────────────────────────────────────────────────────────────────

/// Spawns the shared tooltip overlay as a **root-level** node (no parent) so it
/// is never clipped by any scroll container. Called once during scenario view spawn.
#[tracing::instrument(skip_all)]
pub fn spawn_tooltip_overlay(commands: &mut Commands, _parent: Entity) -> Entity {
    commands
        .spawn((
            TooltipOverlay,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                max_width: Val::Px(280.0),
                padding: UiRect::axes(Val::Px(10.0), Val::Px(8.0)),
                border_radius: BorderRadius::all(Val::Px(5.0)),
                border: UiRect::all(Val::Px(1.0)),
                display: Display::None,
                ..default()
            },
            ZIndex(500),
            BackgroundColor(colors::BG1),
            BorderColor::all(colors::BG3),
        ))
        .with_children(|overlay| {
            overlay.spawn((
                TooltipText,
                Text::new(String::new()),
                TextFont {
                    font_size: 11.0,
                    ..default()
                },
                TextColor(colors::FG0),
            ));
        })
        .id()
}

// ── Systems ───────────────────────────────────────────────────────────────────

/// Every frame: checks which `[?]` button the cursor is over (using the correct
/// Bevy 0.18 `UiGlobalTransform` hit-test), accumulates hover time, and
/// shows/hides the overlay accordingly.
///
/// - Shows the tooltip after the cursor has been over the button for 1 second.
/// - Hides it immediately when the cursor leaves.
#[tracing::instrument(skip_all)]
pub fn handle_tooltip_hover(
    time: Res<Time>,
    windows: Query<&Window>,
    btns: Query<(Entity, &TooltipButton, &UiGlobalTransform, &ComputedNode)>,
    mut state: ResMut<TooltipState>,
    mut overlays: Query<&mut Node, With<TooltipOverlay>>,
    mut texts: Query<&mut Text, With<TooltipText>>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let Some(logical_cursor) = window.cursor_position() else {
        // Cursor left the window — hide immediately.
        state.hovered_entity = None;
        state.hover_secs = 0.0;
        if let Ok(mut node) = overlays.single_mut() {
            node.display = Display::None;
        }
        return;
    };

    let scale = window.scale_factor();
    let physical_cursor = logical_cursor * scale;
    let win_w = window.width();
    let win_h = window.height();

    // Find which button (if any) is under the cursor right now.
    let mut under_cursor: Option<(Entity, &TooltipButton)> = None;
    for (entity, btn, ui_transform, computed) in &btns {
        if computed.contains_point(*ui_transform, physical_cursor) {
            under_cursor = Some((entity, btn));
            break;
        }
    }

    match under_cursor {
        None => {
            // Cursor is not over any button — reset timer and hide overlay.
            if state.hovered_entity.is_some() {
                state.hovered_entity = None;
                state.hover_secs = 0.0;
                if let Ok(mut node) = overlays.single_mut() {
                    node.display = Display::None;
                }
            }
        }
        Some((entity, btn)) => {
            if state.hovered_entity == Some(entity) {
                // Same button as last frame — accumulate time.
                state.hover_secs += time.delta_secs();

                if state.hover_secs >= TOOLTIP_DELAY_SECS {
                    let Ok(mut overlay_node) = overlays.single_mut() else {
                        return;
                    };
                    if overlay_node.display == Display::None {
                        // Show for the first time — position near cursor.
                        let left = (logical_cursor.x + 12.0).clamp(4.0, win_w - 290.0);
                        let top = (logical_cursor.y + 16.0).clamp(4.0, win_h - 120.0);
                        overlay_node.left = Val::Px(left);
                        overlay_node.top = Val::Px(top);
                        overlay_node.display = Display::Flex;

                        let tooltip_text = btn.text.clone();
                        for mut text in &mut texts {
                            text.0.clone_from(&tooltip_text);
                        }
                    }
                }
            } else {
                // Moved onto a new button — reset timer and hide any open tooltip.
                state.hovered_entity = Some(entity);
                state.hover_secs = 0.0;
                if let Ok(mut node) = overlays.single_mut() {
                    node.display = Display::None;
                }
            }
        }
    }
}

/// No-op stub kept so the plugin registration compiles without changes.
#[tracing::instrument(skip_all)]
pub fn handle_tooltip_click(
    _mouse: Res<ButtonInput<MouseButton>>,
    _windows: Query<&Window>,
    _btns: Query<(&TooltipButton, &UiGlobalTransform, &ComputedNode)>,
    _overlays: Query<&mut Node, With<TooltipOverlay>>,
    _texts: Query<&mut Text, With<TooltipText>>,
) {
}

/// No-op stub kept so the plugin registration compiles without changes.
#[tracing::instrument(skip_all)]
pub fn hide_tooltip_on_other_click(
    _mouse: Res<ButtonInput<MouseButton>>,
    _overlays: Query<(), With<TooltipOverlay>>,
) {
}
