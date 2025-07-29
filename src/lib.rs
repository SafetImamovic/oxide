use std::sync::Arc;
use winit::{
        application::ApplicationHandler,
        event::*,
        event_loop::{ActiveEventLoop, EventLoop},
        keyboard::{KeyCode, PhysicalKey},
        window::Window,
};

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

/// Represents the rendering state of the application.
///
/// This struct holds references to key rendering resources and manages
/// per-frame updates, including window resizing and surface configuration.
///
/// It encapsulates the lifecycle of the WebGPU rendering context,
/// coordinating the surface, device, queue, and configuration.
///
/// # Rendering Flow Overview
///
/// The rendering pipeline roughly follows this flow:
///
/// ```text
/// +--------------------+
/// |     Application    |
/// |     (your code)    |
/// +---------+----------+
///           |
///           v
/// +--------------------+   create
/// |      Surface       |<----------+
/// |  (Canvas Context)  |           |
/// +---------+----------+           |
///           |                      |
///           |  configure           |
///           v                      |
/// +--------------------+           |
/// |   SurfaceConfig    |           |
/// +---------+----------+           |
///           |                      |
///           |                      |
///           v                      |
/// +--------------------+   create  |
/// |      Device        |>----------+
/// |   (GPU Interface)  |
/// +---------+----------+
///           |
///           | submit commands
///           v
/// +--------------------+
/// |       Queue        |
/// |  (Command Buffer)  |
/// +---------+----------+
///           |
///           v
/// +--------------------+
/// |        GPU         |
/// | (Render & Compute) |
/// +---------+----------+
///           |
///           v
/// +---------+----------+
/// |    Framebuffer /   |
/// |    Canvas Output   |
/// +--------------------+
/// ```
///
/// # Example
///
/// ```
/// use std::sync::Arc;
///
/// async fn create_state_example() -> anyhow::Result<()> {
///     let window = Arc::new(Window::new().unwrap());
///     let state = State::new(window).await?;
///     Ok(())
/// }
///
/// let x = 1;
/// let y = 2;
///
/// assert_eq!(3, x + y);
/// ```
pub struct State
{
        /// A thread-safe reference to the window.
        pub window: Arc<Window>,

        /// Handle to a presentable surface.
        ///
        /// This type is unique to the Rust API of wgpu. In the WebGPU specification,
        /// `GPUCanvasContext` serves a similar role.
        ///
        /// A `GPUCanvasContext` object is created via the `getContext()` method of an
        /// `HTMLCanvasElement` instance by passing the string literal `'webgpu'` as its
        /// contextType argument.
        ///
        /// Example (WebGPU Spec in JavaScript):
        /// ```no_run
        /// const canvas = document.createElement('canvas');
        /// const context = canvas.getContext('webgpu');
        /// ```
        ///
        /// Reference: <https://www.w3.org/TR/webgpu/#canvas-rendering>
        pub surface: wgpu::Surface<'static>,

        /// Open connection to a graphics and/or compute device.
        ///
        /// A `GPUDevice` encapsulates a device and exposes its functionality.
        /// It is the top-level interface through which WebGPU interfaces are created.
        ///
        /// Reference: <https://gpuweb.github.io/gpuweb/#gpu-device>
        pub device: wgpu::Device,

        /// Handle to a command queue on a device.
        ///
        /// Used to submit commands for execution.
        ///
        /// Reference: <https://gpuweb.github.io/gpuweb/#gpu-queue>
        pub queue: wgpu::Queue,

        /// Describes a Surface configuration.
        ///
        /// Contains surface format, usage flags, width, height, and present mode.
        ///
        /// Reference: <https://gpuweb.github.io/gpuweb/#canvas-configuration>
        pub config: wgpu::SurfaceConfiguration,

        /// Tracks if the surface has been configured yet.
        ///
        /// Rendering commands require a configured surface.
        pub is_surface_configured: bool,
}

impl State
{
        /// Asynchronously creates a new [`State`] instance.
        ///
        /// Initializes rendering resources and prepares the engine
        /// for drawing.
        async fn new(window: Arc<Window>) -> anyhow::Result<State>
        {
                let size = window.inner_size();

                // The instance is a handle to our GPU
                // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
                let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
                        #[cfg(not(target_arch = "wasm32"))]
                        backends: wgpu::Backends::PRIMARY,
                        #[cfg(target_arch = "wasm32")]
                        backends: wgpu::Backends::GL,
                        ..Default::default()
                });

                let surface = instance.create_surface(window.clone()).unwrap();

                let adapter = instance
                .request_adapter(&wgpu::RequestAdapterOptions {
                        power_preference: wgpu::PowerPreference::default(),
                        compatible_surface: Some(&surface),
                        force_fallback_adapter: false,
                    })
                .await?;
        }

        /// Handles window resize events.
        ///
        /// # Parameters
        /// - `width`: New window width in pixels
        /// - `height`: New window height in pixels
        pub fn resize(&mut self, _width: u32, _height: u32) {}

        /// Requests a redraw for the next frame.
        ///
        /// This method triggers a `RedrawRequested` event on the window,
        /// allowing the render loop to run again.
        pub fn render(&mut self)
        {
                self.window.request_redraw();
        }
}

