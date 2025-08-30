pub mod config;
pub mod engine;
pub mod geometry;
pub mod input;
pub mod renderer;
pub mod resource;
pub mod scene;
pub mod texture;
pub mod ui;
pub mod utils;

/// WebAssembly (WASM) architecture note:
///
/// We explicitly target `wasm32` instead of `wasm64` because:
///
/// 1. The current WebAssembly specification and all major browsers (Chrome,
///    Firefox, Safari, Edge) only support a 32-bit memory model. Each WASM
///    module can address up to 4 GB of linear memory.
///
/// 2. The Rust toolchain (`rustc`, `wasm-bindgen`, `web-sys`, `wgpu`) provides
///    stable support only for 32-bit targets:
///        - wasm32-unknown-unknown
///        - wasm32-wasi
///        - wasm32-unknown-emscripten
///
/// 3. `wasm64` is experimental and not yet standardized or implemented in
///    production environments.
///
/// Using `#[cfg(target_arch = "wasm32")]` ensures that
/// WASM-specific imports and bindings (e.g., `wasm_bindgen`)
/// are only compiled for WebAssembly builds, keeping native
/// binaries clean and free of unnecessary dependencies.
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use crate::{
        config::Config,
        geometry::{
                mesh::{Mesh, Primitive},
        },
};

#[cfg(not(target_arch = "wasm32"))]
use crate::utils::exit::get_exit_message;

#[cfg(target_arch = "wasm32")]
pub fn get_body_size() -> Option<(u32, u32)>
{
        let window = web_sys::window()?;

        let document = window.document()?;

        let body = document.body()?;

        let width = body.client_width() as u32;

        let height = body.client_height() as u32;

        log::info!("Body: {}, {}", width, height);

        Some((width, height))
}

pub fn run_oxide() -> anyhow::Result<()>
{
        utils::bootstrap::config_logging();

        utils::bootstrap::show_start_message();

        /*
        let mut engine = Engine::new();


        // Load resources
        let tex = engine.load_texture("brick.png");
        let mesh = engine.load_mesh("cube.obj");

        // Create material
        let mat = engine.create_material("pbr_shader")
            .with_texture("albedo", tex);

        // Scene
        engine.draw(mesh, mat, Transform::from_position([0.0, 0.0, -5.0]));

        // Effects
        engine.add_effect(Effect::Bloom { intensity: 1.2 });
        engine.add_effect(Effect::DepthOfField { focus: 0.5 });
        engine.add_effect(Effect::ShadowMapping);

        // Main loop
        engine.run();
        */

        //oxide::run().unwrap();

        let engine = engine::EngineBuilder::new().build()?;

        let mesh_pentagon = Mesh::basic("pentagon", Primitive::Pentagon);

        let mesh_square = Mesh::basic("square", Primitive::Square);

        let mesh_triangle = Mesh::basic("triangle", Primitive::Triangle);

        {
                let mut resources = engine.resources.lock().unwrap_or_else(|e| e.into_inner());

                resources.add_mesh(mesh_pentagon);

                resources.add_mesh(mesh_triangle);

                resources.add_mesh(mesh_square);
        }

        let runner = engine::EngineRunner::new(engine)?;

        runner.run()?;

        #[cfg(not(target_arch = "wasm32"))]
        {
                let msg = get_exit_message();

                log::info!("{msg}");
        }

        Ok(())
}

/// WebAssembly entry point for the engine runtime.
///
/// The browser automatically calls this function when
/// the WebAssembly module is initialized, thanks to the
/// [`wasm_bindgen(start)`] attribute.
///
/// It sets up a panic hook for better error reporting in the browser,
/// then delegates to [`start`] to perform the normal setup and run cycle.
///
/// # Errors
/// Returns a [`JsValue`] if initialization fails, though in practice
/// most errors will already result in a panic being reported to the console.
///
/// # Notes
/// - This function replaces `main` on wasm targets.
/// - It is important that `fn setup() -> EngineRunner` is declared statically
///   in the handler type, since it must be accessible without instance state.
///
/// # Examples
/// ```ignore
/// // No need to call this manually. The browser automatically
/// // invokes `run_wasm` when the wasm module loads.
/// ```
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn run_oxide_wasm() -> Result<(), JsValue>
{
        console_error_panic_hook::set_once();

        let _ = run_oxide();

        Ok(())
}
