use std::sync::mpsc;

use bevy::{
    prelude::*,
    render::view::screenshot::{save_to_disk, Screenshot},
    ui::UiGlobalTransform,
};
use bevy_editor_cam::controller::component::{EditorCam, EnabledMotion};

#[cfg(feature = "native")]
use super::helpers::default_screenshot_file_name;
use super::{
    helpers::{
        color_mode_label, control_value_text, cycle_color_mode,
        step_f32, step_playback_speed, step_usize, step_vec3_axis,
    },
    plot::{build_signal_plot_image, cursor_left_px, image_from_plot, PlotImageRequest},
    types::{
        BlocksCameraMotion, ColorModeButton, ColorModeButtonLabel, ColorModeChevron,
        ColorModeDropdown, ColorModeOptionButton, ControlAction, ControlActionButton,
        ControlValueText, OverlayPanelHost, OverlayPanelTitle, OverlayTabStrip, PlotCanvas,
        PlotCollapseButton, PlotCollapseLabel, PlotContainer, PlotCursor, PlotEmptyLabel,
        PlotImageNode, PlotImageState, PlotResizeHandle, PlotStatusLabel, ScreenshotDialogReceiver,
        SectionPanel, SectionTabButton, SectionTabLabel, ToolbarFullscreenButton,
        ToolbarFullscreenLabel, ToolbarResetCameraButton, ToolbarScreenshotButton,
        ToolbarScreenshotLabel, VolumetricViewState, DEFAULT_CAMERA_ROTATION,
        DEFAULT_FULLSCREEN_CAMERA_TRANSLATION, DEFAULT_PLOT_HEIGHT, MAX_PLOT_HEIGHT_RATIO,
        MIN_PLOT_HEIGHT,
    },
};
use crate::{
    ui::colors,
    vis::{
        cutting_plane::CuttingPlaneSettings,
        options::{ColorOptions, VisibilityOptions},
        sample_tracker::SampleTracker,
        sensors::BacketSettings,
    },
    ActiveLoadedScenario,
};

#[tracing::instrument(skip_all)]
#[allow(clippy::type_complexity)]
pub(super) fn sync_volumetric_layout(
    windows: Query<&Window>,
    state: Res<VolumetricViewState>,
    mut layout_nodes: ParamSet<(
        Query<&mut Node, With<OverlayPanelHost>>,
        Query<&mut Node, With<OverlayTabStrip>>,
        Query<&mut Node, With<PlotContainer>>,
    )>,
) {
    let Ok(window) = windows.single() else {
        return;
    };

    let panel_width = if window.resolution.width() < super::types::CONTROL_PANEL_BREAKPOINT {
        (window.resolution.width() * 0.72).clamp(280.0, 420.0)
    } else {
        super::types::DEFAULT_PANEL_WIDTH
    };

    for mut node in &mut layout_nodes.p1() {
        node.display = if state.fullscreen {
            Display::None
        } else {
            Display::Flex
        };
    }

    for mut node in &mut layout_nodes.p0() {
        node.display = if state.fullscreen || state.active_section.is_none() {
            Display::None
        } else {
            Display::Flex
        };
        node.width = Val::Px(panel_width);
    }

    for mut node in &mut layout_nodes.p2() {
        node.display = if state.fullscreen {
            Display::None
        } else {
            Display::Flex
        };
        node.height = Val::Px(if state.plot_collapsed {
            super::types::COLLAPSED_PLOT_HEIGHT
        } else {
            state.plot_height
        });
    }
}

#[tracing::instrument(skip_all)]
pub(super) fn update_section_tab_visuals(
    state: Res<VolumetricViewState>,
    mut buttons: Query<(&SectionTabButton, &Interaction, &mut BackgroundColor)>,
    mut labels: Query<(&SectionTabLabel, &mut TextColor)>,
) {
    for (tab, interaction, mut background) in &mut buttons {
        let active = !state.fullscreen && state.active_section == Some(tab.section);
        background.0 = if active {
            colors::ORANGE
        } else if *interaction == Interaction::Hovered {
            colors::BG5
        } else {
            colors::BG3
        };
    }

    for (label, mut color) in &mut labels {
        color.0 = if !state.fullscreen && state.active_section == Some(label.section) {
            colors::BG0
        } else {
            colors::FG0
        };
    }
}

