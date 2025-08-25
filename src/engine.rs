use std::sync::Arc;

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
pub struct Engine
{
        // --- Core Context ---
        /// The OS/Browser window for rendering and input handling.
        pub window: Option<Arc<winit::window::Window>>,

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

        /// The main camera used to view the scene.
        pub camera: Option<crate::scene::camera::Camera>,

        // --- Misc ---
        /// Input system state (keyboard, mouse, gamepad, etc.).
        pub input: Option<crate::input::InputState>,

        /// Optional UI system (e.g., egui) for rendering overlays.
        pub ui: Option<crate::ui::UiSystem>,
}

pub struct EngineState
{
        pub instance: wgpu::Instance,

        /// The handle to a physical graphics device.
        pub adapter: wgpu::Adapter,

        /// The rendering surface tied to the window.
        pub surface: wgpu::Surface<'static>,

        /// The GPU device handle used to submit rendering commands.
        pub device: wgpu::Device,

        /// The GPU queue used to execute command buffers.
        pub queue: wgpu::Queue,
}

impl EngineState
{
        pub fn new(window: Arc<winit::window::Window>) -> Self
        {
                let instance = EngineBuilder::instance();

                let surface = instance.create_surface(window.clone()).unwrap();

                let adapter = pollster::block_on(EngineBuilder::adapter(&instance, window.clone()))
                        .unwrap();

                let (device, queue) =
                        pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
                                label: Some("device_queue"),
                                required_features: wgpu::Features::default(),
                                required_limits: wgpu::Limits::default(),
                                memory_hints: wgpu::MemoryHints::Performance,
                                trace: wgpu::Trace::Off,
                        }))
                        .unwrap();

                EngineState {
                        instance,
                        adapter,
                        surface,
                        device,
                        queue,
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
                let window = Arc::new(
                        event_loop
                                .create_window(winit::window::Window::default_attributes())
                                .unwrap(),
                );

                self.window = Some(window.clone());

                let state = EngineState::new(window.clone());

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

/// Builder for [`Engine`].
///
/// Returns a result instance of `anyhow::Result<Engine>` after calling
/// `build()` because of field validation.
pub struct EngineBuilder
{
        engine: Engine,
}

#[allow(clippy::new_without_default)]
impl EngineBuilder
{
        /// Initializes a new build process for [`Engine`].
        ///
        /// Every field is set to `None`.
        ///
        /// # Returns
        ///
        /// `EngineBuilder`.
        pub fn new() -> Self
        {
                Self {
                        engine: Engine {
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
}
