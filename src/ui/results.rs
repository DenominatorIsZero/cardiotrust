use std::{
    collections::HashMap,
    fs,
    path::Path,
    thread::{self, JoinHandle},
};

use anyhow::{Context, Result};

use bevy::prelude::*;
use bevy_editor_cam::prelude::{EditorCam, EnabledMotion};
use bevy_egui::{egui, EguiContexts};
use egui::{Slider, Spinner};
use ndarray::s;
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter};

use crate::{
    core::{
        algorithm::metrics::predict_voxeltype,
        model::functional::allpass::shapes::ActivationTimeMs, scenario::Scenario,
    },
    vis::plotting::{
        gif::states::states_spherical_plot_over_time,
        png::{
            activation_time::activation_time_plot,
            delay::average_delay_plot,
            line::{standard_log_y_plot, standard_time_plot, standard_y_plot},
            propagation_speed::average_propagation_speed_plot,
            states::states_spherical_plot,
            voxel_type::voxel_type_plot,
        },
        PlotSlice, StateSphericalPlotMode,
    },
    ScenarioList, SelectedSenario,
};

#[derive(Default, Debug)]
pub struct ImageBundle {
    pub path: Option<String>,
    pub join_handle: Option<JoinHandle<()>>,
}

/// An enum representing the different image types that can be displayed in the results UI.
/// Includes slice plots for algorithm/simulation outputs and metrics plots.
#[derive(EnumIter, Debug, PartialEq, Eq, Hash, Display, Default, Clone, Copy)]
pub enum ImageType {
    // 2D-Slices
    #[default]
    StatesMaxAlgorithm,
    StatesMaxSimulation,
    StatesMaxDelta,
    ActivationTimeAlgorithm,
    ActivationTimeSimulation,
    ActivationTimeDelta,
    VoxelTypesAlgorithm,
    VoxelTypesSimulation,
    VoxelTypesPrediction,
    AverageDelaySimulation,
    AveragePropagationSpeedSimulation,
    AverageDelayAlgorithm,
    AveragePropagationSpeedAlgorithm,
    AverageDelayDelta,
    // Metrics
    Dice,
    IoU,
    Recall,
    Precision,
    // Losses
    LossEpoch,
    Loss,
    LossMseEpoch,
    LossMse,
    LossMaximumRegularization,
    LossMaximumRegularizationEpoch,
    // Time functions
    ControlFunctionAlgorithm,
    ControlFunctionSimulation,
    ControlFunctionDelta,
    StateAlgorithm,
    StateSimulation,
    StateDelta,
    MeasurementAlgorithm,
    MeasurementSimulation,
    MeasurementDelta,
}

#[derive(EnumIter, Debug, PartialEq, Eq, Hash, Display, Clone, Copy)]
pub enum GifType {
    StatesAlgorithm,
    StatesSimulation,
}

#[derive(Resource, Debug)]
pub struct ResultImages {
    pub image_bundles: HashMap<ImageType, ImageBundle>,
}

#[derive(Resource, Default, Debug)]
pub struct SelectedResultImage {
    pub image_type: ImageType,
}

#[derive(Resource, Default, Debug)]
pub struct PlaybackSpeed {
    pub value: f32,
}

impl Default for ResultImages {
    /// Populates the image bundles with default `ImageBundle` instances for each `ImageType`.
    /// This provides an initial empty set of images that can be rendered.
    #[tracing::instrument(level = "debug")]
    fn default() -> Self {
        debug!("Creating default result images");
        let mut image_bundles = HashMap::new();

        ImageType::iter().for_each(|image_type| {
            image_bundles.insert(image_type, ImageBundle::default());
        });

        Self { image_bundles }
    }
}

impl ResultImages {
    /// Resets the `ResultImages` to the default state.
    #[tracing::instrument(level = "debug")]
    fn reset(&mut self) {
        debug!("Resetting result images");
        *self = Self::default();
    }
}

