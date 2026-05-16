use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use image::ImageEncoder;
use ndarray::s;
use tracing::debug;

use super::{AnimType, GifType, ImageType};
use crate::{
    core::{
        algorithm::metrics::predict_voxeltype,
        model::functional::allpass::shapes::ActivationTimeMs,
        scenario::{Scenario, ScenarioPayload},
    },
    vis::plotting::{
        gif::{
            matrix::matrix_over_slices_plot, states::states_spherical_plot_over_time,
            voxel_type::voxel_types_over_slices_plot,
        },
        png::{
            activation_time::activation_time_plot,
            delay::average_delay_plot,
            line::{standard_log_y_plot, standard_time_plot, standard_y_plot},
            propagation_speed::average_propagation_speed_plot,
            states::states_spherical_plot,
            voxel_type::voxel_type_plot,
        },
        GifBundle, PngBundle, PlotSlice, StateSphericalPlotMode,
    },
};

// ── Image generation ──────────────────────────────────────────────────────────

/// Generates an image for the given scenario and image type.
///
/// Always renders to an in-memory RGBA buffer via `BitMapBackend`.
/// On native (`"native"` feature), the returned PNG is also written to `path`
/// on disk for inter-session caching. The caller receives the in-memory
/// `PngBundle` on all targets so generation → GPU upload can happen in one hop.
#[allow(
    clippy::needless_pass_by_value,
    clippy::too_many_lines,
    clippy::useless_let_if_seq,
    clippy::no_effect_underscore_binding,
    clippy::collection_is_never_read,
    clippy::used_underscore_binding,
    unreachable_code
)]
#[tracing::instrument(level = "debug", skip(payload))]
pub(super) fn generate_image(
    scenario: Scenario,
    payload: ScenarioPayload,
    path: PathBuf,
    image_type: ImageType,
) -> Result<PngBundle> {
    debug!("Generating image");

    #[cfg(feature = "native")]
    let path_arg: Option<&Path> = Some(&path);
    #[cfg(not(feature = "native"))]
    let path_arg: Option<&Path> = None;

    #[cfg(feature = "native")]
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create image directory: {}", parent.display()))?;
    }

    let estimations = &payload.results.estimations;
    let Some(model) = payload.results.model.as_ref() else {
        return Err(anyhow::anyhow!(
            "Model not available in results for image generation"
        ));
    };
    let data = &payload.data;
    let metrics = &payload.results.metrics;

    let bundle = match image_type {
        ImageType::StatesMaxAlgorithm => states_spherical_plot(
            &estimations.system_states_spherical,
            &estimations.system_states_spherical_max,
            &model.spatial_description.voxels.positions_mm,
            model.spatial_description.voxels.size_mm,
            &model.spatial_description.voxels.numbers,
            path_arg,
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
            path_arg,
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
            path_arg,
            None,
            Some(StateSphericalPlotMode::ABS),
            None,
            None,
        ),
        ImageType::ActivationTimeAlgorithm => activation_time_plot(
            &model.functional_description.ap_params.activation_time_ms,
            &model.spatial_description.voxels.positions_mm,
            model.spatial_description.voxels.size_mm,
            path_arg,
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
            path_arg,
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
                path_arg,
                Some(PlotSlice::Z(0)),
            )
        }
        ImageType::VoxelTypesAlgorithm => voxel_type_plot(
            &model.spatial_description.voxels.types,
            &model.spatial_description.voxels.positions_mm,
            model.spatial_description.voxels.size_mm,
            path_arg,
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
            path_arg,
            None,
        ),
        ImageType::VoxelTypesPrediction => voxel_type_plot(
            &predict_voxeltype(
                estimations,
                &data.simulation.model.spatial_description.voxels.types,
                &model.spatial_description.voxels.numbers,
                scenario
                    .summary
                    .ok_or_else(|| {
                        anyhow::anyhow!("Scenario summary not available for voxel type prediction")
                    })?
                    .threshold,
            ),
            &model.spatial_description.voxels.positions_mm,
            model.spatial_description.voxels.size_mm,
            path_arg,
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
            path_arg,
            None,
            None,
        ),
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
            path_arg,
            None,
        ),
        ImageType::AverageDelayAlgorithm => average_delay_plot(
            &estimations.average_delays,
            &model.spatial_description.voxels.numbers,
            &model.spatial_description.voxels.positions_mm,
            model.spatial_description.voxels.size_mm,
            path_arg,
            None,
            None,
        ),
        ImageType::AveragePropagationSpeedAlgorithm => average_propagation_speed_plot(
            &estimations.average_delays,
            &model.spatial_description.voxels.numbers,
            &model.spatial_description.voxels.positions_mm,
            model.spatial_description.voxels.size_mm,
            data.simulation.sample_rate_hz,
            path_arg,
            None,
        ),
        ImageType::AverageDelayDelta => average_delay_plot(
            &(&data.simulation.average_delays - &estimations.average_delays),
            &model.spatial_description.voxels.numbers,
            &model.spatial_description.voxels.positions_mm,
            model.spatial_description.voxels.size_mm,
            path_arg,
            None,
            None,
        ),
        ImageType::LossEpoch => standard_log_y_plot(
            &metrics.loss_batch,
            path_arg,
            "Sum Loss Per Epoch",
            "Loss",
            "Epoch",
        ),
        ImageType::Loss => standard_y_plot(
            &metrics.loss,
            path_arg,
            "Loss Per Step",
            "Loss",
            "Step",
        ),
        ImageType::LossMseEpoch => standard_log_y_plot(
            &metrics.loss_mse_batch,
            path_arg,
            "Sum MSE Loss Per Epoch",
            "Loss",
            "Epoch",
        ),
        ImageType::LossMse => standard_y_plot(
            &metrics.loss_mse,
            path_arg,
            "MSE Loss Per Step",
            "Loss",
            "Step",
        ),
        ImageType::LossMaximumRegularizationEpoch => standard_log_y_plot(
            &metrics.loss_maximum_regularization_batch,
            path_arg,
            "Sum Max. Reg. Loss Per Epoch",
            "Loss",
            "Epoch",
        ),
        ImageType::LossMaximumRegularization => standard_y_plot(
            &metrics.loss_maximum_regularization,
            path_arg,
            "Max. Reg. Loss Per Step",
            "Loss",
            "Step",
        ),
        ImageType::Dice => standard_y_plot(
            &metrics.dice_score_over_threshold,
            path_arg,
            "Dice Score over Threshold",
            "Dice Score",
            "Threshold * 100",
        ),
        ImageType::IoU => standard_y_plot(
            &metrics.iou_over_threshold,
            path_arg,
            "IoU over Threshold",
            "IoU",
            "Threshold * 100",
        ),
        ImageType::Recall => standard_y_plot(
            &metrics.recall_over_threshold,
            path_arg,
            "Recall over Threshold",
            "Recall",
            "Threshold * 100",
        ),
        ImageType::Precision => standard_y_plot(
            &metrics.precision_over_threshold,
            path_arg,
            "Precision over Threshold",
            "Precision",
            "Threshold * 100",
        ),
        ImageType::ControlFunctionAlgorithm => standard_time_plot(
            &model.functional_description.control_function_values,
            scenario.config.simulation.sample_rate_hz,
            path_arg,
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
            path_arg,
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
            path_arg,
            "Control Function Delta",
            "u [A/mm^2]",
        ),
        ImageType::StateAlgorithm => standard_time_plot(
            &estimations.system_states.slice(s![.., 0]).to_owned(),
            scenario.config.simulation.sample_rate_hz,
            path_arg,
            "System State 0 Algorithm",
            "j [A/mm^2]",
        ),
        ImageType::StateSimulation => standard_time_plot(
            &data.simulation.system_states.slice(s![.., 0]).to_owned(),
            scenario.config.simulation.sample_rate_hz,
            path_arg,
            "System State 0 Simulation",
            "j [A/mm^2]",
        ),
        ImageType::StateDelta => standard_time_plot(
            &(&estimations.system_states.slice(s![.., 0]).to_owned()
                - &data.simulation.system_states.slice(s![.., 0]).to_owned()),
            scenario.config.simulation.sample_rate_hz,
            path_arg,
            "System State 0 Delta",
            "j [A/mm^2]",
        ),
        ImageType::MeasurementAlgorithm => standard_time_plot(
            &estimations.measurements.slice(s![0, .., 0]).to_owned(),
            scenario.config.simulation.sample_rate_hz,
            path_arg,
            "Measurement 0 Algorithm",
            "z [pT]",
        ),
        ImageType::MeasurementSimulation => standard_time_plot(
            &data.simulation.measurements.slice(s![0, .., 0]).to_owned(),
            scenario.config.simulation.sample_rate_hz,
            path_arg,
            "Measurement 0 Simulation",
            "z [pT]",
        ),
        ImageType::MeasurementDelta => standard_time_plot(
            &(&estimations.measurements.slice(s![0, .., 0]).to_owned()
                - &data.simulation.measurements.slice(s![0, .., 0]).to_owned()),
            scenario.config.simulation.sample_rate_hz,
            path_arg,
            "Measurement 0 Delta",
            "z [pT]",
        ),
    }
    .with_context(|| format!("Failed to generate plot for image type: {image_type:?}"))?;

    Ok(bundle)
}

