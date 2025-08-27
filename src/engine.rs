//! Oxide Engine Module
//!
//! This module provides the core engine functionality for Oxide, including:
//! - Engine construction via [`EngineBuilder`]
//! - Engine lifecycle management through [`EngineRunner`]
//! - User-defined setup via the [`EngineHandler`] trait
//! - Platform-agnostic entry points for native and WASM targets
//!
//! # Key Concepts
//!
//! ## Singleton Engine
//! Only one engine instance is allowed per process. Attempting to create
//! multiple instances will result in a panic. This simplifies GPU resource
//! management and event loop handling.
//!
//! ## EngineHandler
//! Users define their engine behavior by implementing [`EngineHandler`]
//! and providing a static `setup` function that returns an [`EngineRunner`].
//! This setup function is registered internally and later executed by [`run`].
//!
//! ## Platform Differences
//! - **Native targets**: `run::<H>()` registers the setup function and
//!   immediately starts the engine loop.
//! - **WASM targets**: [`run_wasm`] is automatically called when the WASM
//!   module is loaded. Users should define the engine in Rust via
//!   [`EngineHandler`] and rely on the WASM entry point for execution.
//!
//! # Usage
//!
//! ```rust
//! use oxide::engine::EngineHandler;
//!
//! struct App;
//!
//! impl EngineHandler for App
//! {
//!         fn setup() -> oxide::engine::EngineRunner
//!         {
//!                 let engine = oxide::engine::EngineBuilder::new().build().unwrap();
//!
//!                 oxide::engine::EngineRunner::new(engine).unwrap()
//!         }
//! }
//!
//! // Run the engine (native execution)
//! oxide::engine::run::<App>();
//! ```
//!
//! On WASM, the same `App` setup is used, but the engine starts automatically
//! when the module is loaded in the browser.

use std::{
        collections::HashMap,
        sync::{Arc, OnceLock},
};

#[cfg(target_arch = "wasm32")]
use winit::platform::web::EventLoopExtWebSys;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use winit::{
        application::ApplicationHandler,
        event::{KeyEvent, WindowEvent},
        event_loop::{ActiveEventLoop, EventLoop},
        keyboard::{KeyCode, PhysicalKey},
        window::WindowId,
};

use crate::{geometry::mesh::Mesh, resource::Resources};

// Engine manages a global setup function
static SETUP_FN: OnceLock<fn() -> EngineRunner> = OnceLock::new();

fn register_setup<H: EngineHandler>()
{
        SETUP_FN.set(H::setup)
                .expect("OxideEngine: Setup function already registered");
}

/// A user-defined handler for your application.
///
/// # Overview
///
/// The [`EngineHandler`] trait allows you to define how your application
/// initializes the engine. It exposes a single required function:
/// [`EngineHandler::setup`]. This function is **static** (does not take
/// `&self`) so it can be safely called in WebAssembly environments, where a
/// plain function pointer is often required by the runtime (e.g.,
/// `wasm-bindgen` startup).
///
/// Implementors of this trait typically build the engine through
/// [`EngineBuilder`](crate::engine::EngineBuilder), then wrap it into an
/// [`EngineRunner`](crate::engine::EngineRunner).
///
/// # Example
///
/// ```
/// use oxide::engine::{EngineHandler, EngineBuilder, EngineRunner};
///
/// struct App;
///
/// impl EngineHandler for App
/// {
///        fn setup() -> oxide::engine::EngineRunner
///        {
///                log::info!("Setting up engine!");
///
///                let engine = oxide::engine::EngineBuilder::new().build().unwrap();
///
///                oxide::engine::EngineRunner::new(engine).unwrap()
///        }
/// }
///
/// // The user then runs the App.
/// oxide::engine::run::<App>();
/// ```
pub trait EngineHandler
{
        /// Called exactly once to build and return the engine runner.
        ///
        /// This is a **static function** instead of `&mut self` because:
        /// - It must be callable from platform entrypoints (native and WASM).
        /// - It does not depend on instance state: the engine setup is global.
        ///
        /// # Returns
        ///
        /// An [`EngineRunner`](crate::engine::EngineRunner) instance that
        /// drives the engine loop.
        fn setup() -> EngineRunner;
}

