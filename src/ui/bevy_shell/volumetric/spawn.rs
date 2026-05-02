use bevy::prelude::*;

use super::{
    helpers::{all_color_modes, color_mode_menu_label},
    types::{
        BlocksCameraMotion, ColorModeButton, ColorModeButtonLabel, ColorModeChevron,
        ColorModeDropdown, ColorModeOptionButton, ControlAction, ControlActionButton,
        ControlValueKind, ControlValueText, OverlayPanelHost, OverlayPanelTitle, OverlayTabStrip,
        PlotCanvas, PlotCollapseButton, PlotCollapseLabel, PlotContainer, PlotCursor,
        PlotEmptyLabel, PlotImageNode, PlotResizeHandle, PlotStatusLabel, SectionPanel,
        SectionTabButton, SectionTabLabel, ToolbarContainer, ToolbarFullscreenButton,
        ToolbarFullscreenLabel, ToolbarResetCameraButton, ToolbarScreenshotButton,
        ToolbarScreenshotLabel, ViewportHost, VisibilityTarget, VolumetricSection,
        VolumetricViewRoot, VolumetricViewState, DEFAULT_PANEL_WIDTH, DEFAULT_PLOT_HEIGHT,
        PANEL_CARD_BG, PANEL_CARD_RADIUS, PANEL_RIGHT_OFFSET, PANEL_ROW_GAP, PANEL_SECTION_GAP,
        PLOT_HANDLE_HEIGHT, SMALL_ACTION_BUTTON_SIZE, TAB_BUTTON_HEIGHT, TAB_STRIP_HALF_HEIGHT,
        TAB_STRIP_WIDTH,
    },
};
use crate::{
    ui::{bevy_shell::content_area::ContentSlot, colors},
    vis::SetupHeartAndSensors,
    ActiveLoadedScenario, LoadedScenario, ScenarioList, SelectedSenario,
};

#[tracing::instrument(skip_all)]
pub(super) fn spawn_volumetric_view(
    mut commands: Commands,
    content_slots: Query<Entity, With<ContentSlot>>,
    windows: Query<&Window>,
    scenario_list: Res<ScenarioList>,
    selected: Res<SelectedSenario>,
    mut view_state: ResMut<VolumetricViewState>,
    mut ev_setup: MessageWriter<SetupHeartAndSensors>,
    mut active_loaded_scenario: ResMut<ActiveLoadedScenario>,
) {
    let Ok(slot) = content_slots.single() else {
        return;
    };

    let _window_width = windows
        .single()
        .map(|window| window.resolution.width())
        .unwrap_or(super::types::CONTROL_PANEL_BREAKPOINT);

    *view_state = VolumetricViewState::default();

    if let Some(index) = selected.index {
        if let Some(entry) = scenario_list.entries.get(index) {
            match entry.load_payload() {
                Ok(payload) => {
                    let loaded = LoadedScenario::from_bundle(entry, payload);
                    ev_setup.write(SetupHeartAndSensors(loaded.clone()));
                    active_loaded_scenario.0 = Some(loaded);
                }
                Err(error) => {
                    error!("Failed to load scenario payload for Volumetric view: {error}");
                    active_loaded_scenario.0 = None;
                }
            }
        }
    }

    let root = commands
        .spawn((
            VolumetricViewRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                min_height: Val::Px(0.0),
                flex_direction: FlexDirection::Column,
                overflow: Overflow::clip(),
                ..default()
            },
            BackgroundColor(Color::NONE),
        ))
        .id();
    commands.entity(slot).add_child(root);

    let workspace = commands
        .spawn((Node {
            flex_grow: 1.0,
            width: Val::Percent(100.0),
            min_height: Val::Px(0.0),
            position_type: PositionType::Relative,
            overflow: Overflow::clip(),
            ..default()
        },))
        .id();
    commands.entity(root).add_child(workspace);

    let viewport = commands
        .spawn((
            ViewportHost,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                min_height: Val::Px(0.0),
                overflow: Overflow::clip(),
                ..default()
            },
            BackgroundColor(Color::NONE),
        ))
        .id();
    commands.entity(workspace).add_child(viewport);

    spawn_toolbar(&mut commands, workspace);
    spawn_overlay_tab_strip(&mut commands, workspace);
    spawn_overlay_panel_host(&mut commands, workspace);
    spawn_plot_container(&mut commands, root);
}