// ── Animation (PNG-sequence) generation ───────────────────────────────────────

/// Generates a PNG frame sequence for the given animation type.
///
/// Frames are generated in-memory. On native the frames are also written to
/// `anim_dir/frame_NNNN.png` for inter-session caching. The caller receives a
/// `GifBundle` with all frame data so upload to GPU can happen in one hop.
#[allow(clippy::needless_pass_by_value, clippy::too_many_lines)]
#[tracing::instrument(level = "debug", skip(payload))]
pub(super) fn generate_animation(
    scenario: Scenario,
    payload: ScenarioPayload,
    anim_dir: PathBuf,
    anim_type: AnimType,
) -> Result<GifBundle> {
    debug!("Generating animation frames for {:?}", anim_type);

    let estimations = &payload.results.estimations;
    let Some(model) = payload.results.model.as_ref() else {
        return Err(anyhow::anyhow!(
            "Model not available in results for animation generation"
        ));
    };
    let data = &payload.data;

    let gif_bundle = match anim_type {
        AnimType::StatesAlgorithm => states_spherical_plot_over_time(
            &estimations.system_states_spherical,
            &estimations.system_states_spherical_max,
            &model.spatial_description.voxels.positions_mm,
            model.spatial_description.voxels.size_mm,
            data.simulation.sample_rate_hz,
            &model.spatial_description.voxels.numbers,
            None,
            Some(PlotSlice::Z(0)),
            Some(StateSphericalPlotMode::ABS),
            Some(0.1),
            Some(20),
        )
        .context("Failed to generate StatesAlgorithm animation frames")?,
        AnimType::StatesSimulation => states_spherical_plot_over_time(
            &data.simulation.system_states_spherical,
            &data.simulation.system_states_spherical_max,
            &data
                .simulation
                .model
                .spatial_description
                .voxels
                .positions_mm,
            model.spatial_description.voxels.size_mm,
            data.simulation.sample_rate_hz,
            &model.spatial_description.voxels.numbers,
            None,
            Some(PlotSlice::Z(0)),
            Some(StateSphericalPlotMode::ABS),
            Some(0.1),
            Some(20),
        )
        .context("Failed to generate StatesSimulation animation frames")?,
        AnimType::MatrixOverSlices => {
            let numbers = &model.spatial_description.voxels.numbers;
            let shape = numbers.shape();
            let max_mag = &estimations.system_states_spherical_max.magnitude;
            let mut arr = ndarray::Array3::<f32>::zeros((shape[0], shape[1], shape[2]));
            for ((x, y, z), num) in numbers.indexed_iter() {
                if let Some(n) = num {
                    let voxel_idx = n / 3;
                    if let Some(&val) = max_mag.get(voxel_idx) {
                        arr[(x, y, z)] = val;
                    }
                }
            }
            matrix_over_slices_plot(
                &arr,
                None,
                None,
                None,
                None,
                None,
                Some("Max State Magnitude"),
                Some("y [mm]"),
                Some("x [mm]"),
                Some("au"),
                None,
                None,
                None,
            )
            .context("Failed to generate MatrixOverSlices animation frames")?
        }
        AnimType::VoxelTypesOverSlices => voxel_types_over_slices_plot(
            &model.spatial_description.voxels.types,
            &model.spatial_description.voxels.positions_mm,
            model.spatial_description.voxels.size_mm,
            None,
            None,
            None,
        )
        .context("Failed to generate VoxelTypesOverSlices animation frames")?,
    };

    #[cfg(feature = "native")]
    {
        std::fs::create_dir_all(&anim_dir).with_context(|| {
            format!(
                "Failed to create animation directory: {}",
                anim_dir.display()
            )
        })?;

        for (index, rgb_bytes) in gif_bundle.data.iter().enumerate() {
            let frame_path = anim_dir.join(format!("frame_{index:04}.png"));
            write_rgb_as_png(rgb_bytes, gif_bundle.width, gif_bundle.height, &frame_path)
                .with_context(|| {
                    format!(
                        "Failed to write animation frame {index} to {}",
                        frame_path.display()
                    )
                })?;
        }
    }

    Ok(gif_bundle)
}

