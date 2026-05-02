pub mod bevy_shell;
pub mod colors;

use bevy::prelude::*;
use bevy_editor_cam::prelude::{EditorCam, EnabledMotion};

use self::bevy_shell::BevyShellPlugin;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub struct UiPlugin;

impl Plugin for UiPlugin {
    #[tracing::instrument(level = "info", skip(app))]
    fn build(&self, app: &mut App) {
        info!("Initializing UI plugin.");
        app.init_state::<UiState>()
            .init_resource::<SidebarState>()
            .add_plugins(BevyShellPlugin)
            .add_systems(Update, enable_camera_motion);
    }
}

/// The different content views of the application.
#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash)]
#[allow(clippy::module_name_repetitions)]
pub enum UiState {
    Home,
    Explorer,
    Scenario,
    Results,
    Volumetric,
    Scheduler,
}

impl Default for UiState {
    #[tracing::instrument(level = "trace")]
    fn default() -> Self {
        Self::Home
    }
}

/// Tracks the sidebar expanded/collapsed state and current width.
///
/// When expanded the sidebar is 200 px wide (icon + label). When collapsed it
/// is 56 px wide (icon only). The `user_expanded` field remembers the user's
/// preference so auto-collapse can restore it when the viewport widens.
#[derive(Resource, Debug)]
pub struct SidebarState {
    pub expanded: bool,
    pub width: f32,
    /// The user's explicit preference (before auto-collapse overrides it).
    pub user_expanded: bool,
}

impl Default for SidebarState {
    #[tracing::instrument(level = "trace")]
    fn default() -> Self {
        Self {
            expanded: true,
            width: 200.0,
            user_expanded: true,
        }
    }
}

#[tracing::instrument(skip_all, level = "trace")]
pub fn enable_camera_motion(mut cameras: Query<&mut EditorCam, With<Camera>>) {
    for mut camera in &mut cameras {
        camera.enabled_motion = EnabledMotion {
            pan: true,
            orbit: true,
            zoom: true,
        };
    }
}
