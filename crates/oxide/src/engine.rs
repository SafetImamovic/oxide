//! Oxide Engine Module
//!
//! This module provides the core engine functionality for Oxide, including
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

use std::sync::Arc;
use std::sync::Mutex;
use wgpu::PresentMode;
#[cfg(target_arch = "wasm32")]
use winit::platform::web::EventLoopExtWebSys;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use crate::renderer::graph::BackgroundPass;
use crate::renderer::graph::GeometryPass;
use crate::renderer::graph::RenderGraph;
use crate::renderer::pipeline::PipelineKind;
use crate::ui::renderer::GuiRenderer;
use crate::{renderer::pipeline::PipelineManager, resource::Resources};
use winit::window::Window;
use winit::{
        application::ApplicationHandler,
        event::{KeyEvent, WindowEvent},
        event_loop::{ActiveEventLoop, EventLoop},
        keyboard::{KeyCode, PhysicalKey},
        window::WindowId,
};

/// Runner for the [`Engine`].
pub struct EngineRunner
{
        pub engine: Option<Engine>,

        pub event_loop: EventLoop<EngineState>,
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
        pub fn new(#[allow(unused_mut)] mut engine: Engine) -> anyhow::Result<Self>
        {
                let event_loop: EventLoop<EngineState> = EventLoop::with_user_event().build()?;

                #[cfg(target_arch = "wasm32")]
                {
                        engine.proxy = Some(event_loop.create_proxy());
                }

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
                #[allow(unused_mut)]
                let mut engine = match self.engine
                {
                        Some(e) => e,
                        None => anyhow::bail!("Engine doesn't exist."),
                };

                #[cfg(target_arch = "wasm32")]
                {
                        let engine = Box::leak(Box::new(engine));

                        self.event_loop.spawn_app(engine);
                }

                #[cfg(not(target_arch = "wasm32"))]
                self.event_loop.run_app(&mut engine)?;

                Ok(())
        }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum FillMode
{
        Fill = 0,
        Wireframe = 1,
        Vertex = 2,
}

/// Main entrypoint of Oxide.
///
/// To construct [`Engine`], use [`EngineBuilder`].
///
/// [Engine] is responsible for the lifetime,
/// event handling, and destruction of itself.
///
/// Every field is **Optional**.
#[derive(Debug)]
pub struct Engine
{
        /// On browser environments, an [`EventLoopProxy`] is needed
        /// to send events back into the event loop asynchronously.
        #[cfg(target_arch = "wasm32")]
        pub proxy: Option<winit::event_loop::EventLoopProxy<EngineState>>,

        pub ui_scale: f32,

        /// Polygon fill mode, depends on the platforms wgpu features.
        pub fill_mode: FillMode,

        // --- Core Context ---
        /// The OS/Browser window for rendering and input handling.
        pub window: Option<Arc<Window>>,

        /// `wgpu` internals.
        pub state: Option<EngineState>,

        // --- Timing ---
        /// The timestamp of the last frame, used for delta time calculations.
        pub time: Option<instant::Instant>,

        pub resources: Arc<Mutex<Resources>>,

        /// The main camera used to view the scene.
        pub camera: Option<crate::renderer::camera::Camera>,

        // --- Misc ---
        /// Input system state (keyboard, mouse, gamepad, etc.).
        pub input: Option<crate::input::InputState>,

        /// Optional UI system (e.g., egui) for rendering overlays.
        pub ui: Option<crate::ui::UiSystem>,
}

impl Engine
{
        pub fn render(&mut self) -> anyhow::Result<()>
        {
                let state = match self.state.as_mut()
                {
                        None =>
                        {
                                anyhow::bail!("EngineState doesn't exist.");
                        }
                        Some(s) => s,
                };

                let window = match self.window.as_ref()
                {
                        None =>
                        {
                                anyhow::bail!("Window doesn't exist.");
                        }
                        Some(w) => w.clone(),
                };

                // The _resize() method is called and sets this flag to true
                if !state.is_surface_configured
                {
                        return Ok(());
                }

                // Get the surface texture ONCE per frame
                //
                // Returns the next texture to be presented by the swapchain for drawing.
                //
                // In order to present the SurfaceTexture returned by this method,
                // first a Queue::submit needs to be done with some work rendering to this
                // texture. Then SurfaceTexture::present needs to be called.
                //
                // ```rust
                //         state.queue.submit(std::iter::once(encoder.finish())); // oxide::EngineState
                //         output.present(); // wgpu::SurfaceTexture
                // ```
                //
                // If a SurfaceTexture referencing this surface is alive when the swapchain is
                // recreated, recreating the swapchain will panic
                let output = match state.surface.get_current_texture()
                {
                        Ok(frame) => frame,
                        Err(wgpu::SurfaceError::Outdated) =>
                        {
                                // This often happens during window resizing
                                println!("wgpu surface outdated");
                                return Err(wgpu::SurfaceError::Outdated).map_err(Into::into);
                        }
                        Err(e) =>
                        {
                                eprintln!("Failed to acquire surface texture: {:?}", e);
                                return Err(e).map_err(Into::into);
                        }
                };

                let view = output
                        .texture
                        .create_view(&wgpu::TextureViewDescriptor::default());

                let mut encoder =
                        state.device
                                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                                        label: Some("Main Render Encoder"),
                                });

                // Pass depth texture view to render graph
                state.render_graph
                        .execute(&view, &mut encoder, &state.pipeline_manager);

                // ------------------ GUI ----------------------

                let pixels_per_point = self.ui_scale;

                let screen_descriptor = egui_wgpu::ScreenDescriptor {
                        size_in_pixels: [
                                state.surface_configuration.width,
                                state.surface_configuration.height,
                        ],
                        pixels_per_point, /* inversely counteracts the
                                           * Browser DPI */
                };

                {
                        let supported = state.adapter.features();

                        let desired = wgpu::Features::POLYGON_MODE_LINE
                                | wgpu::Features::POLYGON_MODE_POINT
                                | wgpu::Features::TIMESTAMP_QUERY;

                        let enabled_features = supported & desired;

                        state.gui.begin_frame(&window.clone(), &mut self.ui_scale);

                        let temp_fill_mode = self.fill_mode;

                        state.gui.render(
                                &mut state.render_graph,
                                &mut self.ui_scale,
                                &mut self.fill_mode,
                                enabled_features,
                        );

                        if temp_fill_mode != self.fill_mode
                        {
                                log::info!("Fill Mode: {:?}", self.fill_mode);

                                // Request Pipeline Rebuild
                                state.pipeline_manager.rebuild_geometry_pipeline(
                                        &state.device,
                                        &state.surface_configuration,
                                        self.fill_mode,
                                        &[],
                                );
                        }

                        state.gui.end_frame_and_draw(
                                &state.device,
                                &state.queue,
                                &mut encoder,
                                &window.clone(),
                                &view,
                                screen_descriptor,
                        );
                }

                state.queue.submit(std::iter::once(encoder.finish()));

                output.present();

                Ok(())
        }