#[tracing::instrument(skip_all)]
pub(super) fn despawn_volumetric_view(
    mut commands: Commands,
    roots: Query<Entity, With<VolumetricViewRoot>>,
    mut view_state: ResMut<VolumetricViewState>,
    mut active_loaded_scenario: ResMut<ActiveLoadedScenario>,
) {
    for entity in &roots {
        commands.entity(entity).despawn();
    }
    *view_state = VolumetricViewState::default();
    active_loaded_scenario.0 = None;
}

#[tracing::instrument(skip_all)]
fn spawn_toolbar(commands: &mut Commands, parent: Entity) {
    let toolbar = commands
        .spawn((
            ToolbarContainer,
            BlocksCameraMotion,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(12.0),
                right: Val::Px(12.0),
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(8.0),
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(6.0)),
                border_radius: BorderRadius::all(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.196, 0.188, 0.184, 0.82)),
            ZIndex(20),
        ))
        .id();
    commands.entity(parent).add_child(toolbar);

    spawn_toolbar_button(commands, toolbar, ToolbarResetCameraButton, "Reset");

    let fullscreen = commands
        .spawn((
            ToolbarFullscreenButton,
            Button,
            Node {
                padding: UiRect::axes(Val::Px(10.0), Val::Px(6.0)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border_radius: BorderRadius::all(Val::Px(6.0)),
                ..default()
            },
            BackgroundColor(colors::BG3),
        ))
        .with_children(|button| {
            button.spawn((
                ToolbarFullscreenLabel,
                Text::new("Fullscreen"),
                TextFont {
                    font_size: 11.0,
                    ..default()
                },
                TextColor(colors::FG0),
            ));
        })
        .id();
    commands.entity(toolbar).add_child(fullscreen);

    let screenshot = commands
        .spawn((
            ToolbarScreenshotButton,
            Button,
            Node {
                padding: UiRect::axes(Val::Px(10.0), Val::Px(6.0)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border_radius: BorderRadius::all(Val::Px(6.0)),
                ..default()
            },
            BackgroundColor(colors::BG3),
        ))
        .with_children(|button| {
            button.spawn((
                ToolbarScreenshotLabel,
                Text::new("Screenshot"),
                TextFont {
                    font_size: 11.0,
                    ..default()
                },
                TextColor(colors::FG0),
            ));
        })
        .id();
    commands.entity(toolbar).add_child(screenshot);
}

#[tracing::instrument(skip_all)]
fn spawn_toolbar_button<T>(commands: &mut Commands, parent: Entity, marker: T, label: &str)
where
    T: Component,
{
    let entity = commands
        .spawn((
            marker,
            Button,
            Node {
                padding: UiRect::axes(Val::Px(10.0), Val::Px(6.0)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border_radius: BorderRadius::all(Val::Px(6.0)),
                ..default()
            },
            BackgroundColor(colors::BG3),
        ))
        .with_children(|button| {
            button.spawn((
                Text::new(label.to_string()),
                TextFont {
                    font_size: 11.0,
                    ..default()
                },
                TextColor(colors::FG0),
            ));
        })
        .id();
    commands.entity(parent).add_child(entity);
}

#[tracing::instrument(skip_all)]
fn spawn_overlay_tab_strip(commands: &mut Commands, parent: Entity) {
    let strip = commands
        .spawn((
            OverlayTabStrip,
            BlocksCameraMotion,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Percent(50.0),
                right: Val::Px(12.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(10.0),
                padding: UiRect::all(Val::Px(6.0)),
                width: Val::Px(TAB_STRIP_WIDTH),
                margin: UiRect::top(Val::Px(-TAB_STRIP_HALF_HEIGHT)),
                border_radius: BorderRadius::all(Val::Px(10.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.196, 0.188, 0.184, 0.82)),
            ZIndex(15),
        ))
        .id();
    commands.entity(parent).add_child(strip);

    for section in [
        VolumetricSection::VoxelColoring,
        VolumetricSection::Visibility,
        VolumetricSection::CuttingPlane,
        VolumetricSection::SensorBracket,
    ] {
        let button = commands
            .spawn((
                SectionTabButton { section },
                Button,
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(TAB_BUTTON_HEIGHT),
                    padding: UiRect::axes(Val::Px(14.0), Val::Px(8.0)),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::FlexStart,
                    column_gap: Val::Px(10.0),
                    border_radius: BorderRadius::all(Val::Px(6.0)),
                    ..default()
                },
                BackgroundColor(colors::BG3),
            ))
            .with_children(|tab| {
                tab.spawn((
                    Text::new(section.sidebar_icon()),
                    TextFont {
                        font_size: 15.0,
                        ..default()
                    },
                    TextColor(colors::GREY1),
                ));
                tab.spawn((
                    SectionTabLabel { section },
                    Text::new(section.title()),
                    TextLayout::new_with_justify(Justify::Center),
                    TextFont {
                        font_size: 13.0,
                        ..default()
                    },
                    TextColor(colors::FG0),
                ));
            })
            .id();
        commands.entity(strip).add_child(button);
    }
}

#[tracing::instrument(skip_all)]
fn spawn_overlay_panel_host(commands: &mut Commands, parent: Entity) {
    let panel = commands
        .spawn((
            OverlayPanelHost,
            BlocksCameraMotion,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(56.0),
                right: Val::Px(PANEL_RIGHT_OFFSET),
                bottom: Val::Px(16.0),
                width: Val::Px(DEFAULT_PANEL_WIDTH),
                min_height: Val::Px(0.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(12.0)),
                row_gap: Val::Px(PANEL_SECTION_GAP),
                border_radius: BorderRadius::all(Val::Px(12.0)),
                display: Display::None,
                ..default()
            },
            BackgroundColor(Color::srgba(0.196, 0.188, 0.184, 0.9)),
            BorderColor::all(colors::BG3),
            ZIndex(12),
        ))
        .with_children(|panel| {
            panel.spawn((
                OverlayPanelTitle,
                Text::new("Controls"),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(colors::FG0),
            ));

            panel
                .spawn(Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(1.0),
                    margin: UiRect::top(Val::Px(8.0)),
                    ..default()
                })
                .insert(BackgroundColor(colors::BG3));

            panel
                .spawn((Node {
                    flex_grow: 1.0,
                    flex_direction: FlexDirection::Column,
                    width: Val::Percent(100.0),
                    min_height: Val::Px(0.0),
                    row_gap: Val::Px(PANEL_SECTION_GAP),
                    overflow: Overflow::scroll_y(),
                    ..default()
                },))
                .with_children(|body| {
                    spawn_voxel_coloring_panel(body);
                    spawn_visibility_panel(body);
                    spawn_cutting_plane_panel(body);
                    spawn_sensor_bracket_panel(body);
                });
        })
        .id();
    commands.entity(parent).add_child(panel);
}