/// Resets the `ResultImages` if the selected scenario has changed.
///
/// This allows the result images to be cleared when switching between scenarios,
/// so that the new images can be loaded.
#[allow(clippy::needless_pass_by_value)]
#[tracing::instrument(level = "trace")]
pub fn reset_result_images(
    mut result_images: ResMut<ResultImages>,
    selected_scenario: Res<SelectedSenario>,
) {
    trace!("Runing system to check if result images need to be reset");
    if selected_scenario.is_changed() {
        result_images.reset();
    }
}

/// Draws the UI to display result images for the selected scenario.
///
/// Allows selecting the result image type to display, generating gifs, exporting data,
/// and loading/displaying the image. Handles async image loading in the background.
/// Resets images when switching scenarios.
#[allow(clippy::module_name_repetitions, clippy::needless_pass_by_value)]
#[tracing::instrument(skip_all, level = "trace")]
pub fn draw_ui_results(
    mut contexts: EguiContexts,
    mut result_images: ResMut<ResultImages>,
    mut selected_image: ResMut<SelectedResultImage>,
    scenario_list: Res<ScenarioList>,
    selected_scenario: Res<SelectedSenario>,
    mut playback_speed: ResMut<PlaybackSpeed>,
    mut cameras: Query<&mut EditorCam, With<Camera>>,
) {
    trace!("Runing system to draw results UI");
    let ctx = match contexts.ctx_mut() {
        Ok(ctx) => ctx,
        Err(e) => {
            error!("EGUI context not available: {}", e);
            return;
        }
    };
    egui_extras::install_image_loaders(ctx);
    let ctx = match contexts.ctx_mut() {
        Ok(ctx) => ctx,
        Err(e) => {
            error!("EGUI context not available for central panel: {}", e);
            return;
        }
    };
    egui::CentralPanel::default().show(ctx, |ui| {
        for mut camera in &mut cameras {
            if ui.ui_contains_pointer() {
                camera.enabled_motion = EnabledMotion {
                    pan: false,
                    orbit: false,
                    zoom: false,
                };
            }
        }
        ui.label("");
        ui.horizontal(|ui| {
            egui::ComboBox::new("cb_result_image", "")
                .selected_text(selected_image.image_type.to_string())
                .width(300.0)
                .show_ui(ui, |ui| {
                    ImageType::iter().for_each(|image_type| {
                        ui.selectable_value(
                            &mut selected_image.image_type,
                            image_type,
                            image_type.to_string(),
                        );
                    });
                });
            ui.add(Slider::new(&mut playback_speed.value, 0.001..=0.1));
            if ui
                .add(egui::Button::new("Generate Algorithm Gif"))
                .clicked()
            {
                if let Some(index) = selected_scenario.index {
                    let scenario = &scenario_list.entries[index].scenario;
                    let send_scenario = scenario.clone();
                    let send_playback_speed = playback_speed.value;
                    thread::spawn(move || {
                        if let Err(e) = generate_gifs(send_scenario, GifType::StatesAlgorithm, send_playback_speed) {
                            error!("Failed to generate algorithm GIF: {}", e);
                        }
                    });
                } else {
                    error!("No scenario selected for GIF generation");
                }
            }
            if ui
                .add(egui::Button::new("Generate Simulation Gif"))
                .clicked()
            {
                if let Some(index) = selected_scenario.index {
                    let scenario = &scenario_list.entries[index].scenario;
                    let send_scenario = scenario.clone();
                    let send_playback_speed = playback_speed.value;
                    thread::spawn(move || {
                        if let Err(e) = generate_gifs(
                            send_scenario,
                            GifType::StatesSimulation,
                            send_playback_speed,
                        ) {
                            error!("Failed to generate simulation GIF: {}", e);
                        }
                    });
                } else {
                    error!("No scenario selected for GIF generation");
                }
            }
            if ui.add(egui::Button::new("Export to .npy")).clicked() {
                if let Some(index) = selected_scenario.index {
                    let scenario = &scenario_list.entries[index].scenario;
                    let send_scenario = scenario.clone();
                    thread::spawn(move || {
                        send_scenario.save_npy();
                    });
                } else {
                    error!("No scenario selected for NPY export");
                }
            }
        });
        let Some(image_bundle) = result_images
            .image_bundles
            .get_mut(&selected_image.image_type) else {
                error!("Image bundle not found for type: {:?}", selected_image.image_type);
                return;
            };
        if let Some(image_path) = image_bundle.path.as_ref() {
            ui.image(image_path);
        } else if let Some(index) = selected_scenario.index {
            let scenario = &scenario_list.entries[index].scenario;
            let send_scenario = scenario.clone();
            let image_type = selected_image.image_type;
            match image_bundle.join_handle.as_mut() {
                Some(join_handle) => {
                    if join_handle.is_finished() {
                        image_bundle.path =
                            Some(get_image_path(scenario, selected_image.image_type));
                    }
                }
                None => {
                    image_bundle.join_handle = Some(thread::spawn(move || {
                        if let Err(e) = generate_image(send_scenario, image_type) {
                            error!("Failed to generate image for type {:?}: {}", image_type, e);
                        }
                    }));
                }
            }
            ui.add(Spinner::new().size(480.0));
        } else {
            error!("No scenario selected for image generation");
            ui.label("No scenario selected");
        }
    });
}

