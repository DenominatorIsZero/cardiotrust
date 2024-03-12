use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use egui::{Slider, Spinner};
use ndarray::s;
use std::{
    collections::HashMap,
    fs,
    path::Path,
    thread::{self, JoinHandle},
};
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter};

use crate::{
    core::{algorithm::metrics::predict_voxeltype, scenario::Scenario},
    vis::plotting::{
        matrix::{
            plot_activation_time, plot_activation_time_delta, plot_states_max,
            plot_states_max_delta, plot_states_over_time, plot_voxel_types,
        },
        time::{standard_time_plot, standard_y_plot},
    },
    ScenarioList, SelectedSenario,
};

#[derive(Default)]
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
    DeltaStatesMeanEpoch,
    DeltaStatesMean,
    DeltaStatesMaxEpoch,
    DeltaStatesMax,
    DeltaMeasurementsMeanEpoch,
    DeltaMeasurementsMean,
    DeltaMeasurementsMax,
    DeltaMeasurementsMaxEpoch,
    DeltaGainsMeanEpoch,
    DeltaGainsMean,
    DeltaGainsMaxEpoch,
    DeltaGainsMax,
    DeltaDelaysMeanEpoch,
    DeltaDelaysMean,
    DeltaDelaysMaxEpoch,
    DeltaDelaysMax,
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

#[derive(Resource)]
pub struct ResultImages {
    pub image_bundles: HashMap<ImageType, ImageBundle>,
}

#[derive(Resource, Default)]
pub struct SelectedResultImage {
    pub image_type: ImageType,
}

#[derive(Resource, Default)]
pub struct PlaybackSpeed {
    pub value: f32,
}

impl Default for ResultImages {
    /// Populates the image bundles with default `ImageBundle` instances for each `ImageType`.
    /// This provides an initial empty set of images that can be rendered.
    fn default() -> Self {
        let mut image_bundles = HashMap::new();

        ImageType::iter().for_each(|image_type| {
            image_bundles.insert(image_type, ImageBundle::default());
        });

        Self { image_bundles }
    }
}

impl ResultImages {
    /// Resets the `ResultImages` to the default state.
    fn reset(&mut self) {
        *self = Self::default();
    }
}

/// Resets the `ResultImages` if the selected scenario has changed.
///
/// This allows the result images to be cleared when switching between scenarios,
/// so that the new images can be loaded.
#[allow(clippy::needless_pass_by_value)]
pub fn reset_result_images(
    mut result_images: ResMut<ResultImages>,
    selected_scenario: Res<SelectedSenario>,
) {
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
pub fn draw_ui_results(
    mut contexts: EguiContexts,
    mut result_images: ResMut<ResultImages>,
    mut selected_image: ResMut<SelectedResultImage>,
    scenario_list: Res<ScenarioList>,
    selected_scenario: Res<SelectedSenario>,
    mut playback_speed: ResMut<PlaybackSpeed>,
) {
    egui_extras::install_image_loaders(contexts.ctx_mut());
    egui::CentralPanel::default().show(contexts.ctx_mut(), |ui| {
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
                let scenario = &scenario_list.entries[selected_scenario.index.unwrap()].scenario;
                let send_scenario = scenario.clone();
                let send_playback_speed = playback_speed.value;
                thread::spawn(move || {
                    generate_gifs(send_scenario, GifType::StatesAlgorithm, send_playback_speed);
                });
            };
            if ui
                .add(egui::Button::new("Generate Simulation Gif"))
                .clicked()
            {
                let scenario = &scenario_list.entries[selected_scenario.index.unwrap()].scenario;
                let send_scenario = scenario.clone();
                let send_playback_speed = playback_speed.value;
                thread::spawn(move || {
                    generate_gifs(
                        send_scenario,
                        GifType::StatesSimulation,
                        send_playback_speed,
                    );
                });
            };
            if ui.add(egui::Button::new("Export to .npy")).clicked() {
                let scenario = &scenario_list.entries[selected_scenario.index.unwrap()].scenario;
                let send_scenario = scenario.clone();
                thread::spawn(move || {
                    send_scenario.save_npy();
                });
            };
        });
        let image_bundle = result_images
            .image_bundles
            .get_mut(&selected_image.image_type)
            .unwrap();
        if let Some(image_path) = image_bundle.path.as_ref() {
            ui.image(image_path);
        } else {
            let scenario = &scenario_list.entries[selected_scenario.index.unwrap()].scenario;
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
                        generate_image(send_scenario, image_type);
                    }));
                }
            }
            ui.add(Spinner::new().size(480.0));
        }
    });
}

