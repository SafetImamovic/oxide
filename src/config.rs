#[derive(Debug)]
pub struct Config
{
        #[cfg(target_arch = "wasm32")]
        pub default_canvas_width: u32,

        #[cfg(target_arch = "wasm32")]
        pub default_canvas_height: u32,
}


impl Default for Config
{
        fn default() -> Self
        {
                Self { #[cfg(target_arch = "wasm32")]
                       default_canvas_width: 1280,
                       #[cfg(target_arch = "wasm32")]
                       default_canvas_height: 720 }
        }
}
