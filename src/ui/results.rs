use std::{
    fs,
    path::Path,
    thread::{self, JoinHandle},
};

use bevy::prelude::*;
use egui::{Spinner, TextureHandle, TextureOptions, Vec2};
use image::io::Reader;
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter};

use bevy_egui::{egui, EguiContexts};
use std::collections::HashMap;

use crate::{
    core::scenario::{self, results, Scenario},
    vis::plotting::matrix::{
        plot_activation_time, plot_activation_time_delta, plot_states_max, plot_states_max_delta,
        plot_voxel_types,
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
    ControlFunction,
    State,
    Measurement,
}

#[derive(Resource)]
pub struct ResultImages {
    pub image_bundles: HashMap<ImageType, ImageBundle>,
}

#[derive(Resource, Default)]
pub struct SelectedResultImage {
    pub image_type: ImageType,
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
    scenario_list: ResMut<ScenarioList>,
    selected_scenario: ResMut<SelectedSenario>,
) {
    egui::CentralPanel::default().show(contexts.ctx_mut(), |ui| {
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

#[allow(clippy::needless_pass_by_value)]
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
        _ => {
            panic!("Generation of {image_type} not yet imlemented.");
        }
    };
}