/// Returns the file path for the image of the given type for the provided scenario.
/// Joins the results directory, scenario ID, image folder, image type string,
/// and png extension to generate the path.
fn get_image_path(scenario: &Scenario, image_type: ImageType) -> String {
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
fn generate_image(scenario: Scenario, image_type: ImageType) {
    let mut path = Path::new("results").join(scenario.get_id()).join("img");
    fs::create_dir_all(&path).unwrap();
    path = path.join(image_type.to_string()).with_extension("png");
    if path.is_file() {
        return;
    }
    let _file_name = path.with_extension("");
    let _estimations = &scenario.results.as_ref().unwrap().estimations;
    let _model = scenario.results.as_ref().unwrap().model.as_ref().unwrap();
    let _data = scenario.data.as_ref().unwrap();
    let _metrics = &scenario.results.as_ref().unwrap().metrics;
    match image_type {
        ImageType::StatesMaxAlgorithm => {
            todo!();
            plot_states_max(
                &_estimations.system_states,
                &_model.spatial_description.voxels,
                path.with_extension("").to_str().unwrap(),
                "Maximum Estimated Current Densities",
            );
        }
        ImageType::StatesMaxSimulation => {
            todo!();
            plot_states_max(
                _data.get_system_states(),
                &_model.spatial_description.voxels,
                path.with_extension("").to_str().unwrap(),
                "Maximum Simulated Current Densities",
            );
        }
        ImageType::StatesMaxDelta => {
            todo!();
            plot_states_max_delta(
                &_estimations.system_states,
                _data.get_system_states(),
                &_model.spatial_description.voxels,
                _file_name.to_str().unwrap(),
                "Maximum Current Densities Delta",
            );
        }
        ImageType::ActivationTimeAlgorithm => {
            todo!();
            plot_activation_time(
                &_model.functional_description.ap_params.activation_time_ms,
                _file_name.to_str().unwrap(),
                "Activation Times Algorithm (Start) [ms]",
            );
        }
        ImageType::ActivationTimeSimulation => {
            todo!();
            plot_activation_time(
                _data.get_activation_time_ms(),
                _file_name.to_str().unwrap(),
                "Activation Times Simulation [ms]",
            );
        }
        ImageType::ActivationTimeDelta => {
            todo!();
            plot_activation_time_delta(
                &_model.functional_description.ap_params.activation_time_ms,
                _data.get_activation_time_ms(),
                _file_name.to_str().unwrap(),
                "Activation Times Simulation [ms]",
            );
        }
        ImageType::VoxelTypesAlgorithm => {
            todo!();
            plot_voxel_types(
                &_model.spatial_description.voxels.types.values,
                _file_name.to_str().unwrap(),
                "Voxel Types Algorithm",
            );
        }
        ImageType::VoxelTypesSimulation => {
            todo!();
            plot_voxel_types(
                &_data.get_voxel_types().values,
                _file_name.to_str().unwrap(),
                "Voxel Types Simulation",
            );
        }
        ImageType::VoxelTypesPrediction => {
            todo!();
            plot_voxel_types(
                &predict_voxeltype(
                    _estimations,
                    _data.get_voxel_types(),
                    &_model.spatial_description.voxels.numbers,
                    scenario.summary.unwrap().threshold,
                )
                .values,
                _file_name.to_str().unwrap(),
                "Voxel Types Predictions",
            );
        }
        ImageType::LossEpoch => {
            todo!();
            standard_y_plot(
                &_metrics.loss_epoch.values,
                _file_name.to_str().unwrap(),
                "Sum Loss Per Epoch",
                "Loss",
                "Epoch",
            );
        }
        ImageType::Loss => {
            todo!();
            standard_y_plot(
                &_metrics.loss.values,
                _file_name.to_str().unwrap(),
                "Loss Per Step",
                "Loss",
                "Step",
            );
        }
        ImageType::LossMseEpoch => {
            todo!();
            standard_y_plot(
                &_metrics.loss_mse_epoch.values,
                _file_name.to_str().unwrap(),
                "Sum MSE Loss Per Epoch",
                "Loss",
                "Epoch",
            );
        }
        ImageType::LossMse => {
            todo!();
            standard_y_plot(
                &_metrics.loss_mse.values,
                _file_name.to_str().unwrap(),
                "MSE Loss Per Step",
                "Loss",
                "Step",
            );
        }
        ImageType::LossMaximumRegularizationEpoch => {
            todo!();
            standard_y_plot(
                &_metrics.loss_maximum_regularization_epoch.values,
                _file_name.to_str().unwrap(),
                "Sum Max. Reg. Loss Per Epoch",
                "Loss",
                "Epoch",
            );
        }
        ImageType::LossMaximumRegularization => {
            todo!();
            standard_y_plot(
                &_metrics.loss_maximum_regularization.values,
                _file_name.to_str().unwrap(),
                "Max. Reg. Loss Per Step",
                "Loss",
                "Step",
            );
        }
        ImageType::DeltaStatesMeanEpoch => {
            todo!();
            standard_y_plot(
                &_metrics.delta_states_mean_epoch.values,
                _file_name.to_str().unwrap(),
                "Mean Absolute Error Of System States Per Epoch",
                "Error",
                "Epoch",
            );
        }
        ImageType::DeltaStatesMean => {
            todo!();
            standard_y_plot(
                &_metrics.delta_states_mean.values,
                _file_name.to_str().unwrap(),
                "Mean Absolute Error Of System States Per Step",
                "Error",
                "Step",
            );
        }
        ImageType::DeltaStatesMaxEpoch => {
            todo!();
            standard_y_plot(
                &_metrics.delta_states_max_epoch.values,
                _file_name.to_str().unwrap(),
                "Max Absolute Error Of System States Per Epoch",
                "Error",
                "Epoch",
            );
        }
        ImageType::DeltaStatesMax => {
            todo!();
            standard_y_plot(
                &_metrics.delta_states_max.values,
                _file_name.to_str().unwrap(),
                "Max Absolute Error Of System States Per Step",
                "Error",
                "Step",
            );
        }
        ImageType::DeltaMeasurementsMeanEpoch => {
            todo!();
            standard_y_plot(
                &_metrics.delta_measurements_mean_epoch.values,
                _file_name.to_str().unwrap(),
                "Mean Absolute Error Of Measurements Per Epoch",
                "Error",
                "Epoch",
            );
        }
        ImageType::DeltaMeasurementsMean => {
            todo!();
            standard_y_plot(
                &_metrics.delta_measurements_mean.values,
                _file_name.to_str().unwrap(),
                "Mean Absolute Error Of Measurements Per Step",
                "Error",
                "Step",
            );
        }
        ImageType::DeltaMeasurementsMaxEpoch => {
            todo!();
            standard_y_plot(
                &_metrics.delta_measurements_max_epoch.values,
                _file_name.to_str().unwrap(),
                "Max Absolute Error Of Measurements Per Epoch",
                "Error",
                "Epoch",
            );
        }
        ImageType::DeltaMeasurementsMax => {
            todo!();
            standard_y_plot(
                &_metrics.delta_measurements_max.values,
                _file_name.to_str().unwrap(),
                "Max Absolute Error Of Measurements Per Step",
                "Error",
                "Step",
            );
        }
        ImageType::DeltaGainsMeanEpoch => {
            todo!();
            standard_y_plot(
                &_metrics.delta_gains_mean_epoch.values,
                _file_name.to_str().unwrap(),
                "Final Mean Absolute Error Of Gains Per Epoch",
                "Error",
                "Epoch",
            );
        }
        ImageType::DeltaGainsMean => {
            todo!();
            standard_y_plot(
                &_metrics.delta_gains_mean.values,
                _file_name.to_str().unwrap(),
                "Mean Absolute Error Of Gains Per Step",
                "Error",
                "Step",
            );
        }
        ImageType::DeltaGainsMaxEpoch => {
            todo!();
            standard_y_plot(
                &_metrics.delta_gains_max_epoch.values,
                _file_name.to_str().unwrap(),
                "Final Max Absolute Error Of Gains Per Epoch",
                "Error",
                "Epoch",
            );
        }
        ImageType::DeltaGainsMax => {
            todo!();
            standard_y_plot(
                &_metrics.delta_gains_max.values,
                _file_name.to_str().unwrap(),
                "Max Absolute Error Of Gains Per Step",
                "Error",
                "Step",
            );
        }
        ImageType::DeltaDelaysMeanEpoch => {
            todo!();
            standard_y_plot(
                &_metrics.delta_delays_mean_epoch.values,
                _file_name.to_str().unwrap(),
                "Final Mean Absolute Error Of Delays Per Epoch",
                "Error",
                "Epoch",
            );
        }
        ImageType::DeltaDelaysMean => {
            todo!();
            standard_y_plot(
                &_metrics.delta_delays_mean.values,
                _file_name.to_str().unwrap(),
                "Mean Absolute Error Of Delays Per Step",
                "Error",
                "Step",
            );
        }
        ImageType::DeltaDelaysMaxEpoch => {
            todo!();
            standard_y_plot(
                &_metrics.delta_delays_max_epoch.values,
                _file_name.to_str().unwrap(),
                "Final Max Absolute Error Of Delays Per Epoch",
                "Error",
                "Epoch",
            );
        }
        ImageType::DeltaDelaysMax => {
            todo!();
            standard_y_plot(
                &_metrics.delta_delays_max.values,
                _file_name.to_str().unwrap(),
                "Max Absolute Error Of Delays Per Step",
                "Error",
                "Step",
            );
        }
        ImageType::Dice => {
            todo!();
            standard_y_plot(
                &_metrics.dice_score_over_threshold,
                _file_name.to_str().unwrap(),
                "Dice Score over Threshold",
                "Dice Score",
                "Threshold * 100",
            );
        }
        ImageType::IoU => {
            todo!();
            standard_y_plot(
                &_metrics.iou_over_threshold,
                _file_name.to_str().unwrap(),
                "IoU over Threshold",
                "IoU",
                "Threshold * 100",
            );
        }
        ImageType::Recall => {
            todo!();
            standard_y_plot(
                &_metrics.recall_over_threshold,
                _file_name.to_str().unwrap(),
                "Recall over Threshold",
                "Recall",
                "Threshold * 100",
            );
        }
        ImageType::Precision => {
            todo!();
            standard_y_plot(
                &_metrics.precision_over_threshold,
                _file_name.to_str().unwrap(),
                "Precision over Threshold",
                "Precision",
                "Threshold * 100",
            );
        }
        ImageType::ControlFunctionAlgorithm => {
            todo!();
            standard_time_plot(
                &_model.functional_description.control_function_values.values,
                scenario.config.simulation.as_ref().unwrap().sample_rate_hz,
                _file_name.to_str().unwrap(),
                "Control Function Algorithm",
                "u [A/mm^2]",
            );
        }
        ImageType::ControlFunctionSimulation => {
            todo!();
            standard_time_plot(
                &_data.get_control_function_values().values,
                scenario.config.simulation.as_ref().unwrap().sample_rate_hz,
                _file_name.to_str().unwrap(),
                "Control Function Simulation",
                "u [A/mm^2]",
            );
        }
        ImageType::ControlFunctionDelta => {
            todo!();
            standard_time_plot(
                &(&_model.functional_description.control_function_values.values
                    - &_data.get_control_function_values().values),
                scenario.config.simulation.as_ref().unwrap().sample_rate_hz,
                _file_name.to_str().unwrap(),
                "Control Function Delta",
                "u [A/mm^2]",
            );
        }
        ImageType::StateAlgorithm => {
            todo!();
            standard_time_plot(
                &_estimations
                    .system_states
                    .values
                    .slice(s![.., 0])
                    .to_owned(),
                scenario.config.simulation.as_ref().unwrap().sample_rate_hz,
                _file_name.to_str().unwrap(),
                "System State 0 Algorithm",
                "j [A/mm^2]",
            );
        }
        ImageType::StateSimulation => {
            todo!();
            standard_time_plot(
                &_data.get_system_states().values.slice(s![.., 0]).to_owned(),
                scenario.config.simulation.as_ref().unwrap().sample_rate_hz,
                _file_name.to_str().unwrap(),
                "System State 0 Simulation",
                "j [A/mm^2]",
            );
        }
        ImageType::StateDelta => {
            todo!();
            standard_time_plot(
                &(&_estimations
                    .system_states
                    .values
                    .slice(s![.., 0])
                    .to_owned()
                    - &_data.get_system_states().values.slice(s![.., 0]).to_owned()),
                scenario.config.simulation.as_ref().unwrap().sample_rate_hz,
                _file_name.to_str().unwrap(),
                "System State 0 Delta",
                "j [A/mm^2]",
            );
        }
        ImageType::MeasurementAlgorithm => {
            todo!();
            standard_time_plot(
                &_estimations.measurements.values.slice(s![.., 0]).to_owned(),
                scenario.config.simulation.as_ref().unwrap().sample_rate_hz,
                _file_name.to_str().unwrap(),
                "Measurement 0 Algorithm",
                "z [pT]",
            );
        }
        ImageType::MeasurementSimulation => {
            todo!();
            standard_time_plot(
                &_data.get_measurements().values.slice(s![.., 0]).to_owned(),
                scenario.config.simulation.as_ref().unwrap().sample_rate_hz,
                _file_name.to_str().unwrap(),
                "Measurement 0 Simulation",
                "z [pT]",
            );
        }
        ImageType::MeasurementDelta => {
            todo!();
            standard_time_plot(
                &(&_estimations.measurements.values.slice(s![.., 0]).to_owned()
                    - &_data.get_measurements().values.slice(s![.., 0]).to_owned()),
                scenario.config.simulation.as_ref().unwrap().sample_rate_hz,
                _file_name.to_str().unwrap(),
                "Measurement 0 Delta",
                "z [pT]",
            );
        }
    };
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
fn generate_gifs(scenario: Scenario, gif_type: GifType, playback_speed: f32) {
    let mut path = Path::new("results").join(scenario.get_id()).join("img");
    fs::create_dir_all(&path).unwrap();
    path = path.join(gif_type.to_string()).with_extension("png");
    if path.is_file() {
        return;
    }
    let file_name = path.with_extension("");
    let model = scenario.results.as_ref().unwrap().model.as_ref().unwrap();
    let data = scenario.data.as_ref().unwrap();
    let estimations = &scenario.results.as_ref().unwrap().estimations;
    match gif_type {
        GifType::StatesAlgorithm => {
            plot_states_over_time(
                &estimations.system_states,
                &model.spatial_description.voxels,
                20,
                playback_speed,
                file_name.to_str().unwrap(),
                "Estimated Current Densities",
            );
        }
        GifType::StatesSimulation => plot_states_over_time(
            data.get_system_states(),
            &model.spatial_description.voxels,
            20,
            playback_speed,
            file_name.to_str().unwrap(),
            "Estimated Current Densities",
        ),
    }
}