#[tracing::instrument(skip_all)]
fn spawn_voxel_coloring_panel(parent: &mut ChildSpawnerCommands) {
    parent
        .spawn((
            SectionPanel {
                section: VolumetricSection::VoxelColoring,
            },
            Node {
                flex_direction: FlexDirection::Column,
                width: Val::Percent(100.0),
                row_gap: Val::Px(PANEL_ROW_GAP),
                display: Display::None,
                ..default()
            },
        ))
        .with_children(|section| {
            spawn_color_mode_dropdown(section);
            spawn_toggle_row(
                section,
                "Relative Coloring",
                ControlValueKind::RelativeColoring,
                ControlAction::ToggleRelativeColoring,
            );
            spawn_stepper_row(
                section,
                "Playback\nSpeed",
                ControlValueKind::PlaybackSpeed,
                ControlAction::StepPlaybackSpeed(super::types::StepDirection::Decrease),
                ControlAction::StepPlaybackSpeed(super::types::StepDirection::Increase),
            );
            spawn_toggle_row(
                section,
                "Manual Mode",
                ControlValueKind::ManualMode,
                ControlAction::ToggleManualMode,
            );
            spawn_stepper_row(
                section,
                "Sample",
                ControlValueKind::Sample,
                ControlAction::StepSample(super::types::StepDirection::Decrease),
                ControlAction::StepSample(super::types::StepDirection::Increase),
            );
            spawn_stepper_row(
                section,
                "Motion Step",
                ControlValueKind::Beat,
                ControlAction::StepBeat(super::types::StepDirection::Decrease),
                ControlAction::StepBeat(super::types::StepDirection::Increase),
            );
            spawn_stepper_row(
                section,
                "Sensor",
                ControlValueKind::Sensor,
                ControlAction::StepSensor(super::types::StepDirection::Decrease),
                ControlAction::StepSensor(super::types::StepDirection::Increase),
            );
        });
}

