use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use egui_plot::{Line, Plot, PlotPoints, VLine};

use crate::{
    vis::{
        self,
        heart::setup_heart_voxels,
        options::{VisMode, VisOptions},
        sample_tracker::SampleTracker,
    },
    ScenarioList, SelectedSenario,
};

use super::scenario;

#[allow(
    clippy::needless_pass_by_value,
    clippy::too_many_arguments,
    clippy::too_many_lines
)]
pub fn draw_ui_volumetric(
    mut contexts: EguiContexts,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut sample_tracker: ResMut<SampleTracker>,
    mut vis_options: ResMut<VisOptions>,
    mut cameras: Query<&mut Transform, With<Camera>>,
    ass: Res<AssetServer>,
    selected_scenario: Res<SelectedSenario>,
    scenario_list: Res<ScenarioList>,
) {
    let mut scenario = None;
    if selected_scenario.index.is_some() {
        scenario = Some(
            &scenario_list
                .entries
                .get(selected_scenario.index.expect("Scenario to be selected."))
                .as_ref()
                .expect("Scenario to exist.")
                .scenario,
        );
    }
    egui::SidePanel::left("volumetric_left_panel").show(contexts.ctx_mut(), |ui| {
        ui.label("Volumetric");
        if ui
            .add_enabled(scenario.is_some(), egui::Button::new("Init Voxels"))
            .clicked()
        {
            setup_heart_voxels(
                &mut commands,
                &mut meshes,
                &mut materials,
                &mut sample_tracker,
                scenario.as_ref().expect("Scenario to be some."),
                &mut cameras.single_mut(),
                ass,
            );
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
        if playbackspeed != vis_options.playbackspeed {
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
    });
    if scenario.is_some() {
        egui::TopBottomPanel::bottom("Volumetric bottom panel")
            .exact_height(400.0)
            .show(contexts.ctx_mut(), |ui| {
                let samplerate_hz = f64::from(
                    scenario
                        .as_ref()
                        .expect("Scenario to be some.")
                        .config
                        .simulation
                        .as_ref()
                        .expect("Simulation to be some.")
                        .sample_rate_hz,
                );
                let sin: PlotPoints = (0..sample_tracker.max_sample)
                    .map(|i| {
                        let x = i as f64 / samplerate_hz;
                        [
                            x,
                            f64::from(
                                scenario
                                    .as_ref()
                                    .expect("Scenario to be some")
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
                let line = Line::new(sin);
                let vline = VLine::new(sample_tracker.current_sample as f64 / samplerate_hz);
                Plot::new("my_plot")
                    .include_x(0)
                    .include_x(1)
                    .auto_bounds_y()
                    .show(ui, |plot_ui| {
                        plot_ui.line(line);
                        plot_ui.vline(vline);
                    });
            });
    }
}