/// Runs the engine using the given [`EngineHandler`] implementation.
///
/// This is the main entry point for native applications.  
/// It registers the `setup` function of the given handler type `H`
/// so that it can later be retrieved and invoked in [`start`].
///
/// On native platforms, this function:
/// 1. Registers the engine's setup routine via [`register_setup`].
/// 2. Calls [`start`] to construct an [`EngineRunner`] and begin the
///    application's main loop.
///
/// On `wasm32` targets, this function has no direct effect because
/// execution begins from [`run_wasm`] instead.
///
/// # Important
/// - `H` must implement [`EngineHandler`] and provide a static `fn setup() ->
///   EngineRunner`.
/// - On WebAssembly, the setup routine is registered in the same way, but the
///   runtime entry point is [`run_wasm`] due to how `wasm-bindgen` manages
///   lifecycle.
///
/// # Examples
/// ```
/// use oxide::engine::{EngineHandler, EngineBuilder, EngineRunner};
///
/// struct App;
///
/// impl EngineHandler for App
/// {
///        fn setup() -> oxide::engine::EngineRunner
///        {
///                log::info!("Setting up engine!");
///
///                let engine = oxide::engine::EngineBuilder::new().build().unwrap();
///
///                oxide::engine::EngineRunner::new(engine).unwrap()
///        }
/// }
///
/// // The user then runs the App.
/// oxide::engine::run::<App>();
/// ```
pub fn run<H: EngineHandler>()
{
        register_setup::<H>();
        start();
}

/// Starts the engine by invoking the registered setup function
/// and running the returned [`EngineRunner`].
///
/// This function is platform-agnostic, but only executes on
/// **native targets** (`not wasm32`). On WebAssembly targets,
/// [`run_wasm`] is the entry point, which internally calls this function.
///
/// # Panics
/// - If no setup function has been registered via [`run`] or
///   [`register_setup`], this function will panic.
/// - If the underlying [`EngineRunner::run`] call fails, it will panic.
///
/// # Notes
/// - `start` is intentionally split from [`run`] so that both native and wasm
///   entry points can share the same logic.
/// - On wasm, [`run_wasm`] calls this function after the environment is
///   initialized.
///
/// # Examples
/// ```ignore
/// // Normally not called directly by user code.
/// run::<MyApp>(); // internally calls start()
/// ```
fn start()
{
        #[cfg(not(target_arch = "wasm32"))]
        {
                let setup = SETUP_FN.get().expect("No setup function registered");

                let runner = setup();

                runner.run().unwrap();
        }
}

/// WebAssembly entry point for the engine runtime.
///
/// This function is automatically called by the browser when
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
pub fn run_wasm() -> Result<(), JsValue>
{
        console_error_panic_hook::set_once();

        start(); // <-- calls a statically defined fn

        Ok(())
}

/// Runner for the [`Engine`].
pub struct EngineRunner
{
        pub engine: Option<Engine>,

        pub event_loop: EventLoop<()>,
}

impl EngineRunner
{
        /// Constructor for [`EnginerRunner`].
        ///
        /// Most importantly, it creates the `event_loop` from
        /// `winit::EventLoop`.
        ///
        /// # Returns
        ///
        /// `anyhow::Result<EngineRunner>`.
        pub fn new(engine: Engine) -> anyhow::Result<Self>
        {
                let event_loop = winit::event_loop::EventLoop::with_user_event().build()?;

                Ok(Self {
                        engine: Some(engine),
                        event_loop,
                })
        }

        /// Runner for [`EngineRunner`].
        ///
        /// Executes the `run_app()` function of `winit::EventLoop`.
        ///
        /// Checks if the [`Engine`] is defined.
        ///
        /// # Returns
        ///
        /// `anyhow::Result<()>`. because `run_app()` returns a Result.
        pub fn run(self) -> anyhow::Result<()>
        {
                let mut engine = match self.engine
                {
                        Some(e) => e,
                        None => anyhow::bail!("Engine doesn't exist."),
                };

                #[cfg(target_arch = "wasm32")]
                {
                        let mut engine = Box::leak(Box::new(engine));

                        self.event_loop.spawn_app(engine);
                }

                #[cfg(not(target_arch = "wasm32"))]
                self.event_loop.run_app(&mut engine)?;

                Ok(())
        }
}

/// Main entrypoint of Oxide.
///
/// To construct [`Engine`], use [`EngineBuilder`].
///
/// [Engine] is responsible for the lifetime,
/// event handling and destruction of itself.
///
/// Every field is **Optional**.
#[derive(Debug)]
pub struct Engine
{
        // --- Core Context ---
        /// The OS/Browser window for rendering and input handling.
        pub window: Option<Arc<winit::window::Window>>,