#[tracing::instrument(skip_all)]
fn spawn_visibility_panel(parent: &mut ChildSpawnerCommands) {
    parent
        .spawn((
            SectionPanel {
                section: VolumetricSection::Visibility,
            },
            Node {
                flex_direction: FlexDirection::Column,
                width: Val::Percent(100.0),
                row_gap: Val::Px(PANEL_ROW_GAP),
                display: Display::None,
                ..default()
            },
        ))
        .with_children(|section| {
            for (label, target) in [
                ("Heart", VisibilityTarget::Heart),
                ("Cutting Plane", VisibilityTarget::CuttingPlane),
                ("Sensors", VisibilityTarget::Sensors),
                ("Sensor Bracket", VisibilityTarget::SensorBracket),
                ("Torso", VisibilityTarget::Torso),
                ("Room", VisibilityTarget::Room),
            ] {
                spawn_toggle_row(
                    section,
                    label,
                    ControlValueKind::Visibility(target),
                    ControlAction::ToggleVisibility(target),
                );
            }
        });
}

#[tracing::instrument(skip_all)]
fn spawn_cutting_plane_panel(parent: &mut ChildSpawnerCommands) {
    parent
        .spawn((
            SectionPanel {
                section: VolumetricSection::CuttingPlane,
            },
            Node {
                flex_direction: FlexDirection::Column,
                width: Val::Percent(100.0),
                row_gap: Val::Px(PANEL_ROW_GAP),
                display: Display::None,
                ..default()
            },
        ))
        .with_children(|section| {
            spawn_toggle_row(
                section,
                "Enabled",
                ControlValueKind::CuttingPlaneEnabled,
                ControlAction::ToggleCuttingPlaneEnabled,
            );
            spawn_axis_row(
                section,
                "Origin",
                [
                    (
                        "X",
                        ControlValueKind::CuttingPlanePosition(0),
                        ControlAction::StepCuttingPlanePosition {
                            axis: 0,
                            direction: super::types::StepDirection::Decrease,
                        },
                        ControlAction::StepCuttingPlanePosition {
                            axis: 0,
                            direction: super::types::StepDirection::Increase,
                        },
                    ),
                    (
                        "Y",
                        ControlValueKind::CuttingPlanePosition(1),
                        ControlAction::StepCuttingPlanePosition {
                            axis: 1,
                            direction: super::types::StepDirection::Decrease,
                        },
                        ControlAction::StepCuttingPlanePosition {
                            axis: 1,
                            direction: super::types::StepDirection::Increase,
                        },
                    ),
                    (
                        "Z",
                        ControlValueKind::CuttingPlanePosition(2),
                        ControlAction::StepCuttingPlanePosition {
                            axis: 2,
                            direction: super::types::StepDirection::Decrease,
                        },
                        ControlAction::StepCuttingPlanePosition {
                            axis: 2,
                            direction: super::types::StepDirection::Increase,
                        },
                    ),
                ],
            );
            spawn_axis_row(
                section,
                "Normal",
                [
                    (
                        "X",
                        ControlValueKind::CuttingPlaneNormal(0),
                        ControlAction::StepCuttingPlaneNormal {
                            axis: 0,
                            direction: super::types::StepDirection::Decrease,
                        },
                        ControlAction::StepCuttingPlaneNormal {
                            axis: 0,
                            direction: super::types::StepDirection::Increase,
                        },
                    ),
                    (
                        "Y",
                        ControlValueKind::CuttingPlaneNormal(1),
                        ControlAction::StepCuttingPlaneNormal {
                            axis: 1,
                            direction: super::types::StepDirection::Decrease,
                        },
                        ControlAction::StepCuttingPlaneNormal {
                            axis: 1,
                            direction: super::types::StepDirection::Increase,
                        },
                    ),
                    (
                        "Z",
                        ControlValueKind::CuttingPlaneNormal(2),
                        ControlAction::StepCuttingPlaneNormal {
                            axis: 2,
                            direction: super::types::StepDirection::Decrease,
                        },
                        ControlAction::StepCuttingPlaneNormal {
                            axis: 2,
                            direction: super::types::StepDirection::Increase,
                        },
                    ),
                ],
            );
            spawn_stepper_row(
                section,
                "Opacity",
                ControlValueKind::CuttingPlaneOpacity,
                ControlAction::StepCuttingPlaneOpacity(super::types::StepDirection::Decrease),
                ControlAction::StepCuttingPlaneOpacity(super::types::StepDirection::Increase),
            );
        });
}

