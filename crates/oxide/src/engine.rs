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

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
#[cfg(target_arch = "wasm32")]
use winit::platform::web::EventLoopExtWebSys;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use crate::camera::Camera;
use crate::config::Config;
use crate::material::create_material_bind_group_layout;
use crate::model::Model;
use crate::renderer::graph::BackgroundPass;
use crate::renderer::graph::GeometryPass;
use crate::renderer::graph::RenderGraph;
use crate::renderer::pipeline::PipelineManager;
use crate::renderer::surface::SurfaceManager;
use crate::resources::create_transform_bind_group_layout;
use crate::ui::UiSystem;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use winit::event::{DeviceEvent, DeviceId, ElementState};
use winit::event_loop::ControlFlow;
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
        pub fn new(#[allow(unused_mut)] mut engine: Engine) -> Result<Self>
        {
                let event_loop: EventLoop<EngineState> = EventLoop::with_user_event().build()?;
                event_loop.set_control_flow(ControlFlow::Poll);

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
        pub fn run(self) -> Result<()>
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

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Serialize, Deserialize)]
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

        pub last_render_time: Duration,

        pub config: Config,

        pub model_map: HashMap<String, String>,

        // --- Core Context ---
        /// The OS/Browser window for rendering and input handling.
        pub window: Option<Arc<Window>>,

        /// `wgpu` internals.
        pub state: Option<EngineState>,

        // --- Timing ---
        /// The timestamp of the last frame, used for delta time calculations.
        pub time: Option<instant::Instant>,
}

impl Engine
{
        pub fn render(
                &mut self,
                dt: &Duration,
        ) -> Result<()>
        {
                let state = self.state.as_mut().context("EngineState missing")?;
                let window = self.window.as_ref().context("Window missing")?;

                #[rustfmt::skip]
                let Some((output, frame, mut encoder)) =
                        state.surface_manager.acquire_frame(&state.device)?
                else { return Ok(()); };

                state.render_graph.execute(
                        &frame,
                        &mut encoder,
                        &state.pipeline_manager,
                        &state.camera.get_bind_group(&state.device),
                        &state.depth_texture,
                        Some(&state.models),
                        &state.device,
                );

                if self.config.enable_debug
                {
                        state.show_debug_window(
                                window.clone(),
                                &mut self.config.fill_mode,
                                &frame,
                                &mut encoder,
                                &dt,
                        );
                }

                state.queue.submit(std::iter::once(encoder.finish()));
                output.present();

                state.update(&dt);

                Ok(())
        }