#[tracing::instrument(skip_all)]
#[allow(clippy::type_complexity)]
pub(super) fn update_toolbar_visuals(
    state: Res<VolumetricViewState>,
    mut fullscreen_buttons: Query<&mut BackgroundColor, With<ToolbarFullscreenButton>>,
    mut fullscreen_labels: Query<&mut Text, With<ToolbarFullscreenLabel>>,
    mut screenshot_buttons: Query<
        (&Interaction, &mut BackgroundColor),
        (
            With<ToolbarScreenshotButton>,
            Without<ToolbarFullscreenButton>,
        ),
    >,
    mut screenshot_labels: Query<&mut TextColor, With<ToolbarScreenshotLabel>>,
) {
    for mut background in &mut fullscreen_buttons {
        background.0 = if state.fullscreen {
            colors::ORANGE
        } else {
            colors::BG3
        };
    }

    for mut label in &mut fullscreen_labels {
        label.0 = if state.fullscreen {
            "Exit Fullscreen".to_string()
        } else {
            "Fullscreen".to_string()
        };
    }

    for (interaction, mut background) in &mut screenshot_buttons {
        background.0 = if *interaction == Interaction::Hovered {
            colors::BG5
        } else {
            colors::BG3
        };
    }

    for mut color in &mut screenshot_labels {
        color.0 = colors::FG0;
    }
}

#[tracing::instrument(skip_all)]
pub(super) fn update_plot_labels(
    state: Res<VolumetricViewState>,
    mut collapse_labels: Query<&mut Text, With<PlotCollapseLabel>>,
    mut status_labels: Query<&mut Text, (With<PlotStatusLabel>, Without<PlotCollapseLabel>)>,
) {
    for mut label in &mut collapse_labels {
        label.0 = if state.plot_collapsed {
            "Restore".to_string()
        } else {
            "Collapse".to_string()
        };
    }

    for mut label in &mut status_labels {
        label.0 = if state.plot_collapsed {
            "Signal Plot (collapsed)".to_string()
        } else {
            "Signal Plot".to_string()
        };
    }
}

#[tracing::instrument(skip_all)]
#[allow(clippy::type_complexity)]
pub(super) fn update_color_mode_dropdown(
    state: Res<VolumetricViewState>,
    color_options: Res<ColorOptions>,
    mut dropdowns: Query<&mut Node, With<ColorModeDropdown>>,
    mut text_sets: ParamSet<(
        Query<&mut Text, With<ColorModeButtonLabel>>,
        Query<&mut Text, With<ColorModeChevron>>,
    )>,
    mut buttons: Query<(&Interaction, &mut BackgroundColor), With<ColorModeButton>>,
    mut options: Query<
        (&ColorModeOptionButton, &Interaction, &mut BackgroundColor),
        (With<Button>, Without<ColorModeButton>),
    >,
) {
    for mut dropdown in &mut dropdowns {
        dropdown.display = if state.color_mode_open {
            Display::Flex
        } else {
            Display::None
        };
    }

    for mut label in &mut text_sets.p0() {
        label.0 = color_mode_label(&color_options.mode).to_string();
    }

    for mut chevron in &mut text_sets.p1() {
        chevron.0 = if state.color_mode_open {
            "^".to_string()
        } else {
            "v".to_string()
        };
    }

    for (interaction, mut background) in &mut buttons {
        background.0 = if state.color_mode_open {
            colors::BG3
        } else if *interaction == Interaction::Hovered {
            colors::BG2
        } else {
            colors::BG1
        };
    }

    for (option, interaction, mut background) in &mut options {
        background.0 = if option.mode == color_options.mode {
            colors::BG3
        } else if *interaction == Interaction::Hovered {
            colors::BG2
        } else {
            Color::NONE
        };
    }
}