        fn resize(&mut self)
        {
                #[cfg(target_arch = "wasm32")]
                {
                        let (width, height) = Self::get_body_size().unwrap();
                        self._resize(width, height);
                }

                #[cfg(not(target_arch = "wasm32"))]
                {
                        let size = self.window.as_ref().unwrap().inner_size();
                        self._resize(size.width, size.height);
                }
        }

        #[cfg(target_arch = "wasm32")]
        fn get_body_size() -> Option<(u32, u32)>
        {
                let window = web_sys::window()?;

                let document = window.document()?;

                let body = document.body()?;

                let width = body.client_width() as u32;

                let height = body.client_height() as u32;

                log::info!("Body: {}, {}", width, height);

                Some((width, height))
        }

        /// Handles window resize events.
        ///
        /// # Parameters
        /// - `width`: New window width in pixels
        /// - `height`: New window height in pixels
        fn _resize(
                &mut self,
                width: u32,
                height: u32,
        )
        {
                if width == 0 || height == 0
                {
                        return;
                }

                let state = &mut self.state.as_mut().unwrap();

                // Clamping to max dim to prevent panic!
                let max_dim = state.device.limits().max_texture_dimension_2d;
                let final_width = width.min(max_dim);
                let final_height = height.min(max_dim);

                //log::info!("Resizing surface -> width: {}, height: {}", final_width,
                // final_height);

                state.surface_configuration.width = final_width;
                state.surface_configuration.height = final_height;

                state.surface
                        .configure(&state.device, &state.surface_configuration);

                state.is_surface_configured = true;
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
        pub gui: GuiRenderer,

        pub is_surface_configured: bool,

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

        pub vertex_buffers: Vec<wgpu::Buffer>,

        pub index_buffers: Vec<wgpu::Buffer>,

        pub render_graph: RenderGraph,

        pub pipeline_manager: PipelineManager,
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
        pub async fn new(window: Arc<Window>) -> anyhow::Result<EngineState>
        {
                let instance = EngineBuilder::instance();

                Self::log_all_adapters(&instance);

                let size = window.inner_size();

                let surface = instance.create_surface(window.clone())?;

                let adapter = EngineBuilder::adapter(&instance, window.clone()).await?;

                Self::log_adapter_info(&adapter);

                let (device, queue) = EngineBuilder::device_queue(&adapter).await?;

                let surface_caps = surface.get_capabilities(&adapter);

                let texture_format = EngineBuilder::texture_format(&surface_caps);

                let surface_configuration =
                        EngineBuilder::surface_configuration(texture_format, &size, &surface_caps);

                let depth_texture = crate::texture::Texture::create_depth_texture(
                        &device,
                        &surface_configuration,
                        "depth_texture",
                );

                let mut pipeline_manager = PipelineManager::new();

                let geom_pipeline = PipelineManager::create_geometry_pipeline(
                        &device,
                        &surface_configuration,
                        &[],
                        &depth_texture,
                        &FillMode::Fill,
                );

                pipeline_manager
                        .render_pipelines
                        .insert(PipelineKind::Geometry, geom_pipeline);

                let bg_pass = BackgroundPass {
                        name: "bg_pass".to_string(),
                        enabled: true,
                        clear_color: wgpu::Color {
                                r: 1.0,
                                g: 0.0,
                                b: 0.0,
                                a: 1.0,
                        },
                };

                let bg_pass_2 = BackgroundPass {
                        name: "bg_pass_2".to_string(),
                        enabled: true,
                        clear_color: wgpu::Color {
                                r: 0.0,
                                g: 1.0,
                                b: 0.0,
                                a: 1.0,
                        },
                };

                let bg_pass_3 = BackgroundPass {
                        name: "bg_pass_3".to_string(),
                        enabled: true,
                        clear_color: wgpu::Color {
                                r: 0.0,
                                g: 0.0,
                                b: 1.0,
                                a: 1.0,
                        },
                };

                let mut render_graph = RenderGraph {
                        passes: vec![],
                };

                render_graph.add_pass(Box::new(bg_pass));
                render_graph.add_pass(Box::new(bg_pass_2));
                render_graph.add_pass(Box::new(bg_pass_3));

                let gui = GuiRenderer::new(&device, surface_configuration.format, None, 1, &window);

                Ok(EngineState {
                        render_graph,
                        is_surface_configured: false,
                        pipeline_manager,
                        gui,
                        surface,
                        adapter,
                        device,
                        queue,
                        surface_caps,
                        texture_format,
                        surface_configuration,
                        index_buffers: vec![],
                        vertex_buffers: vec![],
                })
        }

        pub fn log_adapter_info(adapter: &wgpu::Adapter)
        {
                log::info!("Adapter Info: {:#?}", adapter.get_info());
        }

        /// Logs the adapter features.
        ///
        /// Corresponds to these WebGPU feature Reference
        /// <https://gpuweb.github.io/gpuweb/#enumdef-gpufeaturename>
        pub fn get_adapter_features(&self) -> wgpu::Features
        {
                self.adapter.features()
        }

        pub fn get_all_adapters(instance: &wgpu::Instance) -> Vec<wgpu::Adapter>
        {
                instance.enumerate_adapters(wgpu::Backends::all())
        }

        pub fn log_all_adapters(instance: &wgpu::Instance)
        {
                log::info!("All Available Adapters:");

                Self::get_all_adapters(instance)
                        .iter()
                        .for_each(|a| log::info!("\t{:#?}", a.get_info()));
        }
}

impl ApplicationHandler<EngineState> for Engine
{
        fn resumed(
                &mut self,
                event_loop: &ActiveEventLoop,
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

                #[allow(unused_mut)]
                let mut window_attributes =
                        Window::default_attributes().with_title("Oxide Render Engine");

                #[cfg(target_arch = "wasm32")]
                {
                        use wasm_bindgen::JsCast;
                        use winit::platform::web::WindowAttributesExtWebSys;

                        const CANVAS_ID: &str = "canvas";

                        let window = wgpu::web_sys::window().unwrap_throw();
                        let document = window.document().unwrap_throw();
                        let canvas = document.get_element_by_id(CANVAS_ID).unwrap_throw();
                        let html_canvas_element = canvas.unchecked_into();

                        window_attributes =
                                window_attributes.with_canvas(Some(html_canvas_element));
                }

                let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

                self.window = Some(window.clone());

                #[cfg(not(target_arch = "wasm32"))]
                {
                        // Native builds can block on async state initialization.
                        self.state = Some(pollster::block_on(EngineState::new(window)).unwrap());
                }

                #[cfg(target_arch = "wasm32")]
                {
                        // In WASM builds, async tasks must be spawned without blocking.
                        #[cfg(target_arch = "wasm32")]
                        if let Some(proxy) = self.proxy.take()
                        {
                                wasm_bindgen_futures::spawn_local(async move {
                                        let state_result = EngineState::new(window).await;
                                        match state_result
                                        {
                                                Ok(state) =>
                                                {
                                                        web_sys::console::log_1(
                                                                &"State initialized, sending event"
                                                                        .into(),
                                                        );
                                                        assert!(proxy.send_event(state).is_ok());
                                                }
                                                Err(e) => web_sys::console::error_1(
                                                        &format!("State init failed: {:?}", e)
                                                                .into(),
                                                ),
                                        }
                                });
                        }
                        else
                        {
                                web_sys::console::log_1(
                                        &"Proxy is None, skipping async init".into(),
                                );
                        }
                }

                #[cfg(not(target_arch = "wasm32"))]
                {
                        let state = self.state.as_mut().unwrap();

                        let geometry_pass = GeometryPass {
                                name: "geometry_pass".to_string(),
                                enabled: true,
                                resources: self.resources.clone(),
                        };

                        state.render_graph.add_pass(Box::new(geometry_pass));

                        self.resources.lock().unwrap().upload_all(&state.device);
                }
        }