/// Returns the file path for the image of the given type for the provided scenario.
/// Joins the results directory, scenario ID, image folder, image type string,
/// and png extension to generate the path.
#[tracing::instrument(level = "debug")]
fn get_image_path(scenario: &Scenario, image_type: ImageType) -> String {
    debug!("Generating image path");
    Path::new("file://results")
        .join(scenario.get_id())
        .join("img")
        .join(image_type.to_string())
        .with_extension("png")
        .to_string_lossy()
        .into_owned()
}

/// Generates the image for the given scenario and image type.
#[allow(
    clippy::needless_pass_by_value,
    clippy::too_many_lines,
    clippy::useless_let_if_seq,
    clippy::no_effect_underscore_binding,
    clippy::collection_is_never_read,
    clippy::used_underscore_binding,
    unreachable_code
)]
#[tracing::instrument(level = "debug")]
fn generate_image(scenario: Scenario, image_type: ImageType) -> Result<()> {
    debug!("Generating image");
    let mut path = Path::new("results").join(scenario.get_id()).join("img");
    fs::create_dir_all(&path)
        .with_context(|| format!("Failed to create image directory: {}", path.display()))?;
    path = path.join(image_type.to_string()).with_extension("png");
    if path.is_file() {
        return Ok(());
    }
    let _file_name = path.with_extension("");
    let Some(results) = scenario.results.as_ref() else {
        return Err(anyhow::anyhow!("Scenario results not available for image generation"));
    };
    let estimations = &results.estimations;
    let Some(model) = results.model.as_ref() else {
        return Err(anyhow::anyhow!("Model not available in results for image generation"));
    };
    let Some(data) = scenario.data.as_ref() else {
        return Err(anyhow::anyhow!("Scenario data not available for image generation"));
    };
    let metrics = &results.metrics;
    match image_type {
        // might want to return this at some later point
        ImageType::StatesMaxAlgorithm => states_spherical_plot(
            &estimations.system_states_spherical,
            &estimations.system_states_spherical_max,
            &model.spatial_description.voxels.positions_mm,
            model.spatial_description.voxels.size_mm,
            &model.spatial_description.voxels.numbers,
            Some(&path),
            None,
            Some(StateSphericalPlotMode::ABS),
            None,
            None,
        ),
        ImageType::StatesMaxSimulation => states_spherical_plot(
            &data.simulation.system_states_spherical,
            &data.simulation.system_states_spherical_max,
            &data
                .simulation
                .model
                .spatial_description
                .voxels
                .positions_mm,
            data.simulation.model.spatial_description.voxels.size_mm,
            &data.simulation.model.spatial_description.voxels.numbers,
            Some(&path),
            None,
            Some(StateSphericalPlotMode::ABS),
            None,
            None,
        ),
        ImageType::StatesMaxDelta => states_spherical_plot(
            &(&data.simulation.system_states_spherical - &estimations.system_states_spherical),
            &(&data.simulation.system_states_spherical_max
                - &estimations.system_states_spherical_max),
            &model.spatial_description.voxels.positions_mm,
            model.spatial_description.voxels.size_mm,
            &model.spatial_description.voxels.numbers,
            Some(&path),
            None,
            Some(StateSphericalPlotMode::ABS),
            None,
            None,
        ),
        ImageType::ActivationTimeAlgorithm => activation_time_plot(
            &model.functional_description.ap_params.activation_time_ms,
            &model.spatial_description.voxels.positions_mm,
            model.spatial_description.voxels.size_mm,
            &path,
            Some(PlotSlice::Z(0)),
        ),
        ImageType::ActivationTimeSimulation => activation_time_plot(
            &data
                .simulation
                .model
                .functional_description
                .ap_params
                .activation_time_ms,
            &model.spatial_description.voxels.positions_mm,
            model.spatial_description.voxels.size_mm,
            &path,
            Some(PlotSlice::Z(0)),
        ),
        ImageType::ActivationTimeDelta => {
            let gt = &data
                .simulation
                .model
                .functional_description
                .ap_params
                .activation_time_ms;
            let estimation = &model.functional_description.ap_params.activation_time_ms;
            let mut delta = ActivationTimeMs::empty(gt.raw_dim());
            for x in 0..delta.shape()[0] {
                for y in 0..delta.shape()[1] {
                    for z in 0..delta.shape()[2] {
                        delta[(x, y, z)] = Some(
                            gt[(x, y, z)].unwrap_or(0.0) - estimation[(x, y, z)].unwrap_or(0.0),
                        );
                    }
                }
            }

            activation_time_plot(
                &delta,
                &model.spatial_description.voxels.positions_mm,
                model.spatial_description.voxels.size_mm,
                &path,
                Some(PlotSlice::Z(0)),
            )
        }
        ImageType::VoxelTypesAlgorithm => voxel_type_plot(
            &model.spatial_description.voxels.types,
            &model.spatial_description.voxels.positions_mm,
            model.spatial_description.voxels.size_mm,
            Some(&path),
            None,
        ),
        ImageType::VoxelTypesSimulation => voxel_type_plot(
            &data.simulation.model.spatial_description.voxels.types,
            &data
                .simulation
                .model
                .spatial_description
                .voxels
                .positions_mm,
            data.simulation.model.spatial_description.voxels.size_mm,
            Some(&path),
            None,
        ),
        ImageType::VoxelTypesPrediction => voxel_type_plot(
            &predict_voxeltype(
                estimations,
                &data.simulation.model.spatial_description.voxels.types,
                &model.spatial_description.voxels.numbers,
                scenario.summary
                    .ok_or_else(|| anyhow::anyhow!("Scenario summary not available for voxel type prediction"))?
                    .threshold,
            ),
            &model.spatial_description.voxels.positions_mm,
            model.spatial_description.voxels.size_mm,
            Some(&path),
            None,
        ),
        ImageType::AverageDelaySimulation => average_delay_plot(
            &data.simulation.average_delays,
            &data.simulation.model.spatial_description.voxels.numbers,
            &data
                .simulation
                .model
                .spatial_description
                .voxels
                .positions_mm,
            data.simulation.model.spatial_description.voxels.size_mm,
            &path,
            None,
            None,
        )
        .map_err(|e| e.into()),
        ImageType::AveragePropagationSpeedSimulation => average_propagation_speed_plot(
            &data.simulation.average_delays,
            &data.simulation.model.spatial_description.voxels.numbers,
            &data
                .simulation
                .model
                .spatial_description
                .voxels
                .positions_mm,
            data.simulation.model.spatial_description.voxels.size_mm,
            data.simulation.sample_rate_hz,
            &path,
            None,
        )
        .map_err(|e| e.into()),
        ImageType::AverageDelayAlgorithm => average_delay_plot(
            &estimations.average_delays,
            &model.spatial_description.voxels.numbers,
            &model.spatial_description.voxels.positions_mm,
            model.spatial_description.voxels.size_mm,
            &path,
            None,
            None,
        )
        .map_err(|e| e.into()),
        ImageType::AveragePropagationSpeedAlgorithm => average_propagation_speed_plot(
            &estimations.average_delays,
            &model.spatial_description.voxels.numbers,
            &model.spatial_description.voxels.positions_mm,
            model.spatial_description.voxels.size_mm,
            data.simulation.sample_rate_hz,
            &path,
            None,
        )
        .map_err(|e| e.into()),
        ImageType::AverageDelayDelta => average_delay_plot(
            &(&data.simulation.average_delays - &estimations.average_delays),
            &model.spatial_description.voxels.numbers,
            &model.spatial_description.voxels.positions_mm,
            model.spatial_description.voxels.size_mm,
            &path,
            None,
            None,
        )
        .map_err(|e| e.into()),
        ImageType::LossEpoch => standard_log_y_plot(
            &metrics.loss_batch,
            &path,
            "Sum Loss Per Epoch",
            "Loss",
            "Epoch",
        ),
        ImageType::Loss => standard_y_plot(&metrics.loss, &path, "Loss Per Step", "Loss", "Step"),
        ImageType::LossMseEpoch => standard_log_y_plot(
            &metrics.loss_mse_batch,
            &path,
            "Sum MSE Loss Per Epoch",
            "Loss",
            "Epoch",
        ),
        ImageType::LossMse => standard_y_plot(
            &metrics.loss_mse,
            &path,
            "MSE Loss Per Step",
            "Loss",
            "Step",
        ),
        ImageType::LossMaximumRegularizationEpoch => standard_log_y_plot(
            &metrics.loss_maximum_regularization_batch,
            &path,
            "Sum Max. Reg. Loss Per Epoch",
            "Loss",
            "Epoch",
        ),
        ImageType::LossMaximumRegularization => standard_y_plot(
            &metrics.loss_maximum_regularization,
            &path,
            "Max. Reg. Loss Per Step",
            "Loss",
            "Step",
        ),
        ImageType::Dice => standard_y_plot(
            &metrics.dice_score_over_threshold,
            &path,
            "Dice Score over Threshold",
            "Dice Score",
            "Threshold * 100",
        ),
        ImageType::IoU => standard_y_plot(
            &metrics.iou_over_threshold,
            &path,
            "IoU over Threshold",
            "IoU",
            "Threshold * 100",
        ),
        ImageType::Recall => standard_y_plot(
            &metrics.recall_over_threshold,
            &path,
            "Recall over Threshold",
            "Recall",
            "Threshold * 100",
        ),
        ImageType::Precision => standard_y_plot(
            &metrics.precision_over_threshold,
            &path,
            "Precision over Threshold",
            "Precision",
            "Threshold * 100",
        ),
        ImageType::ControlFunctionAlgorithm => standard_time_plot(
            &model.functional_description.control_function_values,
            scenario.config.simulation.sample_rate_hz,
            &path,
            "Control Function Algorithm",
            "u [A/mm^2]",
        ),
        ImageType::ControlFunctionSimulation => standard_time_plot(
            &data
                .simulation
                .model
                .functional_description
                .control_function_values,
            scenario.config.simulation.sample_rate_hz,
            &path,
            "Control Function Simulation",
            "u [A/mm^2]",
        ),
        ImageType::ControlFunctionDelta => standard_time_plot(
            &(&*model.functional_description.control_function_values
                - &*data
                    .simulation
                    .model
                    .functional_description
                    .control_function_values),
            scenario.config.simulation.sample_rate_hz,
            &path,
            "Control Function Delta",
            "u [A/mm^2]",
        ),
        ImageType::StateAlgorithm => standard_time_plot(
            &estimations.system_states.slice(s![.., 0]).to_owned(),
            scenario.config.simulation.sample_rate_hz,
            &path,
            "System State 0 Algorithm",
            "j [A/mm^2]",
        ),
        ImageType::StateSimulation => standard_time_plot(
            &data.simulation.system_states.slice(s![.., 0]).to_owned(),
            scenario.config.simulation.sample_rate_hz,
            &path,
            "System State 0 Simulation",
            "j [A/mm^2]",
        ),
        ImageType::StateDelta => standard_time_plot(
            &(&estimations.system_states.slice(s![.., 0]).to_owned()
                - &data.simulation.system_states.slice(s![.., 0]).to_owned()),
            scenario.config.simulation.sample_rate_hz,
            &path,
            "System State 0 Delta",
            "j [A/mm^2]",
        ),
        ImageType::MeasurementAlgorithm => standard_time_plot(
            &estimations.measurements.slice(s![0, .., 0]).to_owned(),
            scenario.config.simulation.sample_rate_hz,
            &path,
            "Measurement 0 Algorithm",
            "z [pT]",
        ),
        ImageType::MeasurementSimulation => standard_time_plot(
            &data.simulation.measurements.slice(s![0, .., 0]).to_owned(),
            scenario.config.simulation.sample_rate_hz,
            &path,
            "Measurement 0 Simulation",
            "z [pT]",
        ),
        ImageType::MeasurementDelta => standard_time_plot(
            &(&estimations.measurements.slice(s![0, .., 0]).to_owned()
                - &data.simulation.measurements.slice(s![0, .., 0]).to_owned()),
            scenario.config.simulation.sample_rate_hz,
            &path,
            "Measurement 0 Delta",
            "z [pT]",
        ),
    }.map_err(|e| anyhow::anyhow!("Plotting error: {}", e))?;
    Ok(())
}

