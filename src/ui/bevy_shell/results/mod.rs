//! Results gallery view — Bevy-native plugin.
//!
//! Implements the supported Bevy-native gallery UI
//! that displays generated plots and animations for a completed scenario.
//!
//! # Module layout
//!
//! ```text
//! results/
//!   mod.rs       — ResultsViewPlugin, all shared types and resources
//!   generate.rs  — Image and animation generation functions
//!   gallery.rs   — Gallery shell spawn/despawn
//!   card.rs      — Card component definitions and spawning
//!   sync.rs      — Card state sync systems
//!   animation.rs — Animation playback systems
//!   polling.rs   — Polling systems for async generation/loading
//!   batch.rs     — Batch generation systems
//!   modal.rs     — Modal overlay systems
//!   export.rs    — Export systems
//! ```

pub mod animation;
pub mod batch;
pub mod card;
pub mod export;
pub mod gallery;
pub mod generate;
pub mod modal;
pub mod polling;
pub mod sync;

use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use bevy::prelude::*;
use strum_macros::{Display, EnumIter};

pub use self::gallery::{despawn_results_view, spawn_results_view};
use crate::{
    ui::UiState,
    vis::plotting::{GifBundle, PngBundle},
    ActiveLoadedScenario,
};

// ── Enums ─────────────────────────────────────────────────────────────────────

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

/// The four animation types available in the gallery.
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Display)]
pub enum AnimType {
    StatesAlgorithm,
    StatesSimulation,
    MatrixOverSlices,
    VoxelTypesOverSlices,
}

impl AnimType {
    /// Returns the subdirectory name used for storing frames on disk.
    #[tracing::instrument(level = "trace")]
    pub(crate) fn dir_name(self) -> &'static str {
        match self {
            Self::StatesAlgorithm => "StatesAlgorithm",
            Self::StatesSimulation => "StatesSimulation",
            Self::MatrixOverSlices => "MatrixOverSlices",
            Self::VoxelTypesOverSlices => "VoxelTypesOverSlices",
        }
    }
}

/// Legacy GIF type (retained for backward compat / tests).
#[derive(EnumIter, Debug, PartialEq, Eq, Hash, Display, Clone, Copy)]
pub enum GifType {
    StatesAlgorithm,
    StatesSimulation,
}

/// The four gallery tabs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GalleryTab {
    #[default]
    SpatialMaps,
    Metrics,
    Losses,
    TimeFunctions,
}

/// What the modal is currently showing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModalTarget {
    StaticImage(ImageType),
    Animation(AnimType),
}

// ── Async channel type ────────────────────────────────────────────────────────

/// Shared result channel written by a background thread.
pub type AsyncChannel<T> = Arc<Mutex<Option<anyhow::Result<T>>>>;

/// Creates a new empty async channel.
#[tracing::instrument(level = "trace")]
pub(crate) fn new_channel<T>() -> AsyncChannel<T> {
    Arc::new(Mutex::new(None))
}

// ── State machine types ────────────────────────────────────────────────────────

/// State of a single static image in the result cache.
#[derive(Debug)]
pub enum ResultImageState {
    /// Not yet requested.
    Pending,
    /// Background thread is generating the plot in memory.
    Generating { channel: AsyncChannel<PngBundle> },
    /// Loading pre-existing cached image bytes from disk (native only).
    Loading {
        channel: AsyncChannel<(Vec<u8>, u32, u32)>,
    },
    /// Image is uploaded to the GPU and ready to display.
    Ready(Handle<Image>),
    /// Generation or loading failed.
    Failed(String),
}

/// State of a single animation in the result cache.
#[derive(Debug)]
pub enum AnimState {
    Pending,
    Generating {
        channel: AsyncChannel<GifBundle>,
    },
    /// Loading pre-existing cached frames from disk (native only).
    Loading {
        channels: Vec<AsyncChannel<(Vec<u8>, u32, u32)>>,
        loaded: Vec<Option<Handle<Image>>>,
    },
    Ready(AnimPlaybackState),
    Failed(String),
}

/// Per-animation playback state.
#[derive(Debug)]
pub struct AnimPlaybackState {
    pub frames: Vec<Handle<Image>>,
    pub current_frame: usize,
    pub playing: bool,
    pub timer: Timer,
}

/// State of an async export job.
#[derive(Debug)]
pub enum ExportState {
    InProgress(AsyncChannel<PathBuf>),
    Done(PathBuf),
    Failed(String),
}

// ── Resources ─────────────────────────────────────────────────────────────────

