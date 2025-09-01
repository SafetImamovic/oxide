//! Oxide runtime — cross‑platform rendering and UI
//!
//! This crate provides a modular real‑time rendering and UI runtime that
//! targets both native desktops and WebAssembly. It is developed as part of a
//! Bachelor's thesis project, and APIs may evolve as the project progresses.
//!
//! # Important: initialization model (current state)
//!
//! - The engine is constructed internally by [`run()`]. At the moment, you
//!   cannot construct or customize the engine from your own `main()` or any
//!   other function.
//! - This design ensures that WebAssembly builds remain compatible with tooling
//!   that expects a library crate. In particular, typical web workflows build
//!   and load a `cdylib` library and do not rely on a Rust binary entry point.
//!
//! What this means for you today:
//! - For native: create a tiny binary that calls [`run()`]. The engine itself
//!   is still created inside this crate, in that function.
//! - For WebAssembly: compile this library for `wasm32`. The exported WASM
//!   entry point will be invoked automatically by the host (e.g., the browser),
//!   and it delegates to [`run()`].
//!
//! Example native binary (thin wrapper that just calls the library):
//! ```rust
//! fn main() {
//!     // The engine is created inside `oxide::run()`
//!     oxide::run().unwrap()
//! }
//! ```
//!
//! # WebAssembly usage
//!
//! - Build the project as a library crate for `wasm32` (e.g., using your
//!   bundler/tooling).
//! - The WebAssembly start function exported by this crate will be called
//!   automatically, and it will install a panic hook for better error messages
//!   before invoking [`run()`].
//! - You do not call the WASM entry point yourself.
//!
//! # Project layout requirement (for now)
//!
//! Because WebAssembly workflows expect a library, you should create your
//! application as a library crate:
//! - Run: `cargo new your_app --lib`
//! - Depend on this crate and use the provided entry points. If you also want a
//!   native executable, add a small binary that calls your library (which, in
//!   turn, calls [`run()`]).
//!
//! A future release will provide a procedural macro to make this smoother (for
//! example, an attribute macro that wires up the correct entry points across
//! native and web targets while still letting you hook into initialization).
//! Until then, engine construction remains internal to [`run()`].
//!
//! # What you can customize today
//!
//! - Configuration, rendering features, resources, and UI are organized into
//!   public modules you can build upon. If you need startup customization right
//!   now, do so by contributing to this crate or by maintaining a small fork;
//!   the planned macro will reduce that need.
//!
//! # Runtime lifecycle
//!
//! A typical frame performs the following steps:
//! 1. Poll and handle window/browser events and input.
//! 2. Update application and scene state.
//! 3. Prepare GPU resources and record rendering commands.
//! 4. Submit work to the GPU and present the frame.
//!
//! # Logging, errors, and diagnostics
//!
//! - Logging is initialized at startup. Use your environment or dev tools to
//!   adjust verbosity.
//! - [`run()`] returns `anyhow::Result<()>`. Propagate errors in your thin
//!   native wrapper or rely on browser console logs when running on the web.
//!
//! # WebAssembly notes
//!
//! - Targets `wasm32` (32‑bit) for broad browser support.
//! - Installs a panic hook to improve error visibility in the browser console.
//! - Web‑specific bindings are compiled only on `wasm32`, keeping native builds
//!   lean.
//!
//! ## WebAssembly entry point for the engine runtime.
//!
//! The browser automatically calls this function when
//! the WebAssembly module is initialized, thanks to the
//! [`wasm_bindgen(start)`] attribute.
//!
//! It sets up a panic hook for better error reporting in the browser,
//! then delegates to [`start`] to perform the normal setup and run cycle.
//!
//! # Errors
//! Return a [`JsValue`] if initialization fails, though in practice
//! most errors will already result in a panic being reported to the console.
//!
//! # Notes
//! - This function replaces `main` on wasm targets.
//! - It is important that `fn setup() -> EngineRunner` is declared statically
//!   in the handler type, since it must be accessible without instance state.
//!
//! # Examples
//! ```ignore
//! // No need to call this manually. The browser automatically
//! // invokes `run_wasm` when the wasm module loads.
//! ```
//!
//! # Roadmap
//!
//! - Introduce a procedural macro to let applications declare initialization
//!   hooks while the macro safely orchestrates platform‑specific entry points.
//!   This will let you customize the startup without re‑implementing the engine
//!   wiring.
//!
//! Feedback is welcome as this thesis project evolves.