/// Generates animated GIF visualizations of the system states over time from the simulation results.
///
/// For each GIF type specified, renders frames showing the system state values across all voxels
/// over the timespan of the simulation. The frames are combined into an animated GIF with the
/// specified playback speed. Visualizations are saved to the results folder for the scenario.
#[allow(
    clippy::needless_pass_by_value,
    clippy::too_many_lines,
    clippy::useless_let_if_seq
)]
#[tracing::instrument(level = "debug")]
fn generate_gifs(
    scenario: Scenario,
    gif_type: GifType,
    playback_speed: f32,
) -> Result<()> {
    debug!("Generating GIFs for scenario {}", scenario.get_id());
    let mut path = Path::new("results").join(scenario.get_id()).join("img");
    fs::create_dir_all(&path)
        .with_context(|| format!("Failed to create GIF directory: {}", path.display()))?;
    path = path.join(gif_type.to_string()).with_extension("gif");
    if path.is_file() {
        return Ok(());
    }
    let Some(results) = scenario.results.as_ref() else {
        return Err(anyhow::anyhow!("Scenario results not available for GIF generation"));
    };
    let Some(model) = results.model.as_ref() else {
        return Err(anyhow::anyhow!("Model not available in results for GIF generation"));
    };
    let Some(data) = scenario.data.as_ref() else {
        return Err(anyhow::anyhow!("Scenario data not available for GIF generation"));
    };
    let estimations = &results.estimations;
    match gif_type {
        GifType::StatesAlgorithm => states_spherical_plot_over_time(
            &estimations.system_states_spherical,
            &estimations.system_states_spherical_max,
            &model.spatial_description.voxels.positions_mm,
            model.spatial_description.voxels.size_mm,
            scenario.config.simulation.sample_rate_hz,
            &model.spatial_description.voxels.numbers,
            Some(path.as_path()),
            Some(PlotSlice::Z(0)),
            Some(StateSphericalPlotMode::ABS),
            Some(playback_speed),
            Some(20),
        ),
        GifType::StatesSimulation => states_spherical_plot_over_time(
            &data.simulation.system_states_spherical,
            &data.simulation.system_states_spherical_max,
            &data
                .simulation
                .model
                .spatial_description
                .voxels
                .positions_mm,
            model.spatial_description.voxels.size_mm,
            scenario.config.simulation.sample_rate_hz,
            &model.spatial_description.voxels.numbers,
            Some(path.as_path()),
            Some(PlotSlice::Z(0)),
            Some(StateSphericalPlotMode::ABS),
            Some(playback_speed),
            Some(20),
        ),
    }.map_err(|e| anyhow::anyhow!("GIF generation error: {}", e))?;
    Ok(())
}