#[tracing::instrument(skip_all)]
fn spawn_sensor_bracket_panel(parent: &mut ChildSpawnerCommands) {
    parent
        .spawn((
            SectionPanel {
                section: VolumetricSection::SensorBracket,
            },
            Node {
                flex_direction: FlexDirection::Column,
                width: Val::Percent(100.0),
                row_gap: Val::Px(PANEL_ROW_GAP),
                display: Display::None,
                ..default()
            },
        ))
        .with_children(|section| {
            spawn_axis_row(
                section,
                "Position",
                [
                    (
                        "X",
                        ControlValueKind::SensorBracketOffset(0),
                        ControlAction::StepSensorBracketOffset {
                            axis: 0,
                            direction: super::types::StepDirection::Decrease,
                        },
                        ControlAction::StepSensorBracketOffset {
                            axis: 0,
                            direction: super::types::StepDirection::Increase,
                        },
                    ),
                    (
                        "Y",
                        ControlValueKind::SensorBracketOffset(1),
                        ControlAction::StepSensorBracketOffset {
                            axis: 1,
                            direction: super::types::StepDirection::Decrease,
                        },
                        ControlAction::StepSensorBracketOffset {
                            axis: 1,
                            direction: super::types::StepDirection::Increase,
                        },
                    ),
                    (
                        "Z",
                        ControlValueKind::SensorBracketOffset(2),
                        ControlAction::StepSensorBracketOffset {
                            axis: 2,
                            direction: super::types::StepDirection::Decrease,
                        },
                        ControlAction::StepSensorBracketOffset {
                            axis: 2,
                            direction: super::types::StepDirection::Increase,
                        },
                    ),
                ],
            );
            spawn_stepper_row(
                section,
                "Radius",
                ControlValueKind::SensorBracketRadius,
                ControlAction::StepSensorBracketRadius(super::types::StepDirection::Decrease),
                ControlAction::StepSensorBracketRadius(super::types::StepDirection::Increase),
            );
        });
}