/// Main application struct.
///
/// `App` is responsible for:
/// - Managing the application state
/// - Handling platform-specific event loops
/// - Dispatching window and user events
pub struct App
{
        /// On browser environments, an [`EventLoopProxy`] is needed
        /// to send events back into the event loop asynchronously.
        #[cfg(target_arch = "wasm32")]
        proxy: Option<winit::event_loop::EventLoopProxy<State>>,

        /// The rendering state of the application.
        state: Option<State>,
}

impl App
{
        /// Creates a new [`App`] instance.
        ///
        /// # Platform differences
        /// - On native builds, the event loop is created without a proxy.
        /// - On `wasm32`, a proxy is created to allow async initialization.
        pub fn new(#[cfg(target_arch = "wasm32")] event_loop: &EventLoop<State>) -> Self
        {
                #[cfg(target_arch = "wasm32")]
                let proxy = Some(event_loop.create_proxy());

                Self { state: None,
                       #[cfg(target_arch = "wasm32")]
                       proxy }
        }
}

impl ApplicationHandler<State> for App
{
        /// Called when the application is resumed.
        ///
        /// Creates the window and initializes the rendering state.
        fn resumed(&mut self, event_loop: &ActiveEventLoop)
        {
                #[allow(unused_mut)]
                let mut window_attributes = Window::default_attributes();

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

                #[cfg(not(target_arch = "wasm32"))]
                {
                        // Native builds can block on async state initialization.
                        self.state = Some(pollster::block_on(State::new(window)).unwrap());
                }

                #[cfg(target_arch = "wasm32")]
                {
                        // In WASM builds, async tasks must be spawned without blocking.
                        if let Some(proxy) = self.proxy.take()
                        {
                                wasm_bindgen_futures::spawn_local(async move {
                                        assert!(proxy
                        .send_event(
                            State::new(window)
                                .await
                                .expect("Unable to create canvas!")
                        )
                        .is_ok())
                                });
                        }
                }
        }

        /// Handles custom user events.
        ///
        /// On WASM, async initialization sends the completed [`State`] via a proxy,
        /// which is received here and stored.
        fn user_event(&mut self, _event_loop: &ActiveEventLoop, mut event: State)
        {
                #[cfg(target_arch = "wasm32")]
                {
                        event.window.request_redraw();
                        event.resize(event.window.inner_size().width,
                                     event.window.inner_size().height);
                }
                self.state = Some(event);
        }

        /// Handles native window events.
        ///
        /// This includes:
        /// - Closing the window
        /// - Resizing
        /// - Redraw requests
        /// - Keyboard input
        fn window_event(&mut self,
                        event_loop: &ActiveEventLoop,
                        _window_id: winit::window::WindowId,
                        event: WindowEvent)
        {
                let state = match &mut self.state
                {
                        Some(canvas) => canvas,
                        None => return,
                };

                match event
                {
                        WindowEvent::CloseRequested => event_loop.exit(),
                        WindowEvent::Resized(size) => state.resize(size.width, size.height),
                        WindowEvent::RedrawRequested => state.render(),
                        WindowEvent::KeyboardInput { event:
                                                             KeyEvent { physical_key:
                                                                                PhysicalKey::Code(code),
                                                                        state,
                                                                        .. },
                                                     .. } =>
                        {
                                log::info!("Key pressed: {:?}", code);

                                if let (KeyCode::Escape, true) = (code, state.is_pressed())
                                {
                                        event_loop.exit()
                                }
                        }
                        _ =>
                        {}
                }
        }
}

/// Starts the application in native or WASM environments.
///
/// # Returns
/// - `Ok(())` if the application exits successfully.
/// - An error if initialization fails.
///
/// # Example
/// ```no_run
/// fn main() -> anyhow::Result<()> {
///     run()
/// }
/// ```
pub fn run() -> anyhow::Result<()>
{
        #[cfg(not(target_arch = "wasm32"))]
        env_logger::init();

        #[cfg(target_arch = "wasm32")]
        console_log::init_with_level(log::Level::Info).unwrap_throw();

        let event_loop = EventLoop::with_user_event().build()?;
        let mut app = App::new(#[cfg(target_arch = "wasm32")]
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