        pub fn add_obj_model(
                &mut self,
                handle: impl Into<String>,
                file_name: impl Into<String>,
        )
        {
                self.model_map.insert(handle.into(), file_name.into());
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

                let state = self.state.as_mut().unwrap();

                state.camera.projection.resize(width, height);

                // Clamping to max dim to prevent panic!
                let max_dim = state.device.limits().max_texture_dimension_2d;
                let final_width = width.min(max_dim);
                let final_height = height.min(max_dim);

                state.surface_manager.configuration.width = final_width;
                state.surface_manager.configuration.height = final_height;

                state.surface_manager
                        .surface
                        .configure(&state.device, &state.surface_manager.configuration);

                state.depth_texture = crate::texture::Texture::create_depth_texture(
                        &state.device,
                        &state.surface_manager.configuration,
                        "depth_texture",
                );

                if state.camera.config.aspect_ratio_correction
                {
                        let aspect = final_width as f32 / final_height as f32;

                        state.camera.projection.aspect = aspect;
                }
                else
                {
                        state.camera.projection.aspect =
                                state.camera.config.initial_aspect.unwrap();
                }

                state.surface_manager.is_surface_configured = true;
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
        pub models: HashMap<String, Model>,

        pub instance: wgpu::Instance,

        /// The rendering surface tied to the window.
        pub surface_manager: SurfaceManager,

        /// The handle to a physical graphics device.
        pub adapter: wgpu::Adapter,

        /// The GPU device handle used to submit rendering commands.
        pub device: wgpu::Device,

        /// The GPU queue used to execute command buffers.
        pub queue: wgpu::Queue,

        pub camera: Camera,

        pub depth_texture: crate::texture::Texture,

        pub vertex_buffers: Vec<wgpu::Buffer>,

        pub index_buffers: Vec<wgpu::Buffer>,

        pub render_graph: RenderGraph,

        pub pipeline_manager: PipelineManager,

        pub gui: UiSystem,
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
        pub async fn new(
                window: Arc<Window>,
                model_map: HashMap<String, String>,
        ) -> Result<EngineState>
        {
                let instance = EngineBuilder::instance();

                Self::log_all_adapters(&instance);

                let size = window.inner_size();

                let adapter = EngineBuilder::adapter(&instance, window.clone()).await?;

                Self::log_adapter_info(&adapter);

                let (device, queue) = EngineBuilder::device_queue(&adapter).await?;

                let surface_manager =
                        SurfaceManager::new(&instance, window.clone(), &size, &adapter)?;

                let pipeline_manager = PipelineManager::new();

                let render_graph = RenderGraph::new();

                let gui = UiSystem::new(
                        &device,
                        &surface_manager.configuration.format,
                        None,
                        1,
                        &window,
                );

                let camera = Camera::new();

                let depth_texture = crate::texture::Texture::create_depth_texture(
                        &device,
                        &surface_manager.configuration,
                        "depth_texture",
                );

                let mut models = HashMap::new();

                for (handle, file_name) in model_map.iter()
                {
                        let model = crate::resources::load_model(
                                file_name,
                                Some("de_dust2"),
                                &device,
                                &queue,
                                &create_material_bind_group_layout(&device),
                                &create_transform_bind_group_layout(&device),
                        )
                        .await?;

                        models.insert(handle.to_string(), model);
                }

                Ok(EngineState {
                        instance,
                        camera,
                        models,
                        render_graph,
                        pipeline_manager,
                        adapter,
                        depth_texture,
                        device,
                        queue,
                        gui,
                        surface_manager,
                        index_buffers: vec![],
                        vertex_buffers: vec![],
                })
        }

        pub fn update(
                &mut self,
                dt: &Duration,
        )
        {
                self.camera.update(&dt);

                for model in self.models.values_mut()
                {
                        model.update(&dt);
                }
        }

        pub fn build_pipelines(&mut self)
        {
                let transform_bind_group_layout = create_transform_bind_group_layout(&self.device);

                let material_bind_group_layout = create_material_bind_group_layout(&self.device);

                let model_transform_bind_group_layout =
                        create_transform_bind_group_layout(&self.device);

                self.pipeline_manager.build_geometry_pipeline(
                        &self.device,
                        &self.surface_manager.configuration,
                        &[
                                &self.camera.get_bind_group_layout(&self.device),
                                &transform_bind_group_layout,
                                &material_bind_group_layout,
                                &model_transform_bind_group_layout,
                        ],
                        &FillMode::Fill,
                );
        }

        pub fn build_passes(&mut self)
        {
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
                                r: 0.05,
                                g: 0.05,
                                b: 0.05,
                                a: 1.0,
                        },
                };

                let geometry_pass = GeometryPass {
                        name: "geometry_pass".to_string(),
                        enabled: true,
                };

