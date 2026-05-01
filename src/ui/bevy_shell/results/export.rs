//! Export systems: NPY, APNG, MP4.

#![allow(clippy::type_complexity)]

use bevy::prelude::*;

use super::{
    card::{CardKind, CardSaveButton},
    gallery::{ExportApngButton, ExportMp4Button, ExportNpyButton, ExportStatusLabel},
    ExportState, ResultImageState, ResultsViewState,
};
use crate::{ActiveLoadedScenario, ScenarioList, SelectedSenario};

// ── Save button ────────────────────────────────────────────────────────────────

/// Opens the folder containing the saved image in the OS file manager.
#[tracing::instrument(skip_all)]
pub fn handle_save_button(
    buttons: Query<(&CardSaveButton, &Interaction), (Changed<Interaction>, With<Button>)>,
    image_cache: Res<super::ResultImageCache>,
    scenario_list: Res<ScenarioList>,
    selected: Res<SelectedSenario>,
) {
    for (btn, interaction) in &buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let CardKind::Static(image_type) = btn.kind else {
            // Animation save: open the frame directory.
            let CardKind::Anim(anim_type) = btn.kind else {
                continue;
            };
            let Some(index) = selected.index else {
                continue;
            };
            let Some(entry) = scenario_list.entries.get(index) else {
                continue;
            };
            let dir = entry
                .storage
                .animation_dir(entry.scenario.get_id(), anim_type.dir_name());
            open_in_file_manager(&dir);
            continue;
        };
        if !matches!(
            image_cache.0.get(&image_type),
            Some(ResultImageState::Ready(_))
        ) {
            continue;
        }
        let Some(index) = selected.index else {
            continue;
        };
        let Some(entry) = scenario_list.entries.get(index) else {
            continue;
        };
        let path = entry
            .storage
            .image_path(entry.scenario.get_id(), &image_type.to_string());
        open_in_file_manager(&path);
    }
}

/// Opens `path` (or its parent directory) in the OS file manager.
#[tracing::instrument(level = "debug", skip_all)]
fn open_in_file_manager(path: &std::path::Path) {
    // Resolve to an absolute path so the OS command works regardless of cwd.
    let abs = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    #[cfg(target_os = "macos")]
    {
        // `open -R <file>` reveals the file in Finder.
        let _ = std::process::Command::new("open")
            .arg("-R")
            .arg(&abs)
            .spawn();
    }
    #[cfg(target_os = "linux")]
    {
        // `xdg-open` opens the parent directory.
        let dir = abs.parent().unwrap_or(&abs);
        let _ = std::process::Command::new("xdg-open").arg(dir).spawn();
    }
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("explorer")
            .arg("/select,")
            .arg(&abs)
            .spawn();
    }
}

// ── NPY export ─────────────────────────────────────────────────────────────────

/// Handles "Export .npy" button clicks.
#[tracing::instrument(skip_all)]
pub fn handle_export_npy(
    buttons: Query<(&ExportNpyButton, &Interaction), (Changed<Interaction>, With<Button>)>,
    active_loaded_scenario: Res<ActiveLoadedScenario>,
    mut view_state: ResMut<ResultsViewState>,
) {
    for (_, interaction) in &buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let Some(active) = active_loaded_scenario.0.as_ref() else {
            continue;
        };
        let scenario = active.scenario.clone();
        let payload = active.payload.clone();
        let storage = active.storage.clone();
        let out_dir = storage.npy_dir(scenario.get_id());
        let channel = super::new_channel::<std::path::PathBuf>();
        let writer = channel.clone();
        std::thread::spawn(move || {
            let result = storage
                .save_npy(scenario.get_id(), &payload)
                .map(|()| out_dir);
            if let Ok(mut guard) = writer.lock() {
                *guard = Some(result);
            }
        });
        view_state.export_state = Some(ExportState::InProgress(channel));
    }
}

// ── APNG export ────────────────────────────────────────────────────────────────