#[tracing::instrument(skip_all)]
pub(super) fn sync_overlay_panel_contents(
    state: Res<VolumetricViewState>,
    mut titles: Query<&mut Text, With<OverlayPanelTitle>>,
    mut panels: Query<(&SectionPanel, &mut Node)>,
) {
    let title = state
        .active_section
        .map_or("Controls", |section| section.title())
        .to_string();

    for mut text in &mut titles {
        text.0.clone_from(&title);
    }

    for (panel, mut node) in &mut panels {
        node.display = if !state.fullscreen && state.active_section == Some(panel.section) {
            Display::Flex
        } else {
            Display::None
        };
    }
}

#[tracing::instrument(skip_all)]
pub(super) fn update_control_value_labels(
    sample_tracker: Res<SampleTracker>,
    color_options: Res<ColorOptions>,
    visibility_options: Res<VisibilityOptions>,
    cutting_plane: Res<CuttingPlaneSettings>,
    sensor_bracket_settings: Res<BacketSettings>,
    active_loaded_scenario: Res<ActiveLoadedScenario>,
    mut labels: Query<(&ControlValueText, &mut Text)>,
) {
    let scenario = active_loaded_scenario.0.as_ref();

    for (label, mut text) in &mut labels {
        text.0 = control_value_text(
            label.kind,
            scenario,
            &sample_tracker,
            &color_options,
            &visibility_options,
            &cutting_plane,
            &sensor_bracket_settings,
        );
    }
}

#[tracing::instrument(skip_all)]
#[allow(clippy::type_complexity)]
pub(super) fn handle_section_tab_click(
    mut state: ResMut<VolumetricViewState>,
    buttons: Query<(&SectionTabButton, &Interaction), (Changed<Interaction>, With<Button>)>,
) {
    if state.fullscreen {
        return;
    }

    for (button, interaction) in &buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }
        state.color_mode_open = false;
        state.active_section = if state.active_section == Some(button.section) {
            None
        } else {
            Some(button.section)
        };
    }
}

#[tracing::instrument(skip_all)]
pub(super) fn collapse_overlay_on_outside_click(
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    mut state: ResMut<VolumetricViewState>,
    panels: Query<(&UiGlobalTransform, &ComputedNode), With<OverlayPanelHost>>,
    strips: Query<(&UiGlobalTransform, &ComputedNode), With<OverlayTabStrip>>,
) {
    if !mouse.just_pressed(MouseButton::Left) || state.fullscreen || state.active_section.is_none()
    {
        return;
    }

    let Ok(window) = windows.single() else {
        return;
    };
    let Some(cursor) = window.cursor_position() else {
        return;
    };
    let physical_cursor = cursor * window.scale_factor();

    let inside_panel = panels
        .iter()
        .any(|(transform, computed)| computed.contains_point(*transform, physical_cursor));
    let inside_strip = strips
        .iter()
        .any(|(transform, computed)| computed.contains_point(*transform, physical_cursor));

    if !inside_panel && !inside_strip {
        state.active_section = None;
        state.color_mode_open = false;
    }
}