                self.render_graph.add_pass(Box::new(bg_pass));
                self.render_graph.add_pass(Box::new(bg_pass_2));
                self.render_graph.add_pass(Box::new(bg_pass_3));
                self.render_graph.add_pass(Box::new(geometry_pass));
        }

        pub fn show_debug_window(
                &mut self,
                window: Arc<Window>,
                fill_mode: &mut FillMode,
                frame: &wgpu::TextureView,
                encoder: &mut wgpu::CommandEncoder,
                dt: &Duration,
        )
        {
                let pixels_per_point = self.gui.ui_scale;

                let screen_descriptor = egui_wgpu::ScreenDescriptor {
                        size_in_pixels: [
                                self.surface_manager.configuration.width,
                                self.surface_manager.configuration.height,
                        ],
                        pixels_per_point, /* inversely counteracts the
                                           * Browser DPI */
                };

                {
                        let supported = self.adapter.features();

                        let desired = wgpu::Features::POLYGON_MODE_LINE
                                | wgpu::Features::POLYGON_MODE_POINT
                                | wgpu::Features::TIMESTAMP_QUERY;

                        let enabled_features = supported & desired;

                        self.gui.renderer
                                .begin_frame(window.clone().as_ref(), &mut self.gui.ui_scale);

                        let mut temp_fill_mode = fill_mode.clone();

                        self.gui.renderer.render(
                                &mut self.render_graph,
                                &mut self.gui.ui_scale,
                                &mut temp_fill_mode,
                                enabled_features,
                                &mut self.camera,
                                &dt,
                                &mut self.models,
                        );

                        if temp_fill_mode != *fill_mode
                        {
                                log::info!("Fill Mode: {:?}", temp_fill_mode);

                                // Create transform bind group layout
                                let transform_bind_group_layout =
                                        create_transform_bind_group_layout(&self.device);

                                let material_bind_group_layout =
                                        create_material_bind_group_layout(&self.device);

                                let model_transform_bind_group_layout =
                                        create_transform_bind_group_layout(&self.device);

                                // Request Pipeline Rebuild
                                self.pipeline_manager.build_geometry_pipeline(
                                        &self.device,
                                        &self.surface_manager.configuration,
                                        &[
                                                &self.camera.get_bind_group_layout(&self.device),
                                                &transform_bind_group_layout,
                                                &material_bind_group_layout,
                                                &model_transform_bind_group_layout,
                                        ],
                                        &temp_fill_mode,
                                );
                        }

                        *fill_mode = temp_fill_mode;

                        self.gui.renderer.end_frame_and_draw(
                                &self.device,
                                &self.queue,
                                encoder,
                                window.clone().as_ref(),
                                &frame,
                                screen_descriptor,
                        );
                }
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

                let model_map = self.model_map.clone();

                #[cfg(not(target_arch = "wasm32"))]
                {
                        self.state = Some(pollster::block_on(EngineState::new(window, model_map))
                                .unwrap_or_else(|e| {
                                        log::error!("Failed to initialize EngineState: {:?}", e);
                                        panic!("Failed to initialize EngineState");
                                }));
                }

                #[cfg(target_arch = "wasm32")]
                {
                        // In WASM builds, async tasks must be spawned without blocking.
                        #[cfg(target_arch = "wasm32")]
                        if let Some(proxy) = self.proxy.take()
                        {
                                wasm_bindgen_futures::spawn_local(async move {
                                        let state_result =
                                                EngineState::new(window, model_map).await;
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

                        state.build_pipelines();

                        state.build_passes();
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

                        state.build_pipelines();

                        state.build_passes();
                }

                self.resize();
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
                        .renderer
                        .handle_input(&self.window.as_ref().unwrap(), &event);

                let last = self.last_render_time;

                match event
                {
                        WindowEvent::CloseRequested =>
                        {
                                event_loop.exit();
                        }
                        WindowEvent::Resized(_size) =>
                        {
                                self.resize();
                        }
                        WindowEvent::RedrawRequested =>
                        {
                                let last_render_time = instant::Instant::now();

                                match self.render(&last)
                                {
                                        Ok(_) =>
                                        {
                                                let window: Arc<Window> = match self.window.as_ref()
                                                {
                                                        None => return,
                                                        Some(w) => w.clone(),
                                                };

                                                window.request_redraw();

                                                let now = instant::Instant::now();

                                                self.last_render_time = now - last_render_time;
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
                                state.camera
                                        .controller
                                        .handle_key(code, key_state.is_pressed());

                                if code == KeyCode::Escape && key_state.is_pressed()
                                {
                                        state.camera.locked_in = !state.camera.locked_in;
                                }

                                match self.config.debug_toggle_key
                                {
                                        None =>
                                        {}
                                        Some(k) =>
                                        {
                                                if code as u32 == k
                                                        && key_state == ElementState::Pressed
                                                {
                                                        self.config.enable_debug =
                                                                !self.config.enable_debug;
                                                }
                                        }
                                }
                        }

                        _ => (),
                }
        }

        fn device_event(
                &mut self,
                _event_loop: &ActiveEventLoop,
                _device_id: DeviceId,
                event: DeviceEvent,
        )
        {
                let state = if let Some(state) = &mut self.state
                {
                        state
                }
                else
                {
                        return;
                };
                match event
                {
                        DeviceEvent::MouseMotion {
                                delta: (dx, dy),
                        } =>
                        {
                                if state.camera.locked_in
                                {
                                        state.camera.controller.handle_mouse(dx, dy);
                                }
                        }
                        _ =>
                        {}
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
                let config = Config::new();

                crate::resources::load_resources();

                let model_map = HashMap::new();

                Self {
                        engine: Engine {
                                #[cfg(target_arch = "wasm32")]
                                proxy: None,
                                last_render_time: Duration::from_secs_f32(0.0),
                                config,
                                model_map,
                                state: None,
                                time: None,
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

        /// Render a Debug GUI using `egui`.
        pub fn with_debug_ui(mut self) -> Self
        {
                self.engine.config.enable_debug = true;
                self
        }

        pub fn with_toggle(
                mut self,
                key_code: KeyCode,
        ) -> Result<Self>
        {
                if !self.engine.config.enable_debug
                {
                        anyhow::bail!("with_toggle: Debug UI must be enabled first");
                }

                self.engine.config.debug_toggle_key = Some(key_code as u32);

                Ok(self)
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
        pub fn build(self) -> Result<Engine>
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
        ) -> Result<wgpu::Surface<'a>, wgpu::CreateSurfaceError>
        {
                instance.create_surface(window)
        }

        async fn adapter(
                instance: &wgpu::Instance,
                window: Arc<Window>,
        ) -> Result<wgpu::Adapter>
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
        ) -> Result<(wgpu::Device, wgpu::Queue), wgpu::RequestDeviceError>
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
}