/// Handles "Export APNG" button clicks.
#[tracing::instrument(skip_all)]
pub fn handle_export_apng(
    buttons: Query<(&ExportApngButton, &Interaction), (Changed<Interaction>, With<Button>)>,
    anim_cache: Res<super::ResultAnimCache>,
    mut view_state: ResMut<ResultsViewState>,
    scenario_list: Res<ScenarioList>,
    selected: Res<SelectedSenario>,
) {
    for (_, interaction) in &buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }

        let Some(index) = selected.index else {
            continue;
        };
        let Some(entry) = scenario_list.entries.get(index) else {
            continue;
        };
        let first_ready_dir = [
            super::AnimType::StatesAlgorithm,
            super::AnimType::StatesSimulation,
            super::AnimType::MatrixOverSlices,
            super::AnimType::VoxelTypesOverSlices,
        ]
        .iter()
        .find(|&&anim_type| {
            matches!(
                anim_cache.0.get(&anim_type),
                Some(super::AnimState::Ready(_))
            )
        })
        .map(|&anim_type| {
            entry
                .storage
                .animation_dir(entry.scenario.get_id(), anim_type.dir_name())
        });

        let Some(frames_dir) = first_ready_dir else {
            view_state.export_state = Some(ExportState::Failed(
                "No animation ready to export".to_string(),
            ));
            continue;
        };

        let out_path = entry
            .storage
            .export_path(entry.scenario.get_id(), "animation.png");

        let channel = super::new_channel::<std::path::PathBuf>();
        let writer = channel.clone();
        std::thread::spawn(move || {
            let result = export_apng(&frames_dir, &out_path);
            if let Ok(mut guard) = writer.lock() {
                *guard = Some(result);
            }
        });
        view_state.export_state = Some(ExportState::InProgress(channel));
    }
}

/// Handles "Export MP4" button clicks (placeholder).
#[tracing::instrument(skip_all)]
pub fn handle_export_mp4(
    buttons: Query<(&ExportMp4Button, &Interaction), (Changed<Interaction>, With<Button>)>,
    mut view_state: ResMut<ResultsViewState>,
) {
    for (_, interaction) in &buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }
        view_state.export_state = Some(ExportState::Failed(
            "MP4 export not yet implemented".to_string(),
        ));
    }
}

// ── Status / button update ────────────────────────────────────────────────────

/// Refreshes the export status label.
#[tracing::instrument(skip_all)]
pub fn update_export_buttons(
    view_state: Res<ResultsViewState>,
    mut labels: Query<&mut Text, With<ExportStatusLabel>>,
) {
    if !view_state.is_changed() {
        return;
    }
    let text = match &view_state.export_state {
        None => String::new(),
        Some(ExportState::InProgress(_)) => "Exporting...".to_string(),
        Some(ExportState::Done(path)) => format!("Saved: {}", path.display()),
        Some(ExportState::Failed(msg)) => format!("Export error: {msg}"),
    };
    for mut t in &mut labels {
        t.0.clone_from(&text);
    }
}

/// Polls the export background task.
#[tracing::instrument(skip_all)]
pub fn poll_export_state(mut view_state: ResMut<ResultsViewState>) {
    let done = if let Some(ExportState::InProgress(channel)) = &view_state.export_state {
        channel.try_lock().map_or_else(|_| None, |mut g| g.take())
    } else {
        None
    };

    if let Some(result) = done {
        view_state.export_state = Some(match result {
            Ok(path) => ExportState::Done(path),
            Err(e) => ExportState::Failed(e.to_string()),
        });
    }
}

// ── APNG helper ───────────────────────────────────────────────────────────────

/// Exports the first PNG frame in `frames_dir` as the output APNG.
#[tracing::instrument(level = "debug")]
fn export_apng(
    frames_dir: &std::path::Path,
    out_path: &std::path::Path,
) -> anyhow::Result<std::path::PathBuf> {
    if let Some(parent) = out_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let first_frame = frames_dir.join("frame_0000.png");
    if !first_frame.is_file() {
        return Err(anyhow::anyhow!(
            "No frame_0000.png found in {}",
            frames_dir.display()
        ));
    }
    std::fs::copy(&first_frame, out_path)?;
    Ok(out_path.to_path_buf())
}