#[tracing::instrument(skip_all)]
#[allow(clippy::type_complexity)]
pub(super) fn handle_toolbar_buttons(
    mut state: ResMut<VolumetricViewState>,
    _screenshot_dialog_rx: ResMut<ScreenshotDialogReceiver>,
    reset_buttons: Query<&Interaction, (Changed<Interaction>, With<ToolbarResetCameraButton>)>,
    fullscreen_buttons: Query<
        &Interaction,
        (
            Changed<Interaction>,
            With<ToolbarFullscreenButton>,
            Without<ToolbarResetCameraButton>,
        ),
    >,
    screenshot_buttons: Query<
        &Interaction,
        (
            Changed<Interaction>,
            With<ToolbarScreenshotButton>,
            Without<ToolbarResetCameraButton>,
            Without<ToolbarFullscreenButton>,
        ),
    >,
    mut cameras: Query<(&mut Transform, &mut EditorCam), With<Camera>>,
) {
    for interaction in &reset_buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }
        if let Ok((mut transform, mut camera)) = cameras.single_mut() {
            *transform = Transform {
                translation: DEFAULT_FULLSCREEN_CAMERA_TRANSLATION,
                rotation: DEFAULT_CAMERA_ROTATION,
                scale: Vec3::ONE,
            };
            camera.last_anchor_depth = 2.0;
        }
    }

    for interaction in &fullscreen_buttons {
        if *interaction == Interaction::Pressed {
            state.fullscreen = !state.fullscreen;
        }
    }

    for interaction in &screenshot_buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }

        #[cfg(feature = "native")]
        {
            let mut screenshot_dialog_rx = _screenshot_dialog_rx;
            if screenshot_dialog_rx.0.is_some() {
                continue;
            }

            let (tx, rx) = mpsc::channel();
            std::thread::spawn(move || {
                let path = rfd::FileDialog::new()
                    .set_title("Save volumetric screenshot")
                    .set_file_name(default_screenshot_file_name())
                    .add_filter("PNG image", &["png"])
                    .save_file();
                let _ = tx.send(path);
            });
            screenshot_dialog_rx.0 = Some(std::sync::Mutex::new(rx));
        }

        #[cfg(not(feature = "native"))]
        {
            info!("Screenshot save dialog is unavailable on wasm32");
        }
    }
}

#[tracing::instrument(skip_all)]
pub(super) fn poll_screenshot_dialog(
    mut commands: Commands,
    mut screenshot_dialog_rx: ResMut<ScreenshotDialogReceiver>,
) {
    let done = screenshot_dialog_rx.0.as_ref().is_some_and(|mutex| {
        mutex.lock().map_or(true, |rx| match rx.try_recv() {
            Ok(Some(path)) => {
                let path = path.display().to_string();
                info!("Saving volumetric screenshot to {path}");
                commands
                    .spawn(Screenshot::primary_window())
                    .observe(save_to_disk(path));
                true
            }
            Ok(None) => {
                info!("Volumetric screenshot save cancelled");
                true
            }
            Err(mpsc::TryRecvError::Empty) => false,
            Err(mpsc::TryRecvError::Disconnected) => true,
        })
    });

    if done {
        screenshot_dialog_rx.0 = None;
    }
}

#[tracing::instrument(skip_all)]
pub(super) fn log_camera_pose_on_f3(
    keyboard: Res<ButtonInput<KeyCode>>,
    cameras: Query<&Transform, With<Camera>>,
) {
    if !keyboard.just_pressed(KeyCode::F3) {
        return;
    }

    let Ok(transform) = cameras.single() else {
        return;
    };

    let forward = transform.forward();
    info!(
        "Volumetric camera pose: translation=({:.3}, {:.3}, {:.3}), rotation=({:.6}, {:.6}, {:.6}, {:.6}), forward=({:.6}, {:.6}, {:.6})",
        transform.translation.x,
        transform.translation.y,
        transform.translation.z,
        transform.rotation.x,
        transform.rotation.y,
        transform.rotation.z,
        transform.rotation.w,
        forward.x,
        forward.y,
        forward.z,
    );
}

#[tracing::instrument(skip_all)]
pub(super) fn handle_color_mode_button(
    mut state: ResMut<VolumetricViewState>,
    buttons: Query<&Interaction, (Changed<Interaction>, With<ColorModeButton>)>,
) {
    for interaction in &buttons {
        if *interaction == Interaction::Pressed {
            state.color_mode_open = !state.color_mode_open;
        }
    }
}

