pub mod app;
pub mod config;
pub mod state;

use std::process;

/// WebAssembly (WASM) architecture note:
///
/// We explicitly target `wasm32` instead of `wasm64` because:
///
/// 1. The current WebAssembly specification and all major browsers
///    (Chrome, Firefox, Safari, Edge) only support a 32-bit memory model.
///    Each WASM module can address up to 4 GB of linear memory.
///
/// 2. The Rust toolchain (`rustc`, `wasm-bindgen`, `web-sys`, `wgpu`)
///    provides stable support only for 32-bit targets:
///        - wasm32-unknown-unknown
///        - wasm32-wasi
///        - wasm32-unknown-emscripten
///
/// 3. `wasm64` is experimental and not yet standardized or implemented
///    in production environments.
///
/// Using `#[cfg(target_arch = "wasm32")]` ensures that
/// WASM-specific imports and bindings (e.g., `wasm_bindgen`)
/// are only compiled for WebAssembly builds, keeping native
/// binaries clean and free of unnecessary dependencies.
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use crate::{app::App, config::Config};
use winit::event_loop::EventLoop;

/// Starts the application in native or WASM environments.
///
/// # Returns
/// - `Ok(())` if the application exits successfully.
/// - An error if initialization fails.
///
pub fn run() -> anyhow::Result<()>
{
        #[cfg(not(target_arch = "wasm32"))]
        env_logger::init();

        #[cfg(target_arch = "wasm32")]
        console_log::init_with_level(log::Level::Info).unwrap_throw();

        let event_loop = EventLoop::with_user_event().build()?;

        let config = Config::from_file().unwrap_or_else(|err| {
                                                log::info!("ERRAH! {err}");
                                                Config::default()
                                        });

        let mut app = App::new(config,
                               #[cfg(target_arch = "wasm32")]
                               &event_loop);

        event_loop.run_app(&mut app)?;

        Ok(())
}

/// WASM entrypoint function.
///
/// This function is called automatically by the JavaScript glue code
/// when the WASM module is loaded in the browser.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn run_web() -> Result<(), wasm_bindgen::JsValue>
{
        console_error_panic_hook::set_once();

        run().unwrap_throw();

        Ok(())
}