        /// Handles custom user events.
        ///
        /// On WASM, async initialization sends the completed [`State`] via a
        /// proxy, which is received here and stored.
        fn user_event(
                &mut self,
                _event_loop: &ActiveEventLoop,
                _event: EngineState,
        )
        {
                #[cfg(target_arch = "wasm32")]
                {
                        self.window
                                .clone()
                                .expect("Window doesn't exist.")
                                .request_redraw();

                        self.state = Some(_event);

                        let state = self.state.as_mut().unwrap();

                        let device: &wgpu::Device = &state.device;

                        let depth_texture = crate::texture::Texture::create_depth_texture(
                                &state.device,
                                &state.surface_configuration,
                                "depth_texture",
                        );

                        //let mut pipeline_manager = PipelineManager::new();

                        let geom_pipeline = PipelineManager::create_geometry_pipeline(
                                &device,
                                &state.surface_configuration,
                                &[],
                                &depth_texture,
                                &FillMode::Fill,
                        );

                        state.pipeline_manager
                                .render_pipelines
                                .insert(PipelineKind::Geometry, geom_pipeline);

                        let geometry_pass = GeometryPass {
                                name: "geometry_pass".to_string(),
                                enabled: true,
                                resources: self.resources.clone(),
                        };

                        state.render_graph.add_pass(Box::new(geometry_pass));

                        self.resources.lock().unwrap().upload_all(&state.device);
                }
        }