#[tracing::instrument(skip_all)]
#[allow(clippy::type_complexity)]
pub(super) fn handle_color_mode_option_click(
    mut state: ResMut<VolumetricViewState>,
    mut color_options: ResMut<ColorOptions>,
    options: Query<(&ColorModeOptionButton, &Interaction), (Changed<Interaction>, With<Button>)>,
) {
    for (option, interaction) in &options {
        if *interaction != Interaction::Pressed {
            continue;
        }

        color_options.mode = option.mode.clone();
        state.color_mode_open = false;
    }
}

#[tracing::instrument(skip_all)]
#[allow(clippy::type_complexity)]
pub(super) fn handle_control_action_buttons(
    buttons: Query<(&ControlActionButton, &Interaction), (Changed<Interaction>, With<Button>)>,
    mut sample_tracker: ResMut<SampleTracker>,
    mut color_options: ResMut<ColorOptions>,
    mut visibility_options: ResMut<VisibilityOptions>,
    mut cutting_plane: ResMut<CuttingPlaneSettings>,
    mut sensor_bracket_settings: ResMut<BacketSettings>,
    active_loaded_scenario: Res<ActiveLoadedScenario>,
) {
    let scenario = active_loaded_scenario.0.as_ref();

    for (button, interaction) in &buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }

        match button.action {
            ControlAction::CycleColorMode(direction) => {
                color_options.mode = cycle_color_mode(&color_options.mode, direction);
            }
            ControlAction::ToggleRelativeColoring => {
                color_options.relative_coloring = !color_options.relative_coloring;
            }
            ControlAction::StepPlaybackSpeed(direction) => {
                color_options.playbackspeed =
                    step_playback_speed(color_options.playbackspeed, direction);
            }
            ControlAction::ToggleManualMode => {
                sample_tracker.manual = !sample_tracker.manual;
            }
            ControlAction::StepSample(direction) => {
                let max_sample = sample_tracker.max_sample.saturating_sub(1);
                sample_tracker.current_sample =
                    step_usize(sample_tracker.current_sample, direction, max_sample);
            }
            ControlAction::StepBeat(direction) => {
                let beat_max = scenario
                    .and_then(|scenario| scenario.payload.results.model.as_ref())
                    .map_or(0, |model| {
                        model.spatial_description.sensors.array_offsets_mm.shape()[0]
                            .saturating_sub(1)
                    });
                sample_tracker.selected_beat =
                    step_usize(sample_tracker.selected_beat, direction, beat_max);
            }
            ControlAction::StepSensor(direction) => {
                let sensor_max = scenario.map_or(0, |scenario| {
                    scenario
                        .payload
                        .results
                        .estimations
                        .measurements
                        .num_sensors()
                        .saturating_sub(1)
                });
                sample_tracker.selected_sensor =
                    step_usize(sample_tracker.selected_sensor, direction, sensor_max);
            }
            ControlAction::ToggleVisibility(target) => match target {
                super::types::VisibilityTarget::Heart => {
                    visibility_options.heart = !visibility_options.heart;
                }
                super::types::VisibilityTarget::CuttingPlane => {
                    visibility_options.cutting_plane = !visibility_options.cutting_plane;
                }
                super::types::VisibilityTarget::Sensors => {
                    visibility_options.sensors = !visibility_options.sensors;
                }
                super::types::VisibilityTarget::SensorBracket => {
                    visibility_options.sensor_bracket = !visibility_options.sensor_bracket;
                }
                super::types::VisibilityTarget::Torso => {
                    visibility_options.torso = !visibility_options.torso;
                }
                super::types::VisibilityTarget::Room => {
                    visibility_options.room = !visibility_options.room;
                }
            },
            ControlAction::ToggleCuttingPlaneEnabled => {
                cutting_plane.enabled = !cutting_plane.enabled;
            }
            ControlAction::StepCuttingPlanePosition { axis, direction } => {
                step_vec3_axis(&mut cutting_plane.position, axis, direction, 1.0);
            }
            ControlAction::StepCuttingPlaneNormal { axis, direction } => {
                step_vec3_axis(&mut cutting_plane.normal, axis, direction, 0.05);
                if cutting_plane.normal.length_squared() > f32::EPSILON {
                    cutting_plane.normal = cutting_plane.normal.normalize();
                }
            }
            ControlAction::StepCuttingPlaneOpacity(direction) => {
                cutting_plane.opacity = step_f32(cutting_plane.opacity, direction, 0.05, 0.0, 1.0);
            }
            ControlAction::StepSensorBracketOffset { axis, direction } => {
                step_vec3_axis(&mut sensor_bracket_settings.offset, axis, direction, 1.0);
            }
            ControlAction::StepSensorBracketRadius(direction) => {
                sensor_bracket_settings.radius = step_f32(
                    sensor_bracket_settings.radius,
                    direction,
                    1.0,
                    0.0,
                    f32::MAX,
                );
            }
        }
    }
}