#[tracing::instrument(skip_all)]
fn spawn_color_mode_dropdown(parent: &mut ChildSpawnerCommands) {
    parent
        .spawn((
            Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.0),
                padding: UiRect::axes(Val::Px(12.0), Val::Px(10.0)),
                border_radius: BorderRadius::all(Val::Px(PANEL_CARD_RADIUS)),
                ..default()
            },
            BackgroundColor(PANEL_CARD_BG),
        ))
        .with_children(|row| {
            row.spawn((
                Text::new("Color Mode"),
                TextFont {
                    font_size: 13.0,
                    ..default()
                },
                TextColor(colors::FG1),
            ));

            row.spawn((
                ColorModeButton,
                Button,
                Node {
                    width: Val::Percent(100.0),
                    min_height: Val::Px(38.0),
                    padding: UiRect::axes(Val::Px(10.0), Val::Px(8.0)),
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(10.0),
                    border_radius: BorderRadius::all(Val::Px(999.0)),
                    ..default()
                },
                BackgroundColor(colors::BG1),
            ))
            .with_children(|button| {
                button.spawn((
                    ColorModeButtonLabel,
                    Node {
                        flex_grow: 1.0,
                        min_width: Val::Px(0.0),
                        ..default()
                    },
                    Text::new("--"),
                    TextLayout::new_with_justify(Justify::Left),
                    TextFont {
                        font_size: 11.0,
                        ..default()
                    },
                    TextColor(colors::FG0),
                ));
                button.spawn((
                    ColorModeChevron,
                    Text::new("v"),
                    TextFont {
                        font_size: 10.0,
                        ..default()
                    },
                    TextColor(colors::GREY1),
                ));
            });

            row.spawn((
                ColorModeDropdown,
                Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(4.0),
                    padding: UiRect::all(Val::Px(4.0)),
                    border_radius: BorderRadius::all(Val::Px(8.0)),
                    display: Display::None,
                    ..default()
                },
                BackgroundColor(colors::BG1),
                BorderColor::all(colors::BG3),
            ))
            .with_children(|dropdown| {
                for mode in all_color_modes() {
                    dropdown
                        .spawn((
                            ColorModeOptionButton { mode: mode.clone() },
                            Button,
                            Node {
                                width: Val::Percent(100.0),
                                padding: UiRect::axes(Val::Px(8.0), Val::Px(7.0)),
                                border_radius: BorderRadius::all(Val::Px(6.0)),
                                ..default()
                            },
                            BackgroundColor(Color::NONE),
                        ))
                        .with_children(|option| {
                            option.spawn((
                                Text::new(color_mode_menu_label(&mode)),
                                TextFont {
                                    font_size: 11.0,
                                    ..default()
                                },
                                TextColor(colors::FG0),
                            ));
                        });
                }
            });
        });
}

#[tracing::instrument(skip_all)]
fn spawn_toggle_row(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    value_kind: ControlValueKind,
    action: ControlAction,
) {
    parent
        .spawn((
            Node {
                width: Val::Percent(100.0),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                column_gap: Val::Px(12.0),
                padding: UiRect::axes(Val::Px(12.0), Val::Px(10.0)),
                border_radius: BorderRadius::all(Val::Px(PANEL_CARD_RADIUS)),
                ..default()
            },
            BackgroundColor(PANEL_CARD_BG),
        ))
        .with_children(|row| {
            row.spawn((
                Node {
                    flex_grow: 1.0,
                    min_width: Val::Px(0.0),
                    ..default()
                },
                Text::new(label.to_string()),
                TextFont {
                    font_size: 13.0,
                    ..default()
                },
                TextColor(colors::FG1),
            ));
            row.spawn((
                ControlActionButton { action },
                Button,
                Node {
                    padding: UiRect::axes(Val::Px(10.0), Val::Px(6.0)),
                    min_width: Val::Px(72.0),
                    flex_shrink: 0.0,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border_radius: BorderRadius::all(Val::Px(999.0)),
                    ..default()
                },
                BackgroundColor(colors::BG3),
            ))
            .with_children(|button| {
                button.spawn((
                    ControlValueText { kind: value_kind },
                    Text::new("--"),
                    TextFont {
                        font_size: 11.0,
                        ..default()
                    },
                    TextColor(colors::FG0),
                ));
            });
        });
}

