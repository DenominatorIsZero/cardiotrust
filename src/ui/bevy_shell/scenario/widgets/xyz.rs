//! XYZ group widget — three `NumberInputWidget` nodes in a row.

use bevy::prelude::*;

use super::{number_input::spawn_number_input, param_id::ParamId};

/// Spawns three `NumberInputWidget` nodes in a row with X/Y/Z labels.
#[tracing::instrument(skip_all)]
pub fn spawn_xyz_group(commands: &mut ChildSpawnerCommands, param_id: ParamId, values: [f32; 3]) {
    commands
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(8.0),
            ..default()
        })
        .with_children(|row| {
            spawn_number_input(row, param_id, 0, values[0], "X");
            spawn_number_input(row, param_id, 1, values[1], "Y");
            spawn_number_input(row, param_id, 2, values[2], "Z");
        });
}