#[tracing::instrument(skip_all)]
pub(super) fn handle_plot_collapse_button(
    mut state: ResMut<VolumetricViewState>,
    buttons: Query<&Interaction, (Changed<Interaction>, With<PlotCollapseButton>)>,
) {
    if state.fullscreen {
        return;
    }

    for interaction in &buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }

        if state.plot_collapsed {
            state.plot_collapsed = false;
            state.plot_height = state.last_expanded_plot_height.max(DEFAULT_PLOT_HEIGHT);
        } else {
            state.last_expanded_plot_height = state.plot_height;
            state.plot_collapsed = true;
        }
    }
}

#[tracing::instrument(skip_all)]
pub(super) fn handle_plot_resize(
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    mut state: ResMut<VolumetricViewState>,
    handles: Query<(&UiGlobalTransform, &ComputedNode), With<PlotResizeHandle>>,
    plots: Query<(&UiGlobalTransform, &ComputedNode), With<PlotContainer>>,
    mut dragging: Local<bool>,
) {
    if state.fullscreen || state.plot_collapsed {
        *dragging = false;
        return;
    }

    let Ok(window) = windows.single() else {
        return;
    };
    let Some(cursor) = window.cursor_position() else {
        return;
    };
    let physical_cursor = cursor * window.scale_factor();

    if mouse.just_pressed(MouseButton::Left) {
        *dragging = handles
            .iter()
            .any(|(transform, computed)| computed.contains_point(*transform, physical_cursor));
    }

    if mouse.just_released(MouseButton::Left) {
        *dragging = false;
        return;
    }

    if !*dragging || !mouse.pressed(MouseButton::Left) {
        return;
    }

    let Ok((plot_transform, plot_node)) = plots.single() else {
        return;
    };
    let plot_center = plot_transform.affine().translation;
    let plot_bottom = plot_node.size().y.mul_add(0.5, plot_center.y);
    let max_height = (window.resolution.height() * MAX_PLOT_HEIGHT_RATIO).max(MIN_PLOT_HEIGHT);
    let new_height = ((plot_bottom - physical_cursor.y) / window.scale_factor())
        .clamp(MIN_PLOT_HEIGHT, max_height);

    state.plot_height = new_height;
    state.last_expanded_plot_height = new_height;
}

#[tracing::instrument(skip_all)]
pub(super) fn disable_camera_motion_over_volumetric_ui_preupdate(
    windows: Query<&Window>,
    ui_regions: Query<(&UiGlobalTransform, &ComputedNode), With<BlocksCameraMotion>>,
    mut cameras: Query<&mut EditorCam, With<Camera>>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let Some(cursor) = window.cursor_position() else {
        return;
    };
    let physical_cursor = cursor * window.scale_factor();

    let cursor_over_ui = ui_regions
        .iter()
        .any(|(transform, computed)| computed.contains_point(*transform, physical_cursor));
    if !cursor_over_ui {
        return;
    }

    for mut camera in &mut cameras {
        camera.enabled_motion.zoom = false;
        camera.enabled_motion.pan = false;
        camera.enabled_motion.orbit = false;
    }
}

