use std::thread::JoinHandle;

use bevy::prelude::Resource;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use bevy_egui::{egui, EguiContexts};
use std::collections::HashMap;

#[derive(Default)]
pub struct ImageBundle {
    pub texture: Option<egui::TextureHandle>,
    pub join_handle: Option<JoinHandle<()>>,
}

#[derive(EnumIter, Debug, PartialEq, Eq, Hash)]
pub enum ImageType {
    // 2D-Slices
    StatesMaxAlgorithm,
    StatesMaxSimulation,
    StatesMaxDelta,
    ActivationTime,
    VoxelTypes,
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

impl Default for ResultImages {
    fn default() -> Self {
        let mut image_bundles = HashMap::new();

        ImageType::iter().for_each(|image_type| {
            image_bundles
                .insert(image_type, ImageBundle::default())
                .unwrap();
        });

        Self { image_bundles }
    }
}

#[allow(clippy::module_name_repetitions)]
pub fn draw_ui_results(mut contexts: EguiContexts) {
    egui::CentralPanel::default().show(contexts.ctx_mut(), |ui| {
        ui.label("Results");
    });
}