        fn window_event(
                &mut self,
                event_loop: &ActiveEventLoop,
                #[allow(unused_variables)] id: WindowId,
                event: WindowEvent,
        )
        {
                let state = match &mut self.state
                {
                        Some(canvas) => canvas,
                        None => return,
                };

                state.gui
                        .handle_input(&self.window.as_ref().unwrap(), &event);

                match event
                {
                        WindowEvent::CloseRequested =>
                        {
                                println!("The close button was pressed; stopping");
                                event_loop.exit();
                        }
                        WindowEvent::Resized(_size) =>
                        {
                                self.resize();
                        }
                        WindowEvent::RedrawRequested =>
                        {
                                let start = instant::Instant::now();

                                match self.render()
                                {
                                        Ok(_) =>
                                        {
                                                let window: Arc<Window> = match self.window.as_ref()
                                                {
                                                        None => return,
                                                        Some(w) => w.clone(),
                                                };

                                                window.request_redraw();

                                                let duration = start.elapsed();

                                                let fps = 1.0 / duration.as_secs_f32();

                                                log::info!(
                                                        "Render frame took: {:.2} ms, FPS: {:.1}",
                                                        duration.as_secs_f64() * 1000.0,
                                                        fps
                                                );
                                        }
                                        Err(e) =>
                                        {
                                                log::error!("Unable to render {}", e);
                                        }
                                }
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
                let resources = Arc::new(Mutex::new(Resources::new()));

                Self {
                        engine: Engine {
                                #[cfg(target_arch = "wasm32")]
                                proxy: None,
                                resources,
                                fill_mode: FillMode::Fill,
                                ui_scale: 1.5,
                                state: None,
                                time: None,
                                camera: None,
                                input: None,
                                ui: None,
                                window: None,
                        },
                }
        }

        pub fn keybind<F>(
                self,
                key_code: KeyCode,
                f: F,
        ) -> Self
        where
                F: FnOnce(KeyCode),
        {
                f(key_code);
                self
        }

        /// Set the time (for delta timing)
        pub fn with_time(
                mut self,
                time: instant::Instant,
        ) -> Self
        {
                self.engine.time = Some(time);
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
        /// Does some field validation.
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
                window: Arc<Window>,
        ) -> anyhow::Result<wgpu::Surface<'a>, wgpu::CreateSurfaceError>
        {
                instance.create_surface(window)
        }

        async fn adapter(
                instance: &wgpu::Instance,
                window: Arc<Window>,
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
                let supported = adapter.features();

                log::info!("Platform Specific Features: ");

                for i in supported.iter()
                {
                        log::info!("\t{}", i);
                }

                let desired = wgpu::Features::POLYGON_MODE_LINE
                        | wgpu::Features::POLYGON_MODE_POINT
                        | wgpu::Features::TIMESTAMP_QUERY;

                let required_features = supported & desired;

                adapter.request_device(&wgpu::DeviceDescriptor {
                        label: None,
                        required_features,
                        // WebGL doesn't support all of wgpu's features, so if
                        // we're building for the web we'll have to disable some.
                        // Describes the limit of certain types of resources that we can
                        // create.
                        //
                        // Reference <https://gpuweb.github.io/gpuweb/#gpusupportedlimits>
                        required_limits: if cfg!(target_arch = "wasm32")
                        {
                                wgpu::Limits::downlevel_webgl2_defaults()
                        }
                        else
                        {
                                wgpu::Limits::default()
                        },

                        // Provides the adapter with a preferred memory allocation strategy.
                        memory_hints: Default::default(),

                        // Debug tracing.
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
                        present_mode: PresentMode::Immediate,

                        alpha_mode: surface_caps.alpha_modes[0],

                        view_formats: vec![],

                        desired_maximum_frame_latency: 2,
                }
        }
}