        pub core: EngineCore,

        /// `wgpu` internals.
        pub state: Option<EngineState>,

        // --- Timing ---
        /// The timestamp of the last frame, used for delta time calculations.
        pub time: Option<std::time::Instant>,

        // --- Rendering ---
        /// Context for rendering, may contain pipelines, passes, and resources.
        pub render_ctx: Option<crate::renderer::RenderContext>,

        // --- Scene / Resources ---
        /// The active scene graph or world being rendered and updated.
        pub scene: Option<crate::scene::Scene>,

        pub resources: Resources,

        /// The main camera used to view the scene.
        pub camera: Option<crate::scene::camera::Camera>,

        // --- Misc ---
        /// Input system state (keyboard, mouse, gamepad, etc.).
        pub input: Option<crate::input::InputState>,

        /// Optional UI system (e.g., egui) for rendering overlays.
        pub ui: Option<crate::ui::UiSystem>,
}

impl Engine
{
        pub fn add_mesh(
                &mut self,
                name: &str,
                mesh: Mesh,
        )
        {
                self.resources.meshes.insert(name.to_string(), mesh);
        }
}

/// EngineState holds all GPU-related resources for rendering.
///
/// # Notes
/// - This struct is initialized during engine setup and assumes a persistent
///   window.
/// - The GPU device and queue are used for submitting rendering commands.
/// - The surface and its configuration must match the window's size and format.
///
/// # Panics
/// This function will panic if:
/// - Creating the surface fails.
/// - Selecting an adapter fails.
/// - Creating the device and queue fails.
#[derive(Debug)]
pub struct EngineState
{
        /// The rendering surface tied to the window.
        pub surface: wgpu::Surface<'static>,

        /// The handle to a physical graphics device.
        pub adapter: wgpu::Adapter,

        /// The GPU device handle used to submit rendering commands.
        pub device: wgpu::Device,

        /// The GPU queue used to execute command buffers.
        pub queue: wgpu::Queue,

        pub surface_configuration: wgpu::SurfaceConfiguration,

        pub texture_format: wgpu::TextureFormat,

        pub surface_caps: wgpu::SurfaceCapabilities,
}

#[derive(Debug)]
pub struct EngineCore
{
        pub instance: wgpu::Instance,
}

impl EngineState
{
        /// Creates a new EngineState by initializing the surface, adapter,
        /// device, and queue.
        ///
        /// # Parameters
        /// - `instance`: WGPU instance to create surfaces and request adapters.
        /// - `window`: The window to render to. Must outlive the EngineState.
        ///
        /// # Panics
        /// Panics if surface creation, adapter selection, or device/queue
        /// creation fails.
        pub fn new(
                instance: &wgpu::Instance,
                window: Arc<winit::window::Window>,
        ) -> Self
        {
                let size = window.inner_size();

                let surface = instance.create_surface(window.clone()).unwrap();

                let adapter = pollster::block_on(EngineBuilder::adapter(instance, window.clone()))
                        .unwrap();

                let (device, queue) =
                        pollster::block_on(EngineBuilder::device_queue(&adapter)).unwrap();

                let surface_caps = surface.get_capabilities(&adapter);

                let texture_format = EngineBuilder::texture_format(&surface_caps);

                let surface_configuration =
                        EngineBuilder::surface_configuration(texture_format, &size, &surface_caps);

                EngineState {
                        surface,
                        adapter,
                        device,
                        queue,
                        surface_caps,
                        texture_format,
                        surface_configuration,
                }
        }
}

impl ApplicationHandler for Engine
{
        fn resumed(
                &mut self,
                event_loop: &winit::event_loop::ActiveEventLoop,
        )
        {
                // From the winit docs:
                //
                // # Portability
                //
                // It’s recommended that applications should only initialize their
                // graphics context and create a window after they have received their first
                // Resumed event. Some systems (specifically Android) won’t allow applications
                // to create a render surface until they are resumed.
                //
                // Reference: https://docs.rs/winit/latest/winit/application/trait.ApplicationHandler.html#tymethod.resumed
                //

                if self.state.is_some()
                {
                        log::info!("Engine already resumed, skipping initialization.");
                        return;
                }

                let window = Arc::new(
                        event_loop
                                .create_window(winit::window::Window::default_attributes())
                                .unwrap(),
                );

                self.window = Some(window.clone());

                let state = EngineState::new(&self.core.instance, window.clone());

                self.state = Some(state);
        }

