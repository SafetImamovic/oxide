use std::sync::Arc;

use winit::{
        application::ApplicationHandler,
        event::WindowEvent,
        event_loop::{ActiveEventLoop, EventLoop},
        window::WindowId,
};

/// Runner for the [`Engine`].
pub struct EngineRunner<'a>
{
        pub engine: Engine<'a>,
        pub event_loop: EventLoop<()>,
}

impl<'a> EngineRunner<'a>
{
        /// Constructor for [`EnginerRunner`].
        ///
        /// Most importantly, it creates the `event_loop` from
        /// `winit::EventLoop`.
        ///
        /// # Returns
        ///
        /// `anyhow::Result<EngineRunner>`.
        pub fn new(engine: Engine<'a>) -> anyhow::Result<Self>
        {
                let event_loop = winit::event_loop::EventLoop::with_user_event().build()?;

                Ok(Self {
                        engine,
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
        pub fn run(mut self) -> anyhow::Result<()>
        {
                self.event_loop.run_app(&mut self.engine)?;
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
pub struct Engine<'a>
{
        // --- Core Context ---
        /// The OS/Browser window for rendering and input handling.
        pub window: Option<Arc<winit::window::Window>>,

        /// The WGPU instance, required to create surfaces and devices.
        pub instance: Option<wgpu::Instance>,

        /// The rendering surface tied to the window.
        pub surface: Option<wgpu::Surface<'a>>,

        /// The GPU device handle used to submit rendering commands.
        pub device: Option<wgpu::Device>,

        /// The GPU queue used to execute command buffers.
        pub queue: Option<wgpu::Queue>,

        /// The surface configuration (swapchain) for rendering.
        pub config: Option<wgpu::SurfaceConfiguration>,

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

impl<'a> ApplicationHandler for Engine<'a>
{
        fn resumed(
                &mut self,
                event_loop: &winit::event_loop::ActiveEventLoop,
        )
        {
                self.window = Some(Arc::new(
                        event_loop
                                .create_window(winit::window::Window::default_attributes())
                                .unwrap(),
                ));
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
                        _ => (),
                }
        }
}

/// Builder for [`Engine`].
///
/// Returns a result instance of `anyhow::Result<Engine>` after calling
/// `build()` because of field validation.
pub struct EngineBuilder<'a>
{
        engine: Engine<'a>,
}

#[allow(clippy::new_without_default)]
impl<'a> EngineBuilder<'a>
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
                                window: None,
                                instance: None,
                                surface: None,
                                device: None,
                                queue: None,
                                config: None,
                                time: None,
                                render_ctx: None,
                                scene: None,
                                camera: None,
                                input: None,
                                ui: None,
                        },
                }
        }

        /// Set the instance
        pub fn with_instance(
                mut self,
                instance: wgpu::Instance,
        ) -> Self
        {
                self.engine.instance = Some(instance);
                self
        }

        /// Set the surface
        pub fn with_surface(
                mut self,
                surface: wgpu::Surface<'a>,
        ) -> Self
        {
                self.engine.surface = Some(surface);
                self
        }

        /// Set the device
        pub fn with_device(
                mut self,
                device: wgpu::Device,
        ) -> Self
        {
                self.engine.device = Some(device);
                self
        }

        /// Set the queue
        pub fn with_queue(
                mut self,
                queue: wgpu::Queue,
        ) -> Self
        {
                self.engine.queue = Some(queue);
                self
        }

        /// Set the surface configuration
        pub fn with_config(
                mut self,
                config: wgpu::SurfaceConfiguration,
        ) -> Self
        {
                self.engine.config = Some(config);
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
        /// # Returns
        ///
        /// `anyhow::Result<Engine>`.
        pub fn build(self) -> anyhow::Result<Engine<'a>>
        {
                /*
                    if self.engine.window.is_none()
                    {
                            anyhow::bail!("Engine requires a window");
                    }
                    if self.engine.instance.is_none()
                    {
                            anyhow::bail!("Engine requires a WGPU instance");
                    }
                    if self.engine.surface.is_none()
                    {
                            anyhow::bail!("Engine requires a surface");
                    }
                    if self.engine.device.is_none()
                    {
                            anyhow::bail!("Engine requires a device");
                    }
                    if self.engine.queue.is_none()
                    {
                            anyhow::bail!("Engine requires a queue");
                    }
                    if self.engine.config.is_none()
                    {
                            anyhow::bail!("Engine requires a surface configuration");
                    }
                */

                Ok(self.engine)
        }
}
