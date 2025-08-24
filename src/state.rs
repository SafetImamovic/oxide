use crate::geometry::vertex::{INDICES, TRIANGLE, Vertex};
use crate::gui::GuiRenderer;
use crate::texture::Texture;
use cgmath::prelude::*;
use egui_wgpu::ScreenDescriptor;
use image::GenericImageView;
use std::sync::Arc;
use wgpu::Features;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use winit::dpi::PhysicalSize;
use winit::window::Window;

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
/// |       Oxide        |
/// +---------+----------+
///           |
///           v
/// +--------------------+
/// |     Instance       |
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
///           v                      |
/// +--------------------+           |
/// |      Adapter       |           |
/// +---------+----------+           |
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
pub struct State
{
        /// A thread-safe reference to the window.
        ///
        /// We want to store the window in a shared State and pass clones around
        /// without worrying about ownership.
        pub window: Arc<Window>,

        /// Handle to a rendering (graphics) pipeline.
        ///
        /// Reference <https://gpuweb.github.io/gpuweb/#render-pipeline>
        pub render_pipeline: wgpu::RenderPipeline,

        /// Handle to a presentable surface.
        ///
        /// This type is unique to the Rust API of wgpu. In the WebGPU
        /// specification, `GPUCanvasContext` serves a similar role.
        ///
        /// Reference: <https://www.w3.org/TR/webgpu/#canvas-rendering>
        pub surface: wgpu::Surface<'static>,

        /// Open connection to a graphics and/or compute device.
        ///
        /// A `GPUDevice` encapsulates a device and exposes its functionality.
        /// It is the top-level interface through which WebGPU interfaces are
        /// created.
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
        /// Contains surface format, usage flags, width, height, and present
        /// mode.
        ///
        /// Reference: <https://gpuweb.github.io/gpuweb/#canvas-configuration>
        pub config: wgpu::SurfaceConfiguration,

        /// Tracks if the surface has been configured yet.
        ///
        /// Rendering commands require a configured surface.
        pub is_surface_configured: bool,

        /// Handles the GUI.
        pub gui: GuiRenderer,

        /// Handles to GPU-accessible buffers.
        ///
        /// Corresponds to WebGPU GPUBuffer.
        ///
        /// Reference: <https://gpuweb.github.io/gpuweb/#buffer-interface>
        pub vertex_buffer: wgpu::Buffer,
        pub index_buffer: wgpu::Buffer,

        /// Total Index count.
        pub num_indices: u32,

        pub diffuse_bind_group: wgpu::BindGroup,

        pub diffuse_texture: crate::texture::Texture,

        pub camera: crate::camera::Camera,

        pub camera_uniform: crate::camera::CameraUniform,

        pub camera_buffer: wgpu::Buffer,

        pub camera_bind_group: wgpu::BindGroup,

        pub camera_controller: crate::camera::Controller,

        pub instances: Vec<crate::Instance>,

        pub instance_buffer: wgpu::Buffer,
}

const NUM_INSTANCES_PER_ROW: u32 = 10;
const INSTANCE_DISPLACEMENT: cgmath::Vector3<f32> = cgmath::Vector3::new(
        NUM_INSTANCES_PER_ROW as f32 * 0.5,
        0.0,
        NUM_INSTANCES_PER_ROW as f32 * 0.5,
);