/// Writes raw RGB bytes (row-major, 3 bytes per pixel) as a PNG file.
#[tracing::instrument(level = "trace", skip(rgb_bytes))]
fn write_rgb_as_png(rgb_bytes: &[u8], width: u32, height: u32, path: &Path) -> Result<()> {
    let file = std::fs::File::create(path)
        .with_context(|| format!("Failed to create PNG file: {}", path.display()))?;
    let writer = std::io::BufWriter::new(file);
    let encoder = image::codecs::png::PngEncoder::new(writer);
    encoder
        .write_image(rgb_bytes, width, height, image::ExtendedColorType::Rgb8)
        .with_context(|| format!("Failed to encode PNG for: {}", path.display()))?;
    Ok(())
}

/// Detects an existing PNG file at the standard results path for the given
/// scenario and image type. Used for preloading cached images on native.
#[tracing::instrument(level = "trace")]
pub(super) fn detect_existing_image(scenario_id: &str, image_type: ImageType) -> Option<PathBuf> {
    let path = Path::new("results")
        .join(scenario_id)
        .join("img")
        .join(image_type.to_string())
        .with_extension("png");
    path.is_file().then_some(path)
}

/// Discovers existing `frame_NNNN.png` files in a directory.
#[tracing::instrument(level = "debug")]
pub(super) fn detect_existing_frames(dir: &Path) -> Option<Vec<PathBuf>> {
    if !dir.is_dir() {
        return None;
    }

    let entries = std::fs::read_dir(dir).ok()?;
    let mut frames: Vec<PathBuf> = entries
        .filter_map(|e| {
            let e = e.ok()?;
            let name = e.file_name();
            let name = name.to_string_lossy();
            if name.starts_with("frame_") && name.ends_with(".png") {
                Some(e.path())
            } else {
                None
            }
        })
        .collect();
    frames.sort();

    if frames.is_empty() {
        None
    } else {
        Some(frames)
    }
}

// ── Legacy GIF generation (kept for backward compat / tests) ──────────────────

/// Generates animated GIF visualizations of the system states over time from the simulation results.
#[allow(
    clippy::needless_pass_by_value,
    clippy::too_many_lines,
    clippy::useless_let_if_seq
)]
#[tracing::instrument(level = "debug")]
pub(super) fn generate_gifs(
    scenario: Scenario,
    gif_type: GifType,
    playback_speed: f32,
) -> Result<()> {
    let _ = (scenario, playback_speed);
    Err(anyhow::anyhow!(
        "Legacy GIF generation is unsupported after project-aware storage refactor ({gif_type:?})"
    ))
}