#[tracing::instrument(skip_all)]
pub(super) fn disable_camera_motion_over_volumetric_ui(
    windows: Query<&Window>,
    ui_regions: Query<(&UiGlobalTransform, &ComputedNode), With<BlocksCameraMotion>>,
    mut cameras: Query<&mut EditorCam, With<Camera>>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let Some(cursor) = window.cursor_position() else {
        return;
    };
    let physical_cursor = cursor * window.scale_factor();

    let cursor_over_ui = ui_regions
        .iter()
        .any(|(transform, computed)| computed.contains_point(*transform, physical_cursor));
    if !cursor_over_ui {
        return;
    }

    for mut camera in &mut cameras {
        camera.enabled_motion = EnabledMotion {
            pan: false,
            orbit: false,
            zoom: false,
        };
    }
}

#[allow(
    clippy::type_complexity,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
#[tracing::instrument(skip_all)]
pub(super) fn sync_plot_visuals(
    windows: Query<&Window>,
    plot_canvases: Query<&ComputedNode, With<PlotCanvas>>,
    mut plot_images: Query<(&mut Node, &mut ImageNode), With<PlotImageNode>>,
    mut cursor_nodes: Query<
        &mut Node,
        (
            With<PlotCursor>,
            Without<PlotImageNode>,
            Without<PlotEmptyLabel>,
        ),
    >,
    mut empty_labels: Query<
        &mut Node,
        (
            With<PlotEmptyLabel>,
            Without<PlotImageNode>,
            Without<PlotCursor>,
        ),
    >,
    mut plot_image_state: ResMut<PlotImageState>,
    mut images: ResMut<Assets<Image>>,
    sample_tracker: Res<SampleTracker>,
    active_loaded_scenario: Res<ActiveLoadedScenario>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let Ok(canvas) = plot_canvases.single() else {
        return;
    };
    let scale = window.scale_factor();
    let canvas_size = canvas.size() / scale;
    let width = canvas_size.x.max(0.0);
    let height = canvas_size.y.max(0.0);

    let scenario = active_loaded_scenario.0.as_ref();
    let Some(scenario) = scenario else {
        set_plot_empty_state(&mut plot_images, &mut cursor_nodes, &mut empty_labels, true);
        return;
    };
    let sample_count = scenario.payload.data.simulation.measurements.shape()[1];
    if sample_count == 0 || width <= 1.0 || height <= 1.0 {
        set_plot_empty_state(&mut plot_images, &mut cursor_nodes, &mut empty_labels, true);
        return;
    }

    let measurements = &scenario.payload.data.simulation.measurements;
    let beat_index = sample_tracker
        .selected_beat
        .min(measurements.shape()[0].saturating_sub(1));
    let sensor_index = sample_tracker
        .selected_sensor
        .min(measurements.shape()[2].saturating_sub(1));
    set_plot_empty_state(
        &mut plot_images,
        &mut cursor_nodes,
        &mut empty_labels,
        false,
    );

    let request = PlotImageRequest {
        width: width.round().max(1.0) as u32,
        height: height.round().max(1.0) as u32,
        beat_index,
        sensor_index,
    };

    if plot_image_state.request.as_ref() != Some(&request) {
        let signal = measurements.slice(ndarray::s![beat_index, .., sensor_index]);
        match build_signal_plot_image(
            signal,
            scenario.scenario.config.simulation.sample_rate_hz,
            &request,
        ) {
            Ok(plot) => {
                let handle = images.add(image_from_plot(&plot));
                plot_image_state.request = Some(request.clone());
                plot_image_state.image = Some(plot);
                plot_image_state.handle = Some(handle);
            }
            Err(error) => {
                error!("Failed to render signal plot image: {error}");
                set_plot_empty_state(&mut plot_images, &mut cursor_nodes, &mut empty_labels, true);
                plot_image_state.request = None;
                plot_image_state.image = None;
                plot_image_state.handle = None;
                return;
            }
        }
    }

    for (mut node, mut image_node) in &mut plot_images {
        if let Some(handle) = &plot_image_state.handle {
            image_node.image = handle.clone();
            node.display = Display::Flex;
        } else {
            node.display = Display::None;
        }
    }

    for mut cursor in &mut cursor_nodes {
        let max_sample_index = sample_count.saturating_sub(1);
        let current_sample = sample_tracker.current_sample.min(max_sample_index);
        cursor.display = Display::Flex;
        if let Some(plot) = plot_image_state.image.as_ref() {
            cursor.left = Val::Px(cursor_left_px(current_sample, sample_count, plot));
        }
    }
}

