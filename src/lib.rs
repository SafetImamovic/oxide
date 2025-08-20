pub mod app;
pub mod config;
pub mod gui;
pub mod state;

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
use winit::platform::web::EventLoopExtWebSys;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use crate::{app::App, config::Config};
use winit::event_loop::EventLoop;

/// Starts the application in native or WASM environments.
pub fn run() -> anyhow::Result<()>
{
        log::info!("Oxide initialized.");

        #[cfg(not(target_arch = "wasm32"))]
        {
                env_logger::init();

                log::info!("Running on native.");
        }

        #[cfg(target_arch = "wasm32")]
        {
                console_log::init_with_level(log::Level::Info).unwrap_throw();

                log::info!("Running on wasm32.");
        }

        let event_loop = EventLoop::with_user_event().build()?;

        let config = Config::from_file().unwrap_or_else(|err| {
                log::warn!("Failed to load config: {err}, falling back to default");
                Config::default()
        });

        #[allow(unused_mut)]
        let mut app = App::new(
                config,
                #[cfg(target_arch = "wasm32")]
                &event_loop,
        );

        #[cfg(target_arch = "wasm32")]
        event_loop.spawn_app(Box::leak(Box::new(app)));

        #[cfg(not(target_arch = "wasm32"))]
        event_loop.run_app(&mut app)?;

        let msg = get_exit_message();

        log::info!("{msg}");

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

/// Vertex struct.
///
/// Uses C-compatible memory layout (`#[repr(C)]`)
/// so it can be safely shared with GPU graphics APIs.
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex
{
        pub position: [f32; 3],
        pub tex_coords: [f32; 2],
}

const TRIANGLE: &[Vertex] = &[
        Vertex {
                position: [-0.0868241, 0.49240386, 0.0],
                tex_coords: [0.4131759, 0.00759614],
        }, // A
        Vertex {
                position: [-0.49513406, 0.06958647, 0.0],
                tex_coords: [0.0048659444, 0.43041354],
        }, // B
        Vertex {
                position: [-0.21918549, -0.44939706, 0.0],
                tex_coords: [0.28081453, 0.949397],
        }, // C
        Vertex {
                position: [0.35966998, -0.3473291, 0.0],
                tex_coords: [0.85967, 0.84732914],
        }, // D
        Vertex {
                position: [0.44147372, 0.2347359, 0.0],
                tex_coords: [0.9414737, 0.2652641],
        }, // E
];

const INDICES: &[u16] = &[0, 1, 4, 1, 2, 4, 2, 3, 4];

const SQUARE: &[Vertex] = &[
        Vertex {
                position: [-0.5, 0.5, 0.0],
                tex_coords: [0.0, 0.0],
        }, // A
        Vertex {
                position: [-0.5, -0.5, 0.0],
                tex_coords: [0.0, 1.0],
        }, // B
        Vertex {
                position: [0.5, -0.5, 0.0],
                tex_coords: [1.0, 1.0],
        }, // C
        Vertex {
                position: [0.5, 0.5, 0.0],
                tex_coords: [1.0, 0.0],
        }, // D
];

const SQ_INDICES: &[u16] = &[0, 1, 2, 0, 2, 3];

const SQUARE_2: &[Vertex] = &[
        Vertex {
                position: [-0.5, 0.5, 0.0],
                tex_coords: [0.0, 0.0],
        }, // A
        Vertex {
                position: [-0.5, -0.5, 0.0],
                tex_coords: [1.0, 0.0],
        }, // B
        Vertex {
                position: [0.5, -0.5, 0.0],
                tex_coords: [1.0, 1.0],
        }, // C
        Vertex {
                position: [0.5, 0.5, 0.0],
                tex_coords: [0.0, 1.0],
        }, // D
];

const SQ_INDICES_2: &[u16] = &[0, 1, 2, 0, 2, 3];

impl Vertex
{
        pub fn get_desc() -> wgpu::VertexBufferLayout<'static>
        {
                wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &[
                                wgpu::VertexAttribute {
                                        offset: 0,
                                        shader_location: 0,
                                        format: wgpu::VertexFormat::Float32x3,
                                },
                                wgpu::VertexAttribute {
                                        offset: std::mem::size_of::<[f32; 3]>()
                                                as wgpu::BufferAddress,
                                        shader_location: 1,
                                        format: wgpu::VertexFormat::Float32x2,
                                },
                        ],
                }
        }
}

use colored::*;
use rand::{
        rng,
        seq::{IndexedRandom, SliceRandom},
};

fn get_exit_message() -> String
{
        let messages = [
                ("Oxide has been reduced to its elemental components.", "red"),
                ("Oxide underwent catastrophic reduction and ceased to exist.", "magenta"),
                ("Oxide has been violently reduced back to base metal.", "yellow"),
                ("Oxide experienced spontaneous deoxygenation and met its end.", "cyan"),
                ("Oxide's oxidation state was permanently set to zero.", "green"),
                ("Oxide was stripped of its oxygen atoms and left for dead.", "blue"),
                ("Oxide underwent irreversible reduction at room temperature.", "purple"),
                ("Oxide's reaction has reached equilibrium... with the void.", "white"),
                ("Oxide's half-life has expired.", "bright red"),
                ("Oxide has been oxidized out of existence.", "bright yellow"),
        ];

        let mut rng = rng();

        let (message, color) = messages.choose(&mut rng).unwrap();

        match *color
        {
                "red" => message.red().to_string(),
                "magenta" => message.magenta().to_string(),
                "yellow" => message.yellow().to_string(),
                "cyan" => message.cyan().to_string(),
                "green" => message.green().to_string(),
                "blue" => message.blue().to_string(),
                "purple" => message.purple().to_string(),
                "white" => message.white().to_string(),
                "bright red" => message.bright_red().to_string(),
                "bright yellow" => message.bright_yellow().to_string(),
                _ => message.to_string(),
        }
}
