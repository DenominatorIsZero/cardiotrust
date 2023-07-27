use std::{
    fs,
    path::Path,
    thread::{self, JoinHandle},
};

use bevy::prelude::*;
use egui::{Slider, Spinner, TextureHandle, TextureOptions, Vec2};
use image::io::Reader;
use ndarray::s;
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter};

use bevy_egui::{egui, EguiContexts};
use std::collections::HashMap;

use crate::{
    core::{
        algorithm::metrics,
        scenario::{self, results, Scenario},
    },
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
    pub texture: Option<egui::TextureHandle>,
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
    // Losses
    LossEpoch,
    Loss,
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

#[allow(clippy::module_name_repetitions, clippy::needless_pass_by_value)]
pub fn draw_ui_results(
    mut contexts: EguiContexts,
    mut result_images: ResMut<ResultImages>,
    mut selected_image: ResMut<SelectedResultImage>,
    scenario_list: Res<ScenarioList>,
    selected_scenario: Res<SelectedSenario>,
    mut playback_speed: ResMut<PlaybackSpeed>,
) {
    egui::CentralPanel::default().show(contexts.ctx_mut(), |ui| {
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
        });
        let image_bundle = result_images
            .image_bundles
            .get_mut(&selected_image.image_type)
            .unwrap();
        if let Some(texture) = image_bundle.texture.as_mut() {
            let size = texture.size_vec2() / 1.5;

            ui.image(texture, size);
        } else {
            let scenario = &scenario_list.entries[selected_scenario.index.unwrap()].scenario;
            let send_scenario = scenario.clone();
            let image_type = selected_image.image_type;
            match image_bundle.join_handle.as_mut() {
                Some(join_handle) => {
                    if join_handle.is_finished() {
                        image_bundle.texture =
                            Some(load_image(scenario, selected_image.image_type, ui));
                    } else {
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

fn load_image(scenario: &Scenario, image_type: ImageType, ui: &mut egui::Ui) -> TextureHandle {
    let path = Path::new("results")
        .join(scenario.get_id())
        .join("img")
        .join(image_type.to_string())
        .with_extension("png");
    let img = Reader::open(path).unwrap().decode().unwrap();
    let size = [img.width() as _, img.height() as _];
    let image_buffer = img.to_rgba8();
    let pixels = image_buffer.as_flat_samples();
    ui.ctx().load_texture(
        image_type.to_string(),
        egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice()),
        TextureOptions::default(),
    )
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
            &model.functional_description.ap_params.activation_time_ms,
            file_name.to_str().unwrap(),
            "Activation Times Algorithm (Start) [ms]",
        ),
        ImageType::ActivationTimeSimulation => plot_activation_time(
            data.get_activation_time_ms(),
            file_name.to_str().unwrap(),
            "Activation Times Simulation [ms]",
        ),
        ImageType::ActivationTimeDelta => plot_activation_time_delta(
            &model.functional_description.ap_params.activation_time_ms,
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
            data.get_voxel_types(),
            file_name.to_str().unwrap(),
            "Voxel Types Simulation",
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
        _ => {
            panic!("Generation of {image_type} not yet imlemented.");
        }
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
    let metrics = &scenario.results.as_ref().unwrap().metrics;
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
        _ => {
            panic!("Generation of {gif_type} not yet imlemented.");
        }
    }
}