#[allow(clippy::type_complexity)]
#[tracing::instrument(skip_all)]
fn set_plot_empty_state(
    plot_images: &mut Query<(&mut Node, &mut ImageNode), With<PlotImageNode>>,
    cursor_nodes: &mut Query<
        &mut Node,
        (
            With<PlotCursor>,
            Without<PlotImageNode>,
            Without<PlotEmptyLabel>,
        ),
    >,
    empty_labels: &mut Query<
        &mut Node,
        (
            With<PlotEmptyLabel>,
            Without<PlotImageNode>,
            Without<PlotCursor>,
        ),
    >,
    empty: bool,
) {
    for (mut node, mut image_node) in plot_images.iter_mut() {
        node.display = Display::None;
        image_node.image = Handle::default();
    }
    for mut cursor in cursor_nodes.iter_mut() {
        cursor.display = Display::None;
    }
    for mut label in empty_labels.iter_mut() {
        label.display = if empty { Display::Flex } else { Display::None };
    }
}

#[allow(
    clippy::type_complexity,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)]
#[tracing::instrument(skip_all)]
pub(super) fn handle_plot_click(
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    canvases: Query<(&UiGlobalTransform, &ComputedNode), With<PlotCanvas>>,
    plot_image_state: Res<PlotImageState>,
    mut sample_tracker: ResMut<SampleTracker>,
    active_loaded_scenario: Res<ActiveLoadedScenario>,
    mut dragging_plot: Local<bool>,
) {
    if !sample_tracker.manual {
        return;
    }

    if mouse.just_released(MouseButton::Left) {
        *dragging_plot = false;
    }

    let Some(scenario) = active_loaded_scenario.0.as_ref() else {
        return;
    };
    let Ok(window) = windows.single() else {
        return;
    };
    let Some(cursor) = window.cursor_position() else {
        return;
    };
    let physical_cursor = cursor * window.scale_factor();
    let Some(plot) = plot_image_state.image.as_ref() else {
        return;
    };
    let Ok((transform, computed)) = canvases.single() else {
        return;
    };
    let cursor_inside_canvas = computed.contains_point(*transform, physical_cursor);
    if mouse.just_pressed(MouseButton::Left) {
        *dragging_plot = cursor_inside_canvas;
    }
    if !*dragging_plot || !mouse.pressed(MouseButton::Left) {
        return;
    }
    if !cursor_inside_canvas {
        return;
    }

    let sample_count = scenario.payload.data.simulation.measurements.shape()[1];
    if sample_count == 0 {
        return;
    }

    let center = transform.affine().translation;
    let min_x = computed.size().x.mul_add(-0.5, center.x);
    let canvas_width = computed.size().x.max(1.0);
    let chart_left = (plot.chart_left_px / plot.width as f32).mul_add(canvas_width, min_x);
    let chart_right = (plot.chart_right_px / plot.width as f32).mul_add(canvas_width, min_x);
    let chart_width = (chart_right - chart_left).max(1.0);
    let relative_x = ((physical_cursor.x - chart_left) / chart_width).clamp(0.0, 1.0);
    let max_sample = sample_count.saturating_sub(1);
    sample_tracker.current_sample = (relative_x * max_sample as f32).round() as usize;
}