impl State
{
        async fn get_adapter<'a>(
                instance: &wgpu::Instance,
                surface: &wgpu::Surface<'a>,
        ) -> anyhow::Result<wgpu::Adapter, wgpu::RequestAdapterError>
        {
                instance.request_adapter(&wgpu::RequestAdapterOptions {
                        // Either `HighPerformance` or `LowPower`.
                        //
                        // 1. LowPower will pick an adapter that favors battery life.
                        //
                        // 2. HighPerformance will pick an adapter for more power-hungry yet more
                        //    performant GPU's, such as a dedicated graphics card.
                        power_preference: wgpu::PowerPreference::HighPerformance,

                        // Tells wgpu to find an adapter that can present to the supplied
                        // surface.
                        compatible_surface: Some(surface),

                        // Forces wgpu to pick an adapter that will work on all hardware.
                        // Generally a software implementation on most systems.
                        force_fallback_adapter: false,
                })
                .await
        }

        /// Requests a connection to a physical device, creating a logical
        /// device.
        ///
        /// Returns the Device together with a Queue that executes command
        /// buffers.
        async fn get_device_and_queue(
                adapter: &wgpu::Adapter
        ) -> anyhow::Result<(wgpu::Device, wgpu::Queue), wgpu::RequestDeviceError>
        {
                adapter.request_device(&wgpu::DeviceDescriptor {
                        label: None,
                        required_features: Features::empty(),
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

        /// Logs the adapter features.
        ///
        /// Corresponds to these WebGPU feature Reference
        /// <https://gpuweb.github.io/gpuweb/#enumdef-gpufeaturename>
        pub fn log_adapter_features(adapter: &wgpu::Adapter)
        {
                adapter.features()
                        .iter()
                        .for_each(|f| log::info!("FEATURE: {}", f));
        }

        pub fn log_adapter_info(adapter: &wgpu::Adapter)
        {
                log::info!("Adapter Info: {:#?}", adapter.get_info());
        }

        pub fn log_device_info(device: &wgpu::Device)
        {
                log::info!("Device Info: {:#?}", device);
        }

        /// Represents different Display-Surface sync modes.
        ///
        /// For example, FiFo is essentially VSync.
        pub fn log_present_modes(surface_caps: &wgpu::SurfaceCapabilities)
        {
                surface_caps
                        .present_modes
                        .iter()
                        .for_each(|f| log::info!("PRESENT_MODE: {:?}", f));
        }

        fn get_surface_config(
                surface_format: wgpu::TextureFormat,
                size: &PhysicalSize<u32>,
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

        fn get_render_pipeline_layout(
                device: &wgpu::Device,
                bind_group_layout: &wgpu::BindGroupLayout,
                camera_bind_group_layout: &wgpu::BindGroupLayout,
        ) -> wgpu::PipelineLayout
        {
                device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                        label: Some("Render Pipeline Layout"),
                        bind_group_layouts: &[&bind_group_layout, &camera_bind_group_layout],
                        push_constant_ranges: &[],
                })
        }

        fn get_render_pipeline(
                device: &wgpu::Device,
                config: &wgpu::SurfaceConfiguration,
                bind_group_layout: &wgpu::BindGroupLayout,
                camera_bind_group_layout: &wgpu::BindGroupLayout,
        ) -> wgpu::RenderPipeline
        {
                let shader = Self::load_shader_module(device);

                let render_pipeline_layout = Self::get_render_pipeline_layout(
                        device,
                        bind_group_layout,
                        camera_bind_group_layout,
                );

                let vertex_buffer = Vertex::get_desc();

                device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                        label: Some("Render Pipeline"),
                        layout: Some(&render_pipeline_layout),
                        vertex: wgpu::VertexState {
                                module: &shader,
                                entry_point: Some("vs_main"), // 1.
                                buffers: &[vertex_buffer, crate::InstanceRaw::desc()], // 2.
                                compilation_options: wgpu::PipelineCompilationOptions::default(),
                        },
                        fragment: Some(wgpu::FragmentState {
                                // 3.
                                module: &shader,
                                entry_point: Some("fs_main"),
                                targets: &[Some(wgpu::ColorTargetState {
                                        // 4.
                                        format: config.format,
                                        blend: Some(wgpu::BlendState::REPLACE),
                                        write_mask: wgpu::ColorWrites::ALL,
                                })],
                                compilation_options: wgpu::PipelineCompilationOptions::default(),
                        }),
                        primitive: wgpu::PrimitiveState {
                                topology: wgpu::PrimitiveTopology::TriangleList, // 1.
                                strip_index_format: None,
                                front_face: wgpu::FrontFace::Ccw, // 2.
                                cull_mode: Some(wgpu::Face::Back),
                                // Setting this to anything other than Fill requires
                                // Features::NON_FILL_POLYGON_MODE
                                polygon_mode: wgpu::PolygonMode::Fill,
                                // Requires Features::DEPTH_CLIP_CONTROL
                                unclipped_depth: false,
                                // Requires Features::CONSERVATIVE_RASTERIZATION
                                conservative: false,
                        },
                        depth_stencil: None, // 1.
                        multisample: wgpu::MultisampleState {
                                count: 1,                         // 2.
                                mask: !0,                         // 3.
                                alpha_to_coverage_enabled: false, // 4.
                        },
                        multiview: None, // 5.
                        cache: None,     // 6.
                })
        }

        /// Loads the shader module data from the `wgsl` file.
        fn load_shader_module(device: &wgpu::Device) -> wgpu::ShaderModule
        {
                device.create_shader_module(wgpu::ShaderModuleDescriptor {
                        label: Some("Shader"),
                        source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
                })
        }

        fn log_all_backends()
        {
                let backends = wgpu::Backends::all();

                log::info!("Available backends: {:?}", backends);
        }

        fn log_current_backend(adapter: &wgpu::Adapter)
        {
                let backend = adapter.get_info().backend;

                log::info!("Current backend: {:?}", backend);
        }

        pub fn new_vertex_buffer<A>(
                device: &wgpu::Device,
                content: &[A],
        ) -> wgpu::Buffer
        where
                A: bytemuck::NoUninit,
        {
                device.create_buffer_init(&BufferInitDescriptor {
                        label: Some("Vertex Buffer"),
                        contents: bytemuck::cast_slice(content),
                        usage: wgpu::BufferUsages::VERTEX,
                })
        }

        pub fn new_index_buffer<A>(
                device: &wgpu::Device,
                content: &[A],
        ) -> wgpu::Buffer
        where
                A: bytemuck::NoUninit,
        {
                device.create_buffer_init(&BufferInitDescriptor {
                        label: Some("Index Buffer"),
                        contents: bytemuck::cast_slice(content),
                        usage: wgpu::BufferUsages::INDEX,
                })
        }

        /// Asynchronously creates a new [`State`] instance.
        ///
        /// Initializes rendering resources and prepares the engine
        /// for drawing.
        pub async fn new(window: Arc<Window>) -> anyhow::Result<State>
        {
                Self::log_all_backends();

                let size = window.inner_size();

                let instance = Self::new_instance();

                let surface = instance.create_surface(window.clone())?;

                let adapter = Self::get_adapter(&instance, &surface).await?;

                Self::log_adapter_info(&adapter);

                Self::log_current_backend(&adapter);

                let (device, queue) = Self::get_device_and_queue(&adapter).await?;

                let surface_caps = surface.get_capabilities(&adapter);

                let surface_format = surface_caps
                        .formats
                        .iter()
                        .find(|f| f.is_srgb())
                        .copied()
                        .unwrap_or(surface_caps.formats[0]);

                let config = Self::get_surface_config(surface_format, &size, &surface_caps);

                let diffuse_texture =
                        crate::texture::Texture::from_bytes(&device, &queue, "Texture")?;

                let texture_bind_group_layout =
                        crate::texture::Texture::new_texture_bind_group_layout(&device);

                let diffuse_bind_group = crate::texture::Texture::new_diffuse_bind_group(
                        &device,
                        &texture_bind_group_layout,
                        &diffuse_texture,
                );

                let gui = GuiRenderer::new(&device, config.format, None, 1.0, 1, &window);

                let vertex_buffer = Self::new_vertex_buffer(&device, TRIANGLE);

                let index_buffer = Self::new_index_buffer(&device, INDICES);

                let num_indices = INDICES.len() as u32;

                let camera = crate::camera::Camera {
                        eye: (0.0, 1.0, 2.0).into(),
                        target: (0.0, 0.0, 0.0).into(),
                        up: cgmath::Vector3::unit_y(),
                        aspect: 1.0,
                        fovy: 45.0,
                        znear: 0.1,
                        zfar: 100.0,
                };

                let mut camera_uniform = crate::camera::CameraUniform::new();

                camera_uniform.update_view_proj(&camera);

                let camera_buffer = camera_uniform.new_buffer(&device);

                let camera_bind_group_layout =
                        crate::camera::CameraUniform::new_bind_group_layout(&device);

                let camera_bind_group = crate::camera::CameraUniform::new_bind_group(
                        &device,
                        &camera_bind_group_layout,
                        &camera_buffer,
                );

                let camera_controller = crate::camera::Controller::new(0.01);

                let render_pipeline = Self::get_render_pipeline(
                        &device,
                        &config,
                        &texture_bind_group_layout,
                        &camera_bind_group_layout,
                );

                let instances = (0..NUM_INSTANCES_PER_ROW)
                        .flat_map(|z| {
                                (0..NUM_INSTANCES_PER_ROW).map(move |x| {
                                        let position = cgmath::Vector3 {
                                                x: x as f32,
                                                y: 0.0,
                                                z: z as f32,
                                        } - INSTANCE_DISPLACEMENT;

                                        let rotation = if position.is_zero()
                                        {
                                                // this is needed so an object at (0, 0, 0) won't
                                                // get scaled to zero
                                                // as Quaternions can affect scale if they're not
                                                // created correctly
                                                cgmath::Quaternion::from_axis_angle(
                                                        cgmath::Vector3::unit_z(),
                                                        cgmath::Deg(0.0),
                                                )
                                        }
                                        else
                                        {
                                                cgmath::Quaternion::from_axis_angle(
                                                        position.normalize(),
                                                        cgmath::Deg(45.0),
                                                )
                                        };

                                        crate::Instance {
                                                position,
                                                rotation,
                                        }
                                })
                        })
                        .collect::<Vec<_>>();

                let instance_data = instances
                        .iter()
                        .map(crate::Instance::to_raw)
                        .collect::<Vec<_>>();
                let instance_buffer =
                        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                                label: Some("Instance Buffer"),
                                contents: bytemuck::cast_slice(&instance_data),
                                usage: wgpu::BufferUsages::VERTEX,
                        });

                Ok(State {
                        camera,
                        camera_controller,
                        diffuse_texture,
                        num_indices,
                        window,
                        gui,
                        render_pipeline,
                        surface,
                        device,
                        queue,
                        config,
                        is_surface_configured: false,
                        vertex_buffer,
                        index_buffer,
                        diffuse_bind_group,
                        camera_uniform,
                        camera_buffer,
                        camera_bind_group,
                        instances,
                        instance_buffer,
                })
        }

        /// Instance of wgpu.
        ///
        /// Generates a [`wgpu::Instance`] which is a handle to our GPU.
        ///
        /// GPU ([`wgpu::Instance`]) is the entry point to `WebGPU`.
        /// Reference <https://gpuweb.github.io/gpuweb/#gpu-interface>
        ///
        /// Defined via [`wgpu::InstanceDecsriptor`], this represents Options
        /// for creating an instance. Reference <https://docs.rs/wgpu/latest/wgpu/struct.InstanceDescriptor.html>
        ///
        /// ```text
        /// BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU.
        /// ```
        fn new_instance() -> wgpu::Instance
        {
                wgpu::Instance::new(&wgpu::InstanceDescriptor {
                        #[cfg(not(target_arch = "wasm32"))]
                        backends: wgpu::Backends::PRIMARY,
                        #[cfg(target_arch = "wasm32")]
                        backends: wgpu::Backends::GL,
                        ..Default::default()
                })
        }

        /// Handles window resize events.
        ///
        /// # Parameters
        /// - `width`: New window width in pixels
        /// - `height`: New window height in pixels
        pub fn resize(
                &mut self,
                width: u32,
                height: u32,
        )
        {
                if width == 0 || height == 0
                {
                        return;
                }

                // Clamping to max dim to prevent panic!
                let max_dim = self.device.limits().max_texture_dimension_2d;
                let final_width = width.min(max_dim);
                let final_height = height.min(max_dim);

                log::info!("Resizing surface -> width: {}, height: {}", final_width, final_height);

                self.config.width = final_width;
                self.config.height = final_height;

                self.surface.configure(&self.device, &self.config);
                self.is_surface_configured = true;
        }

        /// Requests a redraw for the next frame.
        ///
        /// This method triggers a `RedrawRequested` event on the window,
        /// allowing the render loop to run again.
        pub fn render(
                &mut self,
                config: &mut crate::Config,
        ) -> Result<(), wgpu::SurfaceError>
        {
                // Request redraw first
                self.window.request_redraw();

                if !self.is_surface_configured
                {
                        return Ok(());
                }

                // Get the surface texture ONCE per frame
                let output = match self.surface.get_current_texture()
                {
                        Ok(frame) => frame,
                        Err(wgpu::SurfaceError::Outdated) =>
                        {
                                // This often happens during window resizing
                                println!("wgpu surface outdated");
                                return Err(wgpu::SurfaceError::Outdated);
                        }
                        Err(e) =>
                        {
                                eprintln!("Failed to acquire surface texture: {:?}", e);
                                return Err(e);
                        }
                };

                let view = output
                        .texture
                        .create_view(&wgpu::TextureViewDescriptor::default());

                let mut encoder =
                        self.device
                                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                                        label: Some("Main Render Encoder"),
                                });

                // 1. First render the background
                // Diabolical levels of indentation.
                {
                        let mut render_pass =
                                encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                                        label: Some("Background Pass"),
                                        color_attachments: &[Some(
                                                wgpu::RenderPassColorAttachment {
                                                        view: &view,
                                                        resolve_target: None,
                                                        ops: wgpu::Operations {
                                                                load: wgpu::LoadOp::Clear(
                                                                        wgpu::Color {
                                                                                r: 0.0,
                                                                                g: 0.0,
                                                                                b: 0.0,
                                                                                a: 1.0,
                                                                        },
                                                                ),
                                                                store: wgpu::StoreOp::Store,
                                                        },
                                                },
                                        )],
                                        depth_stencil_attachment: None,
                                        occlusion_query_set: None,
                                        timestamp_writes: None,
                                });

                        render_pass.set_pipeline(&self.render_pipeline);

                        render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);

                        render_pass.set_bind_group(1, &self.camera_bind_group, &[]);

                        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));

                        render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
                        render_pass.set_index_buffer(
                                self.index_buffer.slice(..),
                                wgpu::IndexFormat::Uint16,
                        );

                        render_pass.draw_indexed(
                                0..self.num_indices,
                                0,
                                0..self.instances.len() as _,
                        );
                }

                let scale = self.window.as_ref().scale_factor() as f32;

                let screen_descriptor = ScreenDescriptor {
                        size_in_pixels: [self.config.width, self.config.height],
                        pixels_per_point: 1.0 / scale, /* inversely counteracts the
                                                        * Browser DPI */
                };

                {
                        self.gui.begin_frame(&self.window.clone(), config);

                        self.gui.render(config);

                        self.gui.end_frame_and_draw(
                                &self.device,
                                &self.queue,
                                &mut encoder,
                                &self.window.clone(),
                                &view,
                                screen_descriptor,
                        );
                }

                self.queue.submit(std::iter::once(encoder.finish()));

                output.present();

                Ok(())
        }

        pub fn update(&mut self)
        {
                self.camera_controller.update_camera(&mut self.camera);

                self.camera.aspect = self.config.width as f32 / self.config.height as f32;

                self.camera_uniform.update_view_proj(&self.camera);

                self.queue.write_buffer(
                        &self.camera_buffer,
                        0,
                        bytemuck::cast_slice(&[self.camera_uniform]),
                );
        }
}