        fn window_event(
                &mut self,
                event_loop: &ActiveEventLoop,
                #[allow(unused_variables)] id: WindowId,
                event: WindowEvent,
        )
        {
                match event
                {
                        WindowEvent::CloseRequested =>
                        {
                                println!("The close button was pressed; stopping");
                                event_loop.exit();
                        }
                        WindowEvent::RedrawRequested =>
                        {
                                self.window.as_ref().unwrap().request_redraw();
                        }
                        WindowEvent::KeyboardInput {
                                event:
                                        KeyEvent {
                                                physical_key: PhysicalKey::Code(code),
                                                state: key_state,
                                                ..
                                        },
                                ..
                        } =>
                        {
                                println!("Code: {:?}, KeyState: {:?}", code, key_state);

                                if code == KeyCode::Escape
                                {
                                        event_loop.exit();
                                }
                        }
                        _ => (),
                }
        }
}

/// A builder for the engine, responsible for preparing configuration
/// and state before GPU resources are initialized.
///
/// # Important
///
/// Calling [`EngineBuilder::new`] or similar methods like
/// [`EngineBuilder::build`] does **not** immediately create a
/// fully usable [`Engine`].
///
/// These methods prepare an internal [`EngineState`] that is only
/// finalized when the application lifecycle reaches
/// [`ApplicationHandler::resumed`]. Some resources (e.g.,
/// `wgpu::Surface`) require a live window handle, which is only
/// available after `resumed()` is called.
///
/// In other words, constructing an `EngineBuilder` sets up everything
/// that *can* be initialized ahead of time, while deferring
/// GPU- and window-dependent setup until the first `resumed()` event.
///
/// ## Consequences
/// - Do not expect a complete `Engine` immediately after builder methods.
/// - GPU surface, device, and swapchain are created during `resumed()`.
/// - After `resumed()` runs, the engine is fully initialized and ready to
///   render.
///
/// # Example
///
/// ```rust
/// use oxide::engine::{EngineBuilder, EngineRunner, Engine};
///
/// let engine: Engine = EngineBuilder::new()
///     .build()
///     .unwrap();
///
/// // EngineState is None initially.
/// assert!(engine.state.is_none());
///
/// let runner: EngineRunner = EngineRunner::new(engine).unwrap();
///
/// // EngineState is still None.
/// assert!(runner.engine.unwrap().state.is_none());
///
/// // Once run is called, EngineState is constructed.
/// // runner.run().unwrap();
/// ```
#[derive(Debug)]
pub struct EngineBuilder
{
        engine: Engine,
}

#[allow(clippy::new_without_default)]
impl EngineBuilder
{
        /// Creates a new `EngineBuilder` with default configuration.
        ///
        /// # Returns
        ///
        /// `EngineBuilder`.
        ///
        /// See [`EngineBuilder`] for important notes on deferred
        /// initialization.
        pub fn new() -> Self
        {
                let instance = EngineBuilder::instance();

                let core = EngineCore {
                        instance,
                };

                let resources = Resources::new();

                Self {
                        engine: Engine {
                                resources,
                                core,
                                state: None,
                                time: None,
                                render_ctx: None,
                                scene: None,
                                camera: None,
                                input: None,
                                ui: None,
                                window: None,
                        },
                }
        }

        pub fn keybind<F>(
                self,
                key_code: winit::keyboard::KeyCode,
                f: F,
        ) -> Self
        where
                F: FnOnce(winit::keyboard::KeyCode),
        {
                f(key_code);
                self
        }

        /// Set the time (for delta timing)
        pub fn with_time(
                mut self,
                time: std::time::Instant,
        ) -> Self
        {
                self.engine.time = Some(time);
                self
        }

        /// Set the renderer context
        pub fn with_render_ctx(
                mut self,
                render_ctx: crate::renderer::RenderContext,
        ) -> Self
        {
                self.engine.render_ctx = Some(render_ctx);
                self
        }

        /// Set the scene
        pub fn with_scene(
                mut self,
                scene: crate::scene::Scene,
        ) -> Self
        {
                self.engine.scene = Some(scene);
                self
        }

        /// Set the camera
        pub fn with_camera(
                mut self,
                camera: crate::scene::camera::Camera,
        ) -> Self
        {
                self.engine.camera = Some(camera);
                self
        }

