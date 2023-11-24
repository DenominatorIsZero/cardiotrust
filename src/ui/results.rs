use std::{
    fs,
    path::Path,
    thread::{self, JoinHandle},
};

use bevy::prelude::*;
use egui::{Slider, Spinner};

use ndarray::s;
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter};

use bevy_egui::{egui, EguiContexts};
use std::collections::HashMap;

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
    fn default() -> Self {
        let mut image_bundles = HashMap::new();

        ImageType::iter().for_each(|image_type| {
            image_bundles.insert(image_type, ImageBundle::default());
        });

        Self { image_bundles }
    }
}

impl ResultImages {
    fn reset(&mut self) {
        *self = Self::default();
    }
}

#[allow(clippy::needless_pass_by_value)]
pub fn reset_result_images(
    mut result_images: ResMut<ResultImages>,
    selected_scenario: Res<SelectedSenario>,
) {
    if selected_scenario.is_changed() {
        result_images.reset();
    }
}

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

fn get_image_path(scenario: &Scenario, image_type: ImageType) -> String {
    Path::new("file://results")
        .join(scenario.get_id())
        .join("img")
        .join(image_type.to_string())
        .with_extension("png")
        .to_string_lossy()
        .into_owned()
}

#[allow(clippy::needless_pass_by_value, clippy::too_many_lines)]
fn generate_image(scenario: Scenario, image_type: ImageType) {
    let mut path = Path::new("results").join(scenario.get_id()).join("img");
    fs::create_dir_all(&path).unwrap();
    path = path.join(image_type.to_string()).with_extension("png");
    if path.is_file() {
        return;
    }
    let file_name = path.with_extension("");
    let estimations = &scenario.results.as_ref().unwrap().estimations;
    let model = scenario.results.as_ref().unwrap().model.as_ref().unwrap();
    let data = scenario.data.as_ref().unwrap();
    let metrics = &scenario.results.as_ref().unwrap().metrics;
    match image_type {
        ImageType::StatesMaxAlgorithm => plot_states_max(
            &estimations.system_states,
            &model.spatial_description.voxels,
            path.with_extension("").to_str().unwrap(),
            "Maximum Estimated Current Densities",
        ),
        ImageType::StatesMaxSimulation => plot_states_max(
            data.get_system_states(),
            &model.spatial_description.voxels,
            path.with_extension("").to_str().unwrap(),
            "Maximum Simulated Current Densities",
        ),
        ImageType::StatesMaxDelta => plot_states_max_delta(
            &estimations.system_states,
            data.get_system_states(),
            &model.spatial_description.voxels,
            file_name.to_str().unwrap(),
            "Maximum Current Densities Delta",
        ),
        ImageType::ActivationTimeAlgorithm => plot_activation_time(
            &model
                .functional_description
                .ap_params_normal
                .as_ref()
                .unwrap()
                .activation_time_ms,
            file_name.to_str().unwrap(),
            "Activation Times Algorithm (Start) [ms]",
        ),
        ImageType::ActivationTimeSimulation => plot_activation_time(
            data.get_activation_time_ms(),
            file_name.to_str().unwrap(),
            "Activation Times Simulation [ms]",
        ),
        ImageType::ActivationTimeDelta => plot_activation_time_delta(
            &model
                .functional_description
                .ap_params_normal
                .as_ref()
                .unwrap()
                .activation_time_ms,
            data.get_activation_time_ms(),
            file_name.to_str().unwrap(),
            "Activation Times Simulation [ms]",
        ),
        ImageType::VoxelTypesAlgorithm => plot_voxel_types(
            &model.spatial_description.voxels.types.values,
            file_name.to_str().unwrap(),
            "Voxel Types Algorithm",
        ),
        ImageType::VoxelTypesSimulation => plot_voxel_types(
            &data.get_voxel_types().values,
            file_name.to_str().unwrap(),
            "Voxel Types Simulation",
        ),
        ImageType::VoxelTypesPrediction => plot_voxel_types(
            &predict_voxeltype(
                estimations,
                data.get_voxel_types(),
                &model.spatial_description.voxels.numbers,
                scenario.summary.unwrap().threshold,
            )
            .values,
            file_name.to_str().unwrap(),
            "Voxel Types Predictions",
        ),
        ImageType::LossEpoch => standard_y_plot(
            &metrics.loss_epoch.values,
            file_name.to_str().unwrap(),
            "Sum Loss Per Epoch",
            "Loss",
            "Epoch",
        ),
        ImageType::Loss => standard_y_plot(
            &metrics.loss.values,
            file_name.to_str().unwrap(),
            "Loss Per Step",
            "Loss",
            "Step",
        ),
        ImageType::LossMseEpoch => standard_y_plot(
            &metrics.loss_mse_epoch.values,
            file_name.to_str().unwrap(),
            "Sum MSE Loss Per Epoch",
            "Loss",
            "Epoch",
        ),
        ImageType::LossMse => standard_y_plot(
            &metrics.loss_mse.values,
            file_name.to_str().unwrap(),
            "MSE Loss Per Step",
            "Loss",
            "Step",
        ),
        ImageType::LossMaximumRegularizationEpoch => standard_y_plot(
            &metrics.loss_maximum_regularization_epoch.values,
            file_name.to_str().unwrap(),
            "Sum Max. Reg. Loss Per Epoch",
            "Loss",
            "Epoch",
        ),
        ImageType::LossMaximumRegularization => standard_y_plot(
            &metrics.loss_maximum_regularization.values,
            file_name.to_str().unwrap(),
            "Max. Reg. Loss Per Step",
            "Loss",
            "Step",
        ),
        ImageType::DeltaStatesMeanEpoch => standard_y_plot(
            &metrics.delta_states_mean_epoch.values,
            file_name.to_str().unwrap(),
            "Mean Absolute Error Of System States Per Epoch",
            "Error",
            "Epoch",
        ),
        ImageType::DeltaStatesMean => standard_y_plot(
            &metrics.delta_states_mean.values,
            file_name.to_str().unwrap(),
            "Mean Absolute Error Of System States Per Step",
            "Error",
            "Step",
        ),
        ImageType::DeltaStatesMaxEpoch => standard_y_plot(
            &metrics.delta_states_max_epoch.values,
            file_name.to_str().unwrap(),
            "Max Absolute Error Of System States Per Epoch",
            "Error",
            "Epoch",
        ),
        ImageType::DeltaStatesMax => standard_y_plot(
            &metrics.delta_states_max.values,
            file_name.to_str().unwrap(),
            "Max Absolute Error Of System States Per Step",
            "Error",
            "Step",
        ),
        ImageType::DeltaMeasurementsMeanEpoch => standard_y_plot(
            &metrics.delta_measurements_mean_epoch.values,
            file_name.to_str().unwrap(),
            "Mean Absolute Error Of Measurements Per Epoch",
            "Error",
            "Epoch",
        ),
        ImageType::DeltaMeasurementsMean => standard_y_plot(
            &metrics.delta_measurements_mean.values,
            file_name.to_str().unwrap(),
            "Mean Absolute Error Of Measurements Per Step",
            "Error",
            "Step",
        ),
        ImageType::DeltaMeasurementsMaxEpoch => standard_y_plot(
            &metrics.delta_measurements_max_epoch.values,
            file_name.to_str().unwrap(),
            "Max Absolute Error Of Measurements Per Epoch",
            "Error",
            "Epoch",
        ),
        ImageType::DeltaMeasurementsMax => standard_y_plot(
            &metrics.delta_measurements_max.values,
            file_name.to_str().unwrap(),
            "Max Absolute Error Of Measurements Per Step",
            "Error",
            "Step",
        ),
        ImageType::DeltaGainsMeanEpoch => standard_y_plot(
            &metrics.delta_gains_mean_epoch.values,
            file_name.to_str().unwrap(),
            "Final Mean Absolute Error Of Gains Per Epoch",
            "Error",
            "Epoch",
        ),
        ImageType::DeltaGainsMean => standard_y_plot(
            &metrics.delta_gains_mean.values,
            file_name.to_str().unwrap(),
            "Mean Absolute Error Of Gains Per Step",
            "Error",
            "Step",
        ),
        ImageType::DeltaGainsMaxEpoch => standard_y_plot(
            &metrics.delta_gains_max_epoch.values,
            file_name.to_str().unwrap(),
            "Final Max Absolute Error Of Gains Per Epoch",
            "Error",
            "Epoch",
        ),
        ImageType::DeltaGainsMax => standard_y_plot(
            &metrics.delta_gains_max.values,
            file_name.to_str().unwrap(),
            "Max Absolute Error Of Gains Per Step",
            "Error",
            "Step",
        ),
        ImageType::DeltaDelaysMeanEpoch => standard_y_plot(
            &metrics.delta_delays_mean_epoch.values,
            file_name.to_str().unwrap(),
            "Final Mean Absolute Error Of Delays Per Epoch",
            "Error",
            "Epoch",
        ),
        ImageType::DeltaDelaysMean => standard_y_plot(
            &metrics.delta_delays_mean.values,
            file_name.to_str().unwrap(),
            "Mean Absolute Error Of Delays Per Step",
            "Error",
            "Step",
        ),
        ImageType::DeltaDelaysMaxEpoch => standard_y_plot(
            &metrics.delta_delays_max_epoch.values,
            file_name.to_str().unwrap(),
            "Final Max Absolute Error Of Delays Per Epoch",
            "Error",
            "Epoch",
        ),
        ImageType::DeltaDelaysMax => standard_y_plot(
            &metrics.delta_delays_max.values,
            file_name.to_str().unwrap(),
            "Max Absolute Error Of Delays Per Step",
            "Error",
            "Step",
        ),
        ImageType::Dice => standard_y_plot(
            &metrics.dice_score_over_threshold,
            file_name.to_str().unwrap(),
            "Dice Score over Threshold",
            "Dice Score",
            "Threshold * 100",
        ),
        ImageType::IoU => standard_y_plot(
            &metrics.iou_over_threshold,
            file_name.to_str().unwrap(),
            "IoU over Threshold",
            "IoU",
            "Threshold * 100",
        ),
        ImageType::Recall => standard_y_plot(
            &metrics.recall_over_threshold,
            file_name.to_str().unwrap(),
            "Recall over Threshold",
            "Recall",
            "Threshold * 100",
        ),
        ImageType::Precision => standard_y_plot(
            &metrics.precision_over_threshold,
            file_name.to_str().unwrap(),
            "Precision over Threshold",
            "Precision",
            "Threshold * 100",
        ),
        ImageType::ControlFunctionAlgorithm => standard_time_plot(
            &model.functional_description.control_function_values.values,
            scenario
                .get_config()
                .simulation
                .as_ref()
                .unwrap()
                .sample_rate_hz,
            file_name.to_str().unwrap(),
            "Control Function Algorithm",
            "u [A/mm^2]",
        ),
        ImageType::ControlFunctionSimulation => standard_time_plot(
            &data.get_control_function_values().values,
            scenario
                .get_config()
                .simulation
                .as_ref()
                .unwrap()
                .sample_rate_hz,
            file_name.to_str().unwrap(),
            "Control Function Simulation",
            "u [A/mm^2]",
        ),
        ImageType::ControlFunctionDelta => standard_time_plot(
            &(&model.functional_description.control_function_values.values
                - &data.get_control_function_values().values),
            scenario
                .get_config()
                .simulation
                .as_ref()
                .unwrap()
                .sample_rate_hz,
            file_name.to_str().unwrap(),
            "Control Function Delta",
            "u [A/mm^2]",
        ),
        ImageType::StateAlgorithm => standard_time_plot(
            &estimations.system_states.values.slice(s![.., 0]).to_owned(),
            scenario
                .get_config()
                .simulation
                .as_ref()
                .unwrap()
                .sample_rate_hz,
            file_name.to_str().unwrap(),
            "System State 0 Algorithm",
            "j [A/mm^2]",
        ),
        ImageType::StateSimulation => standard_time_plot(
            &data.get_system_states().values.slice(s![.., 0]).to_owned(),
            scenario
                .get_config()
                .simulation
                .as_ref()
                .unwrap()
                .sample_rate_hz,
            file_name.to_str().unwrap(),
            "System State 0 Simulation",
            "j [A/mm^2]",
        ),
        ImageType::StateDelta => standard_time_plot(
            &(&estimations.system_states.values.slice(s![.., 0]).to_owned()
                - &data.get_system_states().values.slice(s![.., 0]).to_owned()),
            scenario
                .get_config()
                .simulation
                .as_ref()
                .unwrap()
                .sample_rate_hz,
            file_name.to_str().unwrap(),
            "System State 0 Delta",
            "j [A/mm^2]",
        ),
        ImageType::MeasurementAlgorithm => standard_time_plot(
            &estimations.measurements.values.slice(s![.., 0]).to_owned(),
            scenario
                .get_config()
                .simulation
                .as_ref()
                .unwrap()
                .sample_rate_hz,
            file_name.to_str().unwrap(),
            "Measurement 0 Algorithm",
            "z [pT]",
        ),
        ImageType::MeasurementSimulation => standard_time_plot(
            &data.get_measurements().values.slice(s![.., 0]).to_owned(),
            scenario
                .get_config()
                .simulation
                .as_ref()
                .unwrap()
                .sample_rate_hz,
            file_name.to_str().unwrap(),
            "Measurement 0 Simulation",
            "z [pT]",
        ),
        ImageType::MeasurementDelta => standard_time_plot(
            &(&estimations.measurements.values.slice(s![.., 0]).to_owned()
                - &data.get_measurements().values.slice(s![.., 0]).to_owned()),
            scenario
                .get_config()
                .simulation
                .as_ref()
                .unwrap()
                .sample_rate_hz,
            file_name.to_str().unwrap(),
            "Measurement 0 Delta",
            "z [pT]",
        ),
    };
}

#[allow(clippy::needless_pass_by_value, clippy::too_many_lines)]
fn generate_gifs(scenario: Scenario, gif_type: GifType, playback_speed: f32) {
    let mut path = Path::new("results").join(scenario.get_id()).join("img");
    fs::create_dir_all(&path).unwrap();
    path = path.join(gif_type.to_string()).with_extension("png");
    if path.is_file() {
        return;
    }
    let file_name = path.with_extension("");
    let estimations = &scenario.results.as_ref().unwrap().estimations;
    let model = scenario.results.as_ref().unwrap().model.as_ref().unwrap();
    let data = scenario.data.as_ref().unwrap();
    match gif_type {
        GifType::StatesAlgorithm => plot_states_over_time(
            &estimations.system_states,
            &model.spatial_description.voxels,
            20,
            playback_speed,
            file_name.to_str().unwrap(),
            "Estimated Current Densities",
        ),
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
