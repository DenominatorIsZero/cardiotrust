use bevy::prelude::*;

use crate::core::scenario::Scenario;

use super::options::VisOptions;
#[derive(Resource, Debug)]
pub struct SampleTracker {
    pub current_sample: usize,
    pub max_sample: usize,
    pub sample_rate: f32,
}

impl Default for SampleTracker {
    fn default() -> Self {
        Self {
            current_sample: 1,
            max_sample: 1,
            sample_rate: 1.0,
        }
    }
}

/// .
///
/// # Panics
///
/// Panics if Simulation or data is none.
#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::needless_pass_by_value,
    clippy::module_name_repetitions
)]
pub fn init_sample_tracker(sample_tracker: &mut SampleTracker, scenario: &Scenario) {
    sample_tracker.current_sample = 0;
    sample_tracker.max_sample = scenario
        .data
        .as_ref()
        .expect("Data to be some")
        .get_measurements()
        .values
        .shape()[0];
    sample_tracker.sample_rate = scenario
        .config
        .simulation
        .as_ref()
        .expect("Simultaion to be some")
        .sample_rate_hz;
}

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::needless_pass_by_value
)]
pub fn update_sample_index(
    mut sample_tracker: ResMut<SampleTracker>,
    time: Res<Time>,
    vis_options: Res<VisOptions>,
) {
    sample_tracker.current_sample = ((time.elapsed_seconds()
        * sample_tracker.sample_rate
        * vis_options.playbackspeed) as usize)
        % sample_tracker.max_sample;
}
