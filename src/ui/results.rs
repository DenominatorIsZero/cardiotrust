mod generate;

use std::{
    collections::HashMap,
    path::Path,
    thread::{self, JoinHandle},
};

use bevy::prelude::*;
use bevy_editor_cam::prelude::{EditorCam, EnabledMotion};
use bevy_egui::{egui, EguiContexts};
use egui::{Slider, Spinner};
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter};

use crate::{core::scenario::Scenario, ScenarioList, SelectedSenario};

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
                        if let Err(e) = generate::generate_gifs(
                            send_scenario,
                            GifType::StatesAlgorithm,
                            send_playback_speed,
                        ) {
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
                        if let Err(e) = generate::generate_gifs(
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
                        if let Err(e) = send_scenario.save_npy() {
                            error!("Failed to export scenario to NPY: {}", e);
                        }
                    });
                } else {
                    error!("No scenario selected for NPY export");
                }
            }
        });
        let Some(image_bundle) = result_images
            .image_bundles
            .get_mut(&selected_image.image_type)
        else {
            error!(
                "Image bundle not found for type: {:?}",
                selected_image.image_type
            );
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
                        if let Err(e) = generate::generate_image(send_scenario, image_type) {
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
