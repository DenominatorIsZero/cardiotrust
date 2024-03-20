use tracing::trace;

use crate::core::{data::shapes::ArraySystemStates, model::spatial::voxels::Voxels};

/// .
///
/// # Panics
///
/// Panics if something fishy happens with io rights.
#[tracing::instrument(level = "trace")]
pub fn plot_states_over_time(
    system_states: &ArraySystemStates,
    voxels: &Voxels,
    fps: u32,
    playback_speed: f32,
    file_name: &str,
    title: &str,
) {
    trace!("Plotting states over time");
    // let directory = format!("./tmp/{file_name}/");
    // let dir_path = Path::new(&directory);
    // if dir_path.is_dir() {
    //     fs::remove_dir_all(dir_path).expect("Could not delete temporary directory");
    // }
    // fs::create_dir_all(dir_path).expect("Could not create temporary directory.");

    // let sample_number = system_states.values.shape()[0];
    // #[allow(
    //     clippy::cast_precision_loss,
    //     clippy::cast_sign_loss,
    //     clippy::cast_possible_truncation
    // )]
    // let image_number = (fps as f32 / playback_speed) as usize;
    // let time_step = sample_number / image_number;

    // let min_j_init = *system_states.values.min_skipnan();
    // let max_j_init = *system_states.values.max_skipnan(); // TODO: This should really be over the absolute values...

    // let time_indices: Vec<usize> = (0..sample_number).step_by(time_step).collect();
    // let mut image_names = Vec::new();

    // for (image_index, time_index) in time_indices.into_iter().enumerate() {
    //     let image_name = format!("./tmp/{file_name}/{image_index}");
    //     plot_states_at_time(
    //         system_states,
    //         voxels,
    //         min_j_init,
    //         max_j_init,
    //         time_index,
    //         &image_name,
    //         title,
    //     );
    //     image_names.push(format!("{image_name}.png"));
    // }

    todo!()
    // let images = engiffen::load_images(image_names.as_slice());
    // let gif = engiffen::engiffen(&images, fps as usize, engiffen::Quantizer::Naive)
    //     .expect("Could not create gif from images.");
    // let mut output_file =
    //     File::create(format!("{file_name}.gif")).expect("Could not create gif file.");
    // gif.write(&mut output_file)
    //     .expect("Could not write gif file.");
    // fs::remove_dir_all(dir_path).expect("Could not remove temporary folders.");
}