        /// Set the input system
        pub fn with_input(
                mut self,
                input: crate::input::InputState,
        ) -> Self
        {
                self.engine.input = Some(input);
                self
        }

        /// Set the UI system
        pub fn with_ui(
                mut self,
                ui: crate::ui::UiSystem,
        ) -> Self
        {
                self.engine.ui = Some(ui);
                self
        }

        /// Finally builds the [`Engine`].
        ///
        /// Does some fields validation.
        ///
        /// Generates the `wgpu::Instance` and sets the `instance` field.
        ///
        /// # Returns
        ///
        /// `anyhow::Result<Engine>`.
        ///
        /// See [`EngineBuilder`] for important notes on deferred
        /// initialization.
        pub fn build(self) -> anyhow::Result<Engine>
        {
                Ok(self.engine)
        }

        fn instance() -> wgpu::Instance
        {
                wgpu::Instance::new(&wgpu::InstanceDescriptor {
                        #[cfg(not(target_arch = "wasm32"))]
                        backends: wgpu::Backends::PRIMARY,
                        #[cfg(target_arch = "wasm32")]
                        backends: wgpu::Backends::GL,
                        ..Default::default()
                })
        }

        fn surface<'a>(
                instance: &wgpu::Instance,
                window: Arc<winit::window::Window>,
        ) -> anyhow::Result<wgpu::Surface<'a>, wgpu::CreateSurfaceError>
        {
                instance.create_surface(window)
        }

        async fn adapter(
                instance: &wgpu::Instance,
                window: Arc<winit::window::Window>,
        ) -> anyhow::Result<wgpu::Adapter>
        {
                let surface = Self::surface(instance, window)?;

                let adapter = instance
                        .request_adapter(&wgpu::RequestAdapterOptions {
                                // Either `HighPerformance` or `LowPower`.
                                //
                                // 1. LowPower will pick an adapter that favors battery life.
                                //
                                // 2. HighPerformance will pick an adapter for more power-hungry yet
                                //    more performant GPU's, such as a dedicated graphics card.
                                power_preference: wgpu::PowerPreference::HighPerformance,

                                // Tells wgpu to find an adapter that can present to the supplied
                                // surface.
                                compatible_surface: Some(&surface),

                                // Forces wgpu to pick an adapter that will work on all hardware.
                                // Generally a software implementation on most systems.
                                force_fallback_adapter: false,
                        })
                        .await
                        .map_err(|e| anyhow::anyhow!(e))?;

                Ok(adapter)
        }

        pub async fn device_queue(
                adapter: &wgpu::Adapter
        ) -> anyhow::Result<(wgpu::Device, wgpu::Queue), wgpu::RequestDeviceError>
        {
                adapter.request_device(&wgpu::DeviceDescriptor {
                        label: Some("device_queue"),
                        required_features: wgpu::Features::default(),
                        required_limits: wgpu::Limits::default(),
                        memory_hints: wgpu::MemoryHints::Performance,
                        trace: wgpu::Trace::Off,
                })
                .await
        }

        fn texture_format(surface_caps: &wgpu::SurfaceCapabilities) -> wgpu::TextureFormat
        {
                surface_caps
                        .formats
                        .iter()
                        .find(|f| f.is_srgb())
                        .copied()
                        .unwrap_or(surface_caps.formats[0])
        }

        fn surface_configuration(
                surface_format: wgpu::TextureFormat,
                size: &winit::dpi::PhysicalSize<u32>,
                surface_caps: &wgpu::SurfaceCapabilities,
        ) -> wgpu::SurfaceConfiguration
        {
                wgpu::SurfaceConfiguration {
                        // Describes how SurfaceTextures will be used.
                        // RENDER_ATTACHMET is guaranteed to be supported.
                        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,

                        format: surface_format,

                        width: size.width,

                        height: size.height,

                        #[cfg(target_arch = "wasm32")]
                        present_mode: surface_caps.present_modes[0],

                        // IMMEDIATE: No VSync for non wasm
                        // environments, because wasm only has 1
                        // present mode.
                        #[cfg(not(target_arch = "wasm32"))]
                        present_mode: surface_caps.present_modes[1],

                        alpha_mode: surface_caps.alpha_modes[0],

                        view_formats: vec![],

                        desired_maximum_frame_latency: 2,
                }
        }
}
