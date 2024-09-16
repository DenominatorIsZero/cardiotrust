use bevy::prelude::*;

use super::options::ColorOptions;
use crate::core::scenario::Scenario;

/// Used for animation. Tracks current sample, max sample, and sample rate.
/// Currently also keeps track of selected sensor.
#[derive(Resource, Debug)]
pub struct SampleTracker {
    pub current_sample: usize,
    pub max_sample: usize,
    pub sample_rate: f32,
    pub manual: bool,
    pub selected_sensor: usize,
    pub selected_beat: usize,
}

impl Default for SampleTracker {
    #[tracing::instrument(level = "debug")]
    fn default() -> Self {
        debug!("Initializing default sample tracker.");
        Self {
            current_sample: 1,
            max_sample: 1,
            sample_rate: 1.0,
            manual: true,
            selected_sensor: 0,
            selected_beat: 0,
        }
    }
}

/// Initializes the sample tracker resource with values from the scenario.
/// Sets the current sample to 0, the max sample to the number of rows in the
/// scenario data, and the sample rate to the rate in the scenario config.
///
/// # Panics
///
/// Panics if the scenario data or config is None.
#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::needless_pass_by_value,
    clippy::module_name_repetitions
)]
#[tracing::instrument(level = "debug")]
pub fn init_sample_tracker(sample_tracker: &mut SampleTracker, scenario: &Scenario) {
    debug!("Initializing sample tracker.");
    sample_tracker.current_sample = 0;
    sample_tracker.max_sample = scenario
        .data
        .as_ref()
        .expect("Data to be some")
        .simulation
        .measurements
        .num_steps();
    sample_tracker.sample_rate = scenario.config.simulation.sample_rate_hz;
}
/// If not in manual mode, calculates a new sample index based on the elapsed
/// time, sample rate, and playback speed. Takes the result modulo the max sample
/// to loop/wrap around.
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::needless_pass_by_value
)]
#[tracing::instrument(level = "trace")]
pub fn update_sample_index(
    mut sample_tracker: ResMut<SampleTracker>,
    time: Res<Time>,
    vis_options: Res<ColorOptions>,
) {
    trace!("Running system to update sample index.");
    if !sample_tracker.manual {
        sample_tracker.current_sample = ((time.elapsed_seconds()
            * sample_tracker.sample_rate
            * vis_options.playbackspeed) as usize)
            % sample_tracker.max_sample;
    }
}