#[tracing::instrument(skip_all)]
fn spawn_stepper_row(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    value_kind: ControlValueKind,
    decrease_action: ControlAction,
    increase_action: ControlAction,
) {
    parent
        .spawn((
            Node {
                width: Val::Percent(100.0),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                column_gap: Val::Px(12.0),
                padding: UiRect::axes(Val::Px(12.0), Val::Px(10.0)),
                border_radius: BorderRadius::all(Val::Px(PANEL_CARD_RADIUS)),
                ..default()
            },
            BackgroundColor(PANEL_CARD_BG),
        ))
        .with_children(|row| {
            row.spawn((
                Node {
                    flex_grow: 1.0,
                    min_width: Val::Px(0.0),
                    ..default()
                },
                Text::new(label.to_string()),
                TextFont {
                    font_size: 13.0,
                    ..default()
                },
                TextColor(colors::FG1),
            ));
            row.spawn(Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(8.0),
                flex_shrink: 0.0,
                ..default()
            })
            .with_children(|controls| {
                spawn_small_action_button(controls, "-", decrease_action);
                controls
                    .spawn((
                        Node {
                            width: Val::Px(98.0),
                            min_height: Val::Px(40.0),
                            padding: UiRect::axes(Val::Px(10.0), Val::Px(6.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border_radius: BorderRadius::all(Val::Px(999.0)),
                            ..default()
                        },
                        BackgroundColor(colors::BG1),
                    ))
                    .with_children(|value| {
                        value.spawn((
                            ControlValueText { kind: value_kind },
                            Text::new("--"),
                            TextLayout::new_with_justify(Justify::Center),
                            TextFont {
                                font_size: 11.0,
                                ..default()
                            },
                            TextColor(colors::FG0),
                        ));
                    });
                spawn_small_action_button(controls, "+", increase_action);
            });
        });
}

#[tracing::instrument(skip_all)]
fn spawn_axis_row(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    axes: [(&str, ControlValueKind, ControlAction, ControlAction); 3],
) {
    parent
        .spawn((
            Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.0),
                padding: UiRect::axes(Val::Px(12.0), Val::Px(10.0)),
                border_radius: BorderRadius::all(Val::Px(PANEL_CARD_RADIUS)),
                ..default()
            },
            BackgroundColor(PANEL_CARD_BG),
        ))
        .with_children(|container| {
            container.spawn((
                Text::new(label.to_string()),
                TextFont {
                    font_size: 13.0,
                    ..default()
                },
                TextColor(colors::FG1),
            ));

            container
                .spawn(Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    column_gap: Val::Px(6.0),
                    align_items: AlignItems::Start,
                    ..default()
                })
                .with_children(|row| {
                    for (axis_label, value_kind, decrease_action, increase_action) in axes {
                        row.spawn(Node {
                            flex_grow: 1.0,
                            min_width: Val::Px(0.0),
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            row_gap: Val::Px(4.0),
                            ..default()
                        })
                        .with_children(|axis| {
                            axis.spawn((
                                Text::new(axis_label.to_string()),
                                TextFont {
                                    font_size: 10.0,
                                    ..default()
                                },
                                TextColor(colors::GREY1),
                            ));
                            axis.spawn(Node {
                                flex_direction: FlexDirection::Column,
                                align_items: AlignItems::Center,
                                column_gap: Val::Px(4.0),
                                row_gap: Val::Px(4.0),
                                ..default()
                            })
                            .with_children(|controls| {
                                spawn_small_action_button(controls, "-", decrease_action);
                                controls
                                    .spawn((
                                        Node {
                                            width: Val::Px(54.0),
                                            padding: UiRect::axes(Val::Px(8.0), Val::Px(5.0)),
                                            justify_content: JustifyContent::Center,
                                            align_items: AlignItems::Center,
                                            border_radius: BorderRadius::all(Val::Px(999.0)),
                                            ..default()
                                        },
                                        BackgroundColor(colors::BG1),
                                    ))
                                    .with_children(|value| {
                                        value.spawn((
                                            ControlValueText { kind: value_kind },
                                            Text::new("--"),
                                            TextFont {
                                                font_size: 10.0,
                                                ..default()
                                            },
                                            TextColor(colors::FG0),
                                        ));
                                    });
                                spawn_small_action_button(controls, "+", increase_action);
                            });
                        });
                    }
                });
        });
}