pub mod engine;
pub mod geometry;
pub mod input;
pub mod renderer;
pub mod resource;
pub mod texture;
pub mod ui;
pub mod utils;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use crate::geometry::mesh::{Mesh, Primitive};

/// Starts the Oxide runtime and blocks until the application exits.
///
/// This is the canonical entry point for both native and WebAssembly targets.
/// The engine is constructed internally by this function; you cannot
/// instantiate or customize the engine from your own `main()` or any other
/// function at this time due to current WebAssembly constraints (tooling builds
/// and loads a library, not a standalone binary).
///
/// Behavior:
/// - Initializes logging and prints a startup message.
/// - Builds the engine internally and registers a few basic meshes in the
///   shared resources (useful as ready-made primitives for quick experiments).
/// - Creates the event-loop runner and enters the main loop; this call blocks
///   until the window/tab is closed or the loop terminates.
/// - Shows an exit message and returns.
///
/// Platform notes:
/// - Native: you may create a tiny binary whose `main()` simply calls
///   `oxide::run()`. The engine itself is still created inside this function.
/// - WebAssembly: build your app as a library crate (e.g., `cargo new your_app
///   --lib`) and compile for `wasm32`. The engine must be defined inside this
///   `run()` function; do not attempt to construct it elsewhere. The host
///   (e.g., the browser/bundler) will drive the runtime, and this function will
///   manage initialization on your behalf.
///
/// Roadmap:
/// - A procedural macro is planned to let applications hook into initialization
///   while keeping the WebAssembly-friendly setup. Until then, `run()` remains
///   the only place where the engine is constructed.
/// - This crate is developed as part of a Bachelor's thesis project; APIs are
///   subject to change.
///
/// # Returns
///
/// - `Ok(())` when the event loop exits cleanly.
/// - An error if engine construction or the runner encounter a failure.
///
/// # Example
///
/// Example (native wrapper):
/// ```no_run
/// {
/// // Engine is created inside `oxide::run()`.
/// oxide::run().unwrap()
/// ```
pub fn run() -> anyhow::Result<()>
{
        utils::bootstrap::config_logging();

        utils::bootstrap::show_start_message();

        let engine = engine::EngineBuilder::new().build()?;

        let _mesh_pentagon = Mesh::basic("pentagon", Primitive::Pentagon);

        let _mesh_square = Mesh::basic("square", Primitive::Square);

        let _mesh_triangle = Mesh::basic("triangle", Primitive::Triangle);

        let hexagon = Mesh::generate_n_gon(128, 0.75);

        {
                let mut resources = engine.resources.lock().unwrap_or_else(|e| e.into_inner());

                resources.add_mesh(hexagon);
        }

        let runner = engine::EngineRunner::new(engine)?;

        runner.run()?;

        utils::exit::show_exit_message();

        Ok(())
}

/// WebAssembly start entry point for the runtime.
///
/// Compiled and exported only on `wasm32` targets, this function is invoked
/// automatically by the `wasm-bindgen` bootstrap when the module is
/// instantiated. It installs a panic hook so Rust panics are logged to the
/// browser console, then delegates to [`run()`].
///
/// Error propagation:
/// - Errors from [`run()`] are mapped into a `JsValue` and returned. This
///   causes module instantiation to fail (e.g., the loader will observe a
///   rejected Promise or thrown exception), allowing JavaScript to handle the
///   failure.
/// - By default, the mapped value is a string. If your application needs a real
///   `Error` object, adjust the mapper to return `js_sys::Error`.
///
/// Returns:
/// - `Ok(())` on successful initialization and startup.
/// - `Err(JsValue)` if initialization fails; the value contains a formatted
///   error message.
///
/// This function is not meant to be called directly from JavaScript; it runs
/// once on module load.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn run_oxide_wasm() -> Result<(), JsValue>
{
        console_error_panic_hook::set_once();

        run().map_err(|e| JsValue::from_str(&format!("oxide run failed: {e:#}")))
}
