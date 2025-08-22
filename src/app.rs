use std::sync::Arc;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
use winit::event_loop::EventLoop;
use winit::{
        application::ApplicationHandler,
        event::{KeyEvent, MouseButton, WindowEvent},
        event_loop::ActiveEventLoop,
        keyboard::{KeyCode, PhysicalKey},
        window::Window,
};

use crate::{
        config::Config,
        geometry::vertex::{INDICES, SQ_INDICES, SQ_INDICES_2, SQUARE, SQUARE_2, TRIANGLE},
        state::State,
};

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
        pub proxy: Option<winit::event_loop::EventLoopProxy<State>>,

        /// The rendering state of the application.
        pub state: Option<State>,

        /// Configuration options for the `App`.
        pub config: Config,
}

impl App
{
        /// Creates a new [`App`] instance.
        ///
        /// # Platform differences
        /// - On native builds, the event loop is created without a proxy.
        /// - On `wasm32`, a proxy is created to allow async initialization.
        pub fn new(
                config: Config,
                #[cfg(target_arch = "wasm32")] event_loop: &EventLoop<State>,
        ) -> Self
        {
                #[cfg(target_arch = "wasm32")]
                let proxy = Some(event_loop.create_proxy());

                Self {
                        state: None,
                        config,
                        #[cfg(target_arch = "wasm32")]
                        proxy,
                }
        }

        fn resize(&mut self)
        {
                let state = match &mut self.state
                {
                        Some(canvas) => canvas,
                        None => return,
                };

                #[cfg(target_arch = "wasm32")]
                {
                        let (width, height) = crate::get_body_size().unwrap();
                        state.resize(width, height);
                }

                #[cfg(not(target_arch = "wasm32"))]
                {
                        let size = state.window.inner_size();
                        state.resize(size.width, size.height);
                }
        }

        pub fn handle_key(
                &mut self,
                event_loop: &ActiveEventLoop,
                code: KeyCode,
                is_pressed: bool,
        )
        {
                log::info!("{:#?}", code);

                if let (KeyCode::Escape, true) = (code, is_pressed)
                {
                        log::info!("Oxide Render Engine Exiting. bye!");

                        event_loop.exit()
                }

                if let (KeyCode::Digit1, true) = (code, is_pressed)
                {
                        log::info!("Switch to Pentagon!");

                        self.state.as_mut().unwrap().vertex_buffer = State::new_vertex_buffer(
                                &self.state.as_ref().unwrap().device,
                                TRIANGLE,
                        );

                        self.state.as_mut().unwrap().index_buffer = State::new_index_buffer(
                                &self.state.as_ref().unwrap().device,
                                INDICES,
                        );

                        self.state.as_mut().unwrap().num_indices = INDICES.len() as u32;
                }

                if let (KeyCode::Digit2, true) = (code, is_pressed)
                {
                        log::info!("Switch to Square!");

                        self.state.as_mut().unwrap().vertex_buffer = State::new_vertex_buffer(
                                &self.state.as_ref().unwrap().device,
                                SQUARE,
                        );

                        self.state.as_mut().unwrap().index_buffer = State::new_index_buffer(
                                &self.state.as_ref().unwrap().device,
                                SQ_INDICES,
                        );

                        self.state.as_mut().unwrap().num_indices = SQ_INDICES.len() as u32;
                }

                if let (KeyCode::Digit3, true) = (code, is_pressed)
                {
                        log::info!("Switch to Square!");

                        self.state.as_mut().unwrap().vertex_buffer = State::new_vertex_buffer(
                                &self.state.as_ref().unwrap().device,
                                SQUARE_2,
                        );

                        self.state.as_mut().unwrap().index_buffer = State::new_index_buffer(
                                &self.state.as_ref().unwrap().device,
                                SQ_INDICES_2,
                        );

                        self.state.as_mut().unwrap().num_indices = SQ_INDICES_2.len() as u32;
                }
        }
}

impl ApplicationHandler<State> for App
{
        /// Called when the application is resumed.
        ///
        /// Creates the window and initializes the rendering state.
        fn resumed(
                &mut self,
                event_loop: &ActiveEventLoop,
        )
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
        /// On WASM, async initialization sends the completed [`State`] via a
        /// proxy, which is received here and stored.
        fn user_event(
                &mut self,
                _event_loop: &ActiveEventLoop,
                event: State,
        )
        {
                #[cfg(target_arch = "wasm32")]
                {
                        event.window.request_redraw();
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
        fn window_event(
                &mut self,
                event_loop: &ActiveEventLoop,
                _window_id: winit::window::WindowId,
                event: WindowEvent,
        )
        {
                let state = match &mut self.state
                {
                        Some(canvas) => canvas,
                        None => return,
                };

                state.camera_controller.process_events(&event);

                state.gui.handle_input(&state.window.clone(), &event);

                match event
                {
                        WindowEvent::CloseRequested => event_loop.exit(),
                        WindowEvent::Resized(_size) => self.resize(),
                        WindowEvent::RedrawRequested =>
                        {
                                state.update();

                                match state.render(&mut self.config)
                                {
                                        Ok(_) =>
                                        {}
                                        // Reconfigure the surface if it's lost or outdated
                                        Err(
                                                wgpu::SurfaceError::Lost
                                                | wgpu::SurfaceError::Outdated,
                                        ) =>
                                        {
                                                self.resize();
                                        }
                                        Err(e) =>
                                        {
                                                log::error!("Unable to render {}", e);
                                        }
                                }
                        }
                        WindowEvent::MouseInput {
                                state,
                                button,
                                ..
                        } => match (button, state.is_pressed())
                        {
                                (MouseButton::Left, true) =>
                                {}
                                (MouseButton::Left, false) =>
                                {}
                                _ =>
                                {}
                        },
                        WindowEvent::KeyboardInput {
                                event:
                                        KeyEvent {
                                                physical_key: PhysicalKey::Code(code),
                                                state: key_state,
                                                ..
                                        },
                                ..
                        } => self.handle_key(event_loop, code, key_state.is_pressed()),

                        _ =>
                        {}
                }
        }
}