#[tracing::instrument(skip_all)]
fn spawn_small_action_button(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    action: ControlAction,
) {
    parent
        .spawn((
            ControlActionButton { action },
            Button,
            Node {
                width: Val::Px(SMALL_ACTION_BUTTON_SIZE),
                height: Val::Px(SMALL_ACTION_BUTTON_SIZE),
                flex_shrink: 0.0,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border_radius: BorderRadius::all(Val::Px(999.0)),
                ..default()
            },
            BackgroundColor(colors::BG3),
        ))
        .with_children(|button| {
            button.spawn((
                Text::new(label.to_string()),
                TextLayout::new_with_justify(Justify::Center),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(colors::FG0),
            ));
        });
}

#[tracing::instrument(skip_all)]
fn spawn_plot_container(commands: &mut Commands, parent: Entity) {
    let plot = commands
        .spawn((
            PlotContainer,
            BlocksCameraMotion,
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(DEFAULT_PLOT_HEIGHT),
                min_height: Val::Px(0.0),
                flex_shrink: 0.0,
                position_type: PositionType::Relative,
                border: UiRect::top(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(colors::BG_DIM),
            BorderColor::all(colors::BG3),
        ))
        .id();
    commands.entity(parent).add_child(plot);

    let canvas = commands
        .spawn((
            PlotCanvas,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(PLOT_HANDLE_HEIGHT + 8.0),
                left: Val::Px(12.0),
                right: Val::Px(12.0),
                bottom: Val::Px(12.0),
                overflow: Overflow::clip(),
                ..default()
            },
            BackgroundColor(colors::BG0),
        ))
        .with_children(|canvas| {
            canvas.spawn((
                PlotImageNode,
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    top: Val::Px(0.0),
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    display: Display::None,
                    ..default()
                },
                ImageNode::default(),
            ));

            canvas.spawn((
                PlotCursor,
                Node {
                    position_type: PositionType::Absolute,
                    display: Display::None,
                    width: Val::Px(super::plot::PLOT_CURSOR_WIDTH_PX),
                    top: Val::Px(0.0),
                    bottom: Val::Px(0.0),
                    ..default()
                },
                BackgroundColor(colors::ORANGE),
            ));

            canvas.spawn((
                PlotEmptyLabel,
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(18.0),
                    top: Val::Px(12.0),
                    ..default()
                },
                Text::new("No signal data available."),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(colors::GREY1),
            ));
        })
        .id();
    commands.entity(plot).add_child(canvas);

    let handle = commands
        .spawn((
            PlotResizeHandle,
            Button,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(0.0),
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                height: Val::Px(PLOT_HANDLE_HEIGHT),
                padding: UiRect::horizontal(Val::Px(10.0)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
            BackgroundColor(colors::BG1),
            ZIndex(2),
        ))
        .with_children(|bar| {
            bar.spawn((
                PlotStatusLabel,
                Text::new("Signal Plot"),
                TextFont {
                    font_size: 11.0,
                    ..default()
                },
                TextColor(colors::GREY1),
            ));

            bar.spawn((
                PlotCollapseButton,
                Button,
                Node {
                    padding: UiRect::axes(Val::Px(8.0), Val::Px(2.0)),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    border_radius: BorderRadius::all(Val::Px(4.0)),
                    ..default()
                },
                BackgroundColor(colors::BG3),
            ))
            .with_children(|button| {
                button.spawn((
                    PlotCollapseLabel,
                    Text::new("Collapse"),
                    TextFont {
                        font_size: 10.0,
                        ..default()
                    },
                    TextColor(colors::FG0),
                ));
            });
        })
        .id();
    commands.entity(plot).add_child(handle);
}
