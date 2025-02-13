use tracing::info;

use crate::core::{
    algorithm::run_epoch, config::algorithm::Algorithm, data::Data,
    model::functional::FunctionalDescription, scenario::results::Results,
};

mod all_pass_optimization;
mod loss_decreases;
mod no_crash;

#[tracing::instrument(level = "info", skip_all)]
fn run(results: &mut Results, data: &Data, algorithm_config: &Algorithm) {
    info!("Running optimization.");
    let mut batch_index = 0;
    for _ in 0..algorithm_config.epochs {
        run_epoch(results, &mut batch_index, data, algorithm_config);
    }
    results
        .estimations
        .system_states_spherical
        .calculate(&results.estimations.system_states);
    results
        .estimations
        .system_states_spherical_max
        .calculate(&results.estimations.system_states_spherical);
}
