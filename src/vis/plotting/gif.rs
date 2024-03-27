pub mod matrix;
pub mod states;

const DEFAULT_PLAYBACK_SPEED: f32 = 0.1;
const DEFAULT_FPS: u32 = 10;
const DEFAULT_TIME_PER_FRAME_MS: u32 = 500;

#[allow(clippy::module_name_repetitions)]
pub struct GifBundle {
    pub data: Vec<Vec<u8>>,
    pub width: u32,
    pub height: u32,
    pub fps: u32,
}
