pub mod activation_time;
pub mod delay;
pub mod line;
pub mod matrix;
pub mod propagation_speed;
pub mod states;
pub mod voxel_type;

#[allow(clippy::module_name_repetitions)]
pub struct PngBundle {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
}
