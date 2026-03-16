//! Root layout node and content area for the Bevy shell.
//!
//! Spawns the full-screen row that holds the sidebar and content column.
//! Both the spawn and despawn systems are registered in [`super::BevyShellPlugin`].

use bevy::prelude::*;

use crate::ui::colors;

/// Marker for the root layout entity (the outermost row node).
/// Despawning this entity recursively cleans up the entire shell.
#[derive(Component, Debug)]
pub struct ShellRoot;

/// Marker for the breadcrumb bar node at the top of the content column.
#[derive(Component, Debug)]
pub struct BreadcrumbBar;

/// Marker for the content slot node below the breadcrumb bar.
#[derive(Component, Debug)]
pub struct ContentSlot;

/// Spawns the root layout:
///
/// ```text
/// Root (Row, 100%×100%)
/// ├── Sidebar placeholder  ← filled by sidebar::spawn_sidebar (called separately)
/// └── ContentColumn (Column, flex-grow: 1)
///     ├── BreadcrumbBar (height: 32px, BG_DIM)
///     └── ContentSlot   (flex-grow: 1)
/// ```
///
/// The sidebar entity is spawned as a *sibling* of the content column inside
/// the root row. `sidebar::spawn_sidebar` adds it as a child of the same root
/// via a `ChildOf` insert, so order is: Sidebar then ContentColumn.
#[tracing::instrument(skip_all)]
pub fn spawn_root_layout(mut commands: Commands) {
    commands
        .spawn((
            ShellRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                ..default()
            },
        ))
        .with_children(|root| {
            // ContentColumn — sidebar is inserted as first child by sidebar::spawn_sidebar
            root.spawn(Node {
                flex_direction: FlexDirection::Column,
                flex_grow: 1.0,
                height: Val::Percent(100.0),
                ..default()
            })
            .with_children(|col| {
                // Breadcrumb bar
                col.spawn((
                    BreadcrumbBar,
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(32.0),
                        align_items: AlignItems::Center,
                        padding: UiRect::horizontal(Val::Px(12.0)),
                        ..default()
                    },
                    BackgroundColor(colors::BG_DIM),
                ))
                .with_children(|bar| {
                    bar.spawn((
                        super::breadcrumb::BreadcrumbText,
                        Text::new("Home"),
                        TextFont {
                            font_size: 13.0,
                            ..default()
                        },
                        TextColor(colors::GREY1),
                    ));
                });

                // Content slot
                col.spawn((
                    ContentSlot,
                    Node {
                        flex_grow: 1.0,
                        width: Val::Percent(100.0),
                        ..default()
                    },
                ));
            });
        });
}

/// Despawns the shell root and all its children when exiting `UiType::Bevy`.
#[tracing::instrument(skip_all)]
pub fn despawn_root_layout(mut commands: Commands, roots: Query<Entity, With<ShellRoot>>) {
    for entity in &roots {
        commands.entity(entity).despawn();
    }
}
