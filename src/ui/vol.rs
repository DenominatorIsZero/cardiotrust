use std::ops::Deref;

use bevy::prelude::*;
use bevy_editor_cam::controller::component::{EditorCam, EnabledMotion};
use bevy_egui::{egui, EguiContexts};
use egui_plot::{Line, Plot, PlotPoints, VLine};

use crate::{
    vis::{
        cutting_plane::CuttingPlaneSettings,
        heart::{MaterialAtlas, MeshAtlas, VoxelData},
        options::{ColorMode, ColorOptions, VisibilityOptions},
        sample_tracker::SampleTracker,
        sensors::{BacketSettings, SensorBracket, SensorData},
        SetupHeartAndSensors,
    },
    ScenarioList, SelectedSenario,
};

/// Draws the UI for the volumetric visualization, including the side panel
/// controls and the time series plot. Handles initializing the voxel meshes if
/// the "Init Voxels" button is clicked. Updates the visualization mode,
/// playback speed, manual sample control, and sensor selection based on UI
/// interactions.
#[allow(
    clippy::needless_pass_by_value,
    clippy::too_many_arguments,
    clippy::too_many_lines,
    clippy::type_complexity
)]
#[tracing::instrument(skip_all, level = "trace")]
pub fn draw_ui_volumetric(
    mut contexts: EguiContexts,
    mut sample_tracker: ResMut<SampleTracker>,
    mut color_options: ResMut<ColorOptions>,
    mut visibility_options: ResMut<VisibilityOptions>,
    mut cutting_plane: ResMut<CuttingPlaneSettings>,
    mut sensor_bracket_settings: ResMut<BacketSettings>,
    mut cameras: Query<(&mut Transform, &mut EditorCam), With<Camera>>,
    mut ev_setup: EventWriter<SetupHeartAndSensors>,
    selected_scenario: Res<SelectedSenario>,
    scenario_list: Res<ScenarioList>,
) {
    trace!("Running system to draw volumetric UI.");
    for (_, mut camera) in &mut cameras {
        camera.enabled_motion = EnabledMotion {
            pan: true,
            orbit: true,
            zoom: true,
        };
    }
    let scenario = if let Some(index) = selected_scenario.index {
        Some(
            &scenario_list
                .entries
                .get(index)
                .as_ref()
                .expect("Scenario to exist.")
                .scenario,
        )
    } else {
        None
    };
    egui::SidePanel::left("volumetric_left_panel").show(contexts.ctx_mut(), |ui| {
        if ui.ui_contains_pointer() {
            for (_, mut camera) in &mut cameras {
                camera.enabled_motion = EnabledMotion {
                    pan: false,
                    orbit: false,
                    zoom: false,
                };
            }
        }
        ui.label(egui::RichText::new("Visibility").underline());
        ui.group(|ui| {
            let mut visible = visibility_options.heart;
            ui.checkbox(&mut visible, "Heart");
            if visible != visibility_options.heart {
                visibility_options.heart = visible;
            }
            let mut visible = visibility_options.cutting_plane;
            ui.checkbox(&mut visible, "Cutting plane");
            if visible != visibility_options.cutting_plane {
                visibility_options.cutting_plane = visible;
            }
            let mut visible = visibility_options.sensors;
            ui.checkbox(&mut visible, "Sensors");
            if visible != visibility_options.sensors {
                visibility_options.sensors = visible;
            }
            let mut visible = visibility_options.sensor_bracket;
            ui.checkbox(&mut visible, "Sensor bracket");
            if visible != visibility_options.sensor_bracket {
                visibility_options.sensor_bracket = visible;
            }
            let mut visible = visibility_options.torso;
            ui.checkbox(&mut visible, "Torso");
            if visible != visibility_options.torso {
                visibility_options.torso = visible;
            }
            let mut visible = visibility_options.room;
            ui.checkbox(&mut visible, "Room");
            if visible != visibility_options.room {
                visibility_options.room = visible;
            }
        });
        let mut enabled = cutting_plane.enabled;
        ui.checkbox(&mut enabled, "Enable cutting plane");
        if enabled != cutting_plane.enabled {
            cutting_plane.enabled = enabled;
        }
        if ui
            .add_enabled(scenario.is_some(), egui::Button::new("Init Voxels"))
            .clicked()
        {
            let scenario = (**scenario.as_ref().unwrap()).clone();
            ev_setup.send(SetupHeartAndSensors(scenario));
        };
        let mut vis_mode = color_options.mode.clone();
        egui::ComboBox::new("cb_vis_mode", "")
            .selected_text(format!("{vis_mode:?}"))
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    &mut vis_mode,
                    ColorMode::EstimationVoxelTypes,
                    "Voxel types (estimation)",
                );
                ui.selectable_value(
                    &mut vis_mode,
                    ColorMode::SimulationVoxelTypes,
                    "Voxel types (simulation)",
                );
                ui.selectable_value(
                    &mut vis_mode,
                    ColorMode::EstimatedCdeNorm,
                    "Cde norm (estimation)",
                );
                ui.selectable_value(
                    &mut vis_mode,
                    ColorMode::SimulatedCdeNorm,
                    "Cde norm (simulation)",
                );
                ui.selectable_value(
                    &mut vis_mode,
                    ColorMode::EstimatedCdeMax,
                    "Cde max (estimation)",
                );
                ui.selectable_value(
                    &mut vis_mode,
                    ColorMode::SimulatedCdeMax,
                    "Cde max (simulation)",
                );
                ui.selectable_value(&mut vis_mode, ColorMode::DeltaCdeMax, "Cde max (delta)");
                ui.selectable_value(
                    &mut vis_mode,
                    ColorMode::EstimatedActivationTime,
                    "Activation time (estimation)",
                );
                ui.selectable_value(
                    &mut vis_mode,
                    ColorMode::SimulatedActivationTime,
                    "Activation time (simulation)",
                );
                ui.selectable_value(
                    &mut vis_mode,
                    ColorMode::DeltaActivationTime,
                    "Activation time (delta)",
                );
            });
        if vis_mode != color_options.mode {
            color_options.mode = vis_mode;
        }
        let mut relative_coloring = color_options.relative_coloring;
        ui.checkbox(&mut relative_coloring, "Relative coloring");
        if relative_coloring != color_options.relative_coloring {
            color_options.relative_coloring = relative_coloring;
        }
        ui.label("Playback speed:");
        let mut playbackspeed = color_options.playbackspeed;
        ui.add(egui::Slider::new(&mut playbackspeed, 0.01..=1.0).logarithmic(true));
        if (playbackspeed - color_options.playbackspeed).abs() > f32::EPSILON {
            color_options.playbackspeed = playbackspeed;
        }
        let mut manual = sample_tracker.manual;
        ui.checkbox(&mut manual, "Manual");
        if manual != sample_tracker.manual {
            sample_tracker.manual = manual;
        }
        ui.label("Sample:");
        let mut current_sample = sample_tracker.current_sample;
        ui.add_enabled(
            sample_tracker.manual,
            egui::Slider::new(&mut current_sample, 0..=sample_tracker.max_sample)
                .drag_value_speed(1.0),
        );
        if current_sample != sample_tracker.current_sample {
            sample_tracker.current_sample = current_sample;
        }
        if scenario.is_some() {
            ui.label("Sensor:");
            let mut selected_sensor = sample_tracker.selected_sensor; // TODO: This needs to live in something like plot options.
            #[allow(clippy::range_minus_one)]
            ui.add(egui::Slider::new(
                &mut selected_sensor,
                0..=scenario
                    .as_ref()
                    .expect("Scenario to be some")
                    .results
                    .as_ref()
                    .expect("Results to be some.")
                    .estimations
                    .measurements
                    .num_sensors()
                    - 1,
            ));
            if selected_sensor != sample_tracker.selected_sensor {
                sample_tracker.selected_sensor = selected_sensor;
            }
            ui.label("Motion Step:");
            let mut motion_step = sample_tracker.selected_beat;
            #[allow(clippy::range_minus_one)]
            ui.add(egui::Slider::new(
                &mut motion_step,
                0..=scenario
                    .as_ref()
                    .expect("Scenario to be some")
                    .results
                    .as_ref()
                    .expect("Results to be some.")
                    .model
                    .as_ref()
                    .expect("Model to be some.")
                    .spatial_description
                    .sensors
                    .array_offsets_mm
                    .shape()[0]
                    - 1,
            ));
            if motion_step != sample_tracker.selected_beat {
                sample_tracker.selected_beat = motion_step;
            }
        }
        ui.label("Cutting plane origin (x, y, z):");

        let mut position = cutting_plane.position;
        ui.horizontal(|ui| {
            ui.add(egui::DragValue::new(&mut position.x).speed(1.0));
            ui.add(egui::DragValue::new(&mut position.y).speed(1.0));
            ui.add(egui::DragValue::new(&mut position.z).speed(1.0));
        });
        if position != cutting_plane.position {
            cutting_plane.position = position;
        }

        ui.label("Cutting plane normal (x, y, z):");

        let mut normal = cutting_plane.normal;
        ui.horizontal(|ui| {
            ui.add(egui::DragValue::new(&mut normal.x).speed(0.01));
            ui.add(egui::DragValue::new(&mut normal.y).speed(0.01));
            ui.add(egui::DragValue::new(&mut normal.z).speed(0.01));
        });
        if normal != cutting_plane.normal {
            cutting_plane.normal = normal.normalize();
        }

        ui.label("Oppacity:");
        let mut opacity = cutting_plane.opacity;
        ui.add(egui::DragValue::new(&mut opacity).speed(0.01));
        #[allow(clippy::float_cmp)]
        if opacity != cutting_plane.opacity {
            cutting_plane.opacity = opacity;
        }

        if ui.ui_contains_pointer() {
            for (_, mut camera) in &mut cameras {
                camera.enabled_motion = EnabledMotion {
                    pan: false,
                    orbit: false,
                    zoom: false,
                };
            }
        }

        ui.label("Sensor positon mm (x, y, z):");
        let mut position = sensor_bracket_settings.offset;
        ui.horizontal(|ui| {
            ui.add(egui::DragValue::new(&mut position.x).speed(1.0));
            ui.add(egui::DragValue::new(&mut position.y).speed(1.0));
            ui.add(egui::DragValue::new(&mut position.z).speed(1.0));
        });
        if position != sensor_bracket_settings.offset {
            sensor_bracket_settings.offset = position;
        }

        ui.label("Sensor radius mm:");

        let mut radius = sensor_bracket_settings.radius;
        ui.add(egui::DragValue::new(&mut radius).speed(1));
        if (radius - sensor_bracket_settings.radius).abs() > 10.0 * f32::EPSILON {
            sensor_bracket_settings.radius = radius;
        }
    });
    if let Some(scenario) = scenario {
        egui::TopBottomPanel::bottom("Volumetric bottom panel")
            .exact_height(400.0)
            .show(contexts.ctx_mut(), |ui| {
                let samplerate_hz = f64::from(scenario.config.simulation.sample_rate_hz);
                let signal: PlotPoints = (0..sample_tracker.max_sample)
                    .map(|i| {
                        #[allow(clippy::cast_precision_loss)]
                        let x = i as f64 / samplerate_hz;
                        [
                            x,
                            f64::from(
                                scenario
                                    .results
                                    .as_ref()
                                    .expect("Results to be some")
                                    .estimations
                                    .measurements[(
                                    sample_tracker.selected_beat,
                                    i,
                                    sample_tracker.selected_sensor,
                                )],
                            ),
                        ]
                    })
                    .collect();
                let sin_line = Line::new(signal);
                #[allow(clippy::cast_precision_loss)]
                let v_line = VLine::new(sample_tracker.current_sample as f64 / samplerate_hz);
                Plot::new("my_plot")
                    .include_x(0)
                    .include_x(1)
                    .show(ui, |plot_ui| {
                        plot_ui.line(sin_line);
                        plot_ui.vline(v_line);
                    });

                if ui.ui_contains_pointer() {
                    for (_, mut camera) in &mut cameras {
                        camera.enabled_motion = EnabledMotion {
                            pan: false,
                            orbit: false,
                            zoom: false,
                        };
                    }
                }
            });
    }
}
