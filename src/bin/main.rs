#[cfg(feature = "native")]
use std::process::Command;

use anyhow::{Context, Result};
use bevy::{log::LogPlugin, prelude::*};
use cardiotrust::{
    scheduler::SchedulerPlugin, ui::UiPlugin, vis::VisPlugin, ActiveLoadedScenario,
    PendingProjectLoad, ProjectState, ScenarioList, SelectedSenario,
};
use tracing::{info, warn};
use tracing_subscriber::{fmt, layer::SubscriberExt};

#[tracing::instrument(level = "info")]
fn main() {
    if let Err(e) = run_app() {
        eprintln!("Application failed to start: {e}");
        std::process::exit(1);
    }
}

#[tracing::instrument(level = "info")]
fn run_app() -> Result<()> {
    // Set up logging with graceful fallback
    setup_logging()?;

    // Get git hash with fallback to "unknown"
    let git_hash = get_git_hash();

    info!("Starting CardioTRust application. Git hash: {}", git_hash);

    let mut app = App::new();

    #[cfg(not(feature = "native"))]
    {
        // On WASM, load embedded demo projects at startup
        match ScenarioList::load_from_embedded() {
            Ok(demo_list) => {
                app.insert_resource(demo_list);
            }
            Err(e) => {
                warn!("Failed to load embedded demo projects: {e}");
                app.insert_resource(ScenarioList::empty());
            }
        }
    }
    #[cfg(feature = "native")]
    {
        app.insert_resource(ScenarioList::empty());
    }

    // ── WASM: embedded asset I/O ───────────────────────────────────────────
    // Register compile-time-embedded 3D assets before the AssetPlugin runs,
    // so `AssetPlugin::init_default_source` sees our source and skips the
    // filesystem-backed default.
    #[cfg(not(feature = "native"))]
    {
        use bevy::asset::io::{
            memory::{Dir, MemoryAssetReader},
            AssetSourceBuilder, AssetSourceBuilders, AssetSourceId,
        };
        use std::path::Path;

        let dir = Dir::default();
        macro_rules! embed {
            ($path:expr) => {
                dir.insert_asset(
                    Path::new($path),
                    include_bytes!(concat!("../../assets/", $path)).as_ref(),
                )
            };
        }
        embed!("bed.glb");
        embed!("room.glb");
        embed!("torso.glb");
        embed!("RoundArrow.obj");
        embed!("RoundArrow.mtl");
        embed!("sensor_array.glb");

        let mut sources = AssetSourceBuilders::default();
        let builder = AssetSourceBuilder::new(move || Box::new(MemoryAssetReader { root: dir.clone() }));
        sources.insert(AssetSourceId::Default, builder);
        app.insert_resource(sources);
    }

    app.init_resource::<SelectedSenario>()
        .init_resource::<PendingProjectLoad>()
        .init_resource::<ActiveLoadedScenario>()
        .insert_resource(ProjectState {
            recent: ProjectState::load_recent(),
        })
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Cardio TRust".into(),
                        ..default()
                    }),
                    ..default()
                })
                .disable::<LogPlugin>(),
        )
        .add_plugins(UiPlugin)
        .add_plugins(SchedulerPlugin)
        .add_plugins(VisPlugin)
        .run();

    Ok(())
}

#[tracing::instrument(level = "debug")]
fn setup_logging() -> Result<()> {
    // Try to set up file logging, fall back to stdout-only if it fails
    #[cfg(feature = "native")]
    {
        if let Err(e) = try_setup_file_logging() {
            eprintln!("Warning: Could not set up file logging ({e}), using stdout only");
            setup_stdout_logging()?;
        }
    }
    #[cfg(not(feature = "native"))]
    {
        #[cfg(target_arch = "wasm32")]
        {
            tracing_wasm::set_as_global_default();
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            setup_stdout_logging()?;
        }
    }

    Ok(())
}

#[tracing::instrument(level = "debug")]
fn setup_stdout_logging() -> Result<()> {
    let subscriber = tracing_subscriber::registry().with(
        fmt::Layer::new()
            .with_writer(std::io::stdout)
            .with_thread_names(true)
            .with_ansi(true),
    );

    tracing::subscriber::set_global_default(subscriber)
        .context("Failed to set up stdout logging")?;

    Ok(())
}

#[cfg(feature = "native")]
#[tracing::instrument(level = "debug")]
fn try_setup_file_logging() -> Result<()> {
    let file_appender = tracing_appender::rolling::daily("./logs", "CardioTRust.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    // Store the guard to prevent it from being dropped
    std::mem::forget(_guard);

    let subscriber = tracing_subscriber::registry()
        .with(
            fmt::Layer::new()
                .with_writer(std::io::stdout)
                .with_thread_names(true)
                .with_ansi(true),
        )
        .with(
            fmt::Layer::new()
                .with_writer(non_blocking)
                .with_thread_names(true)
                .with_line_number(true)
                .fmt_fields(fmt::format::PrettyFields::new())
                .with_ansi(false),
        );

    tracing::subscriber::set_global_default(subscriber).context("Failed to set up file logging")?;

    Ok(())
}

#[cfg(feature = "native")]
#[tracing::instrument(level = "debug")]
fn get_git_hash() -> String {
    Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                String::from_utf8(output.stdout).ok()
            } else {
                None
            }
        })
        .map(|hash| hash.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

#[cfg(not(feature = "native"))]
#[tracing::instrument(level = "debug")]
fn get_git_hash() -> String {
    option_env!("GIT_HASH").unwrap_or("unknown").to_string()
}
