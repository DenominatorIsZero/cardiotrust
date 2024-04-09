use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use bevy_panorbit_camera::PanOrbitCamera;
use egui_plot::{Line, Plot, PlotPoints, VLine};

use crate::{
    vis::{
        cutting_plane::CuttingPlane,
        heart::MaterialAtlas,
        options::{VisMode, VisOptions},
        sample_tracker::SampleTracker,
        setup_heart_and_sensors,
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
    clippy::too_many_lines
)]
#[tracing::instrument(skip_all, level = "trace")]
pub fn draw_ui_volumetric(
    mut contexts: EguiContexts,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    material_atlas: Res<MaterialAtlas>,
    mut sample_tracker: ResMut<SampleTracker>,
    mut vis_options: ResMut<VisOptions>,
    mut cameras: Query<
        (&mut Transform, &mut PanOrbitCamera),
        (With<Camera>, Without<CuttingPlane>),
    >,
    mut cutting_planes: Query<
        (&mut Transform, &Handle<StandardMaterial>, &mut CuttingPlane),
        Without<Camera>,
    >,
    ass: Res<AssetServer>,
    selected_scenario: Res<SelectedSenario>,
    scenario_list: Res<ScenarioList>,
) {
    trace!("Running system to draw volumetric UI.");
    for (_, mut camera) in &mut cameras {
        camera.enabled = true;
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
        ui.label("Volumetric");
        if ui
            .add_enabled(scenario.is_some(), egui::Button::new("Init Voxels"))
            .clicked()
        {
            setup_heart_and_sensors(
                &mut commands,
                &mut meshes,
                &mut materials,
                &material_atlas,
                &mut sample_tracker,
                scenario.as_ref().expect("Scenario to be some."),
                &mut cameras.single_mut().0,
                ass,
            );
            if ui.ui_contains_pointer() {
                for (_, mut camera) in &mut cameras {
                    camera.enabled = false;
                }
            }
        };
        let mut vis_mode = vis_options.mode.clone();
        egui::ComboBox::new("cb_vis_mode", "")
            .selected_text(format!("{vis_mode:?}"))
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    &mut vis_mode,
                    VisMode::EstimationVoxelTypes,
                    "Voxel types (estimation)",
                );
                ui.selectable_value(
                    &mut vis_mode,
                    VisMode::SimulationVoxelTypes,
                    "Voxel types (simulation)",
                );
                ui.selectable_value(
                    &mut vis_mode,
                    VisMode::EstimatedCdeNorm,
                    "Cde norm (estimation)",
                );
                ui.selectable_value(
                    &mut vis_mode,
                    VisMode::SimulatedCdeNorm,
                    "Cde norm (simulation)",
                );
                ui.selectable_value(
                    &mut vis_mode,
                    VisMode::EstimatedCdeMax,
                    "Cde max (estimation)",
                );
                ui.selectable_value(
                    &mut vis_mode,
                    VisMode::SimulatedCdeMax,
                    "Cde max (simulation)",
                );
            });
        if vis_mode != vis_options.mode {
            vis_options.mode = vis_mode;
        }
        let mut relative_coloring = vis_options.relative_coloring;
        ui.checkbox(&mut relative_coloring, "Relative coloring");
        if relative_coloring != vis_options.relative_coloring {
            vis_options.relative_coloring = relative_coloring;
        }
        ui.label("Playback speed:");
        let mut playbackspeed = vis_options.playbackspeed;
        ui.add(egui::Slider::new(&mut playbackspeed, 0.01..=1.0).logarithmic(true));
        if (playbackspeed - vis_options.playbackspeed).abs() > f32::EPSILON {
            vis_options.playbackspeed = playbackspeed;
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
            egui::Slider::new(&mut current_sample, 0..=sample_tracker.max_sample),
        );
        if current_sample != sample_tracker.current_sample {
            sample_tracker.current_sample = current_sample;
        }
        if scenario.is_some() {
            ui.label("Sensor:");
            let mut selected_sensor = sample_tracker.selected_sensor; // TODO: This needs to live in something like plot options.
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
                    .values
                    .shape()[1],
            ));
            if selected_sensor != sample_tracker.selected_sensor {
                sample_tracker.selected_sensor = selected_sensor;
            }
        }
        cutting_planes
            .iter_mut()
            .for_each(|(mut transform, material, mut cutting_plane)| {
                ui.checkbox(&mut cutting_plane.visible, "Show cutting plane");
                ui.label("Cutting plane origin (x, y, z):");
                ui.horizontal(|ui| {
                    ui.add(egui::DragValue::new(&mut cutting_plane.position.x).speed(1.0));
                    ui.add(egui::DragValue::new(&mut cutting_plane.position.y).speed(1.0));
                    ui.add(egui::DragValue::new(&mut cutting_plane.position.z).speed(1.0));
                });
                ui.label("Cutting plane normal (x, y, z):");
                ui.horizontal(|ui| {
                    ui.add(egui::DragValue::new(&mut cutting_plane.normal.x).speed(0.01));
                    ui.add(egui::DragValue::new(&mut cutting_plane.normal.y).speed(0.01));
                    ui.add(egui::DragValue::new(&mut cutting_plane.normal.z).speed(0.01));
                });
                cutting_plane.normal = cutting_plane.normal.normalize();
                ui.label("Oppacity:");
                ui.add(egui::DragValue::new(&mut cutting_plane.opacity).speed(0.01));
                let opacity = if cutting_plane.visible {
                    cutting_plane.opacity
                } else {
                    0.0
                };
                materials.get_mut(material).unwrap().base_color =
                    Color::rgba(1.0, 1.0, 1.0, opacity);

                transform.translation = cutting_plane.position;
                let rotation = Quat::from_rotation_arc(Vec3::Y, cutting_plane.normal);
                transform.rotation = rotation;
            });

        if ui.ui_contains_pointer() {
            for (_, mut camera) in &mut cameras {
                camera.enabled = false;
            }
        }
    });
    if let Some(scenario) = scenario {
        egui::TopBottomPanel::bottom("Volumetric bottom panel")
            .exact_height(400.0)
            .show(contexts.ctx_mut(), |ui| {
                let samplerate_hz = f64::from(
                    scenario
                        .config
                        .simulation
                        .as_ref()
                        .expect("Simulation to be some.")
                        .sample_rate_hz,
                );
                let sin: PlotPoints = (0..sample_tracker.max_sample)
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
                                    .measurements
                                    .values[(i, sample_tracker.selected_sensor)],
                            ),
                        ]
                    })
                    .collect();
                let sin_line = Line::new(sin);
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
                        camera.enabled = false;
                    }
                }
            });
    }
}