/// Cache of all static result image states.
#[derive(Resource, Default, Debug)]
pub struct ResultImageCache(pub HashMap<ImageType, ResultImageState>);

/// Cache of all animation states.
#[derive(Resource, Default, Debug)]
pub struct ResultAnimCache(pub HashMap<AnimType, AnimState>);

/// Top-level view state for the results gallery.
#[derive(Resource, Debug)]
pub struct ResultsViewState {
    pub active_tab: GalleryTab,
    pub modal: Option<ModalTarget>,
    pub batch_total: usize,
    pub batch_done: usize,
    pub playback_speed: f32,
    pub export_state: Option<ExportState>,
    pub modal_anim_state: Option<AnimPlaybackState>,
}

impl Default for ResultsViewState {
    #[tracing::instrument(level = "trace")]
    fn default() -> Self {
        Self {
            active_tab: GalleryTab::default(),
            modal: None,
            batch_total: 0,
            batch_done: 0,
            playback_speed: 10.0,
            export_state: None,
            modal_anim_state: None,
        }
    }
}

// ── Marker components ─────────────────────────────────────────────────────────

/// Marker for the root entity of the entire Results view.
#[derive(Component, Debug)]
pub struct ResultsViewRoot;

// ── Plugin ────────────────────────────────────────────────────────────────────

/// Plugin that owns the Bevy-native Results gallery view.
#[derive(Debug)]
pub struct ResultsViewPlugin;

impl Plugin for ResultsViewPlugin {
    #[tracing::instrument(level = "info", skip(app))]
    fn build(&self, app: &mut App) {
        // Resources
        app.init_resource::<ResultImageCache>();
        app.init_resource::<ResultAnimCache>();
        app.init_resource::<ResultsViewState>();
        app.init_resource::<ActiveLoadedScenario>();

        // Spawn / despawn
        app.add_systems(OnEnter(UiState::Results), spawn_results_view)
            .add_systems(
                OnExit(UiState::Results),
                (despawn_results_view, reset_result_caches),
            );

        let results_condition = in_state(UiState::Results);

        // Tab bar
        app.add_systems(
            Update,
            (
                gallery::handle_gallery_tab_click,
                gallery::update_gallery_tab_visuals,
                gallery::update_gallery_grid_columns,
            )
                .run_if(results_condition.clone()),
        );

        // Card sync
        app.add_systems(
            Update,
            (
                sync::sync_static_card_state,
                sync::sync_anim_card_state,
                sync::update_card_hover,
                sync::handle_generate_button_static,
                sync::handle_generate_button_anim,
                sync::handle_retry_button,
            )
                .run_if(results_condition.clone()),
        );

        // Polling
        app.add_systems(
            Update,
            (
                polling::poll_image_generation,
                polling::poll_image_loading,
                polling::poll_anim_generation,
                polling::poll_anim_loading,
            )
                .run_if(results_condition.clone()),
        );

        // Animation playback
        app.add_systems(
            Update,
            (
                animation::tick_animation_playback,
                animation::handle_play_pause_button,
                animation::handle_frame_increment,
                animation::handle_frame_decrement,
            )
                .run_if(results_condition.clone()),
        );

        // Batch generation
        app.add_systems(
            Update,
            (
                batch::handle_generate_all_in_tab,
                batch::handle_generate_all,
                batch::update_batch_progress,
            )
                .run_if(results_condition.clone()),
        );

        // Modal
        app.add_systems(
            Update,
            (
                modal::handle_static_card_click,
                modal::handle_anim_card_click,
                modal::update_static_modal,
                modal::handle_modal_close,
                modal::handle_modal_nav,
                modal::handle_modal_keyboard,
            )
                .run_if(results_condition.clone()),
        );

        // Export
        app.add_systems(
            Update,
            (
                export::handle_save_button,
                export::handle_export_npy,
                export::handle_export_apng,
                export::handle_export_mp4,
                export::update_export_buttons,
                export::poll_export_state,
            )
                .run_if(results_condition),
        );
    }
}

// ── Systems ───────────────────────────────────────────────────────────────────

/// Resets result caches when leaving the Results state.
#[tracing::instrument(skip_all)]
pub fn reset_result_caches(
    mut image_cache: ResMut<ResultImageCache>,
    mut anim_cache: ResMut<ResultAnimCache>,
    mut view_state: ResMut<ResultsViewState>,
    mut active_loaded_scenario: ResMut<ActiveLoadedScenario>,
) {
    *image_cache = ResultImageCache::default();
    *anim_cache = ResultAnimCache::default();
    *view_state = ResultsViewState::default();
    active_loaded_scenario.0 = None;
}
