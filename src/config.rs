#[derive(Debug)]
pub struct Config
{
        #[cfg(target_arch = "wasm32")]
        DEFAULT_CANVAS_WIDTH: u32 = 800,

        #[cfg(target_arch = "wasm32")]
        DEFAULT_CANVAS_HEIGHT: u32 = 600,
}
