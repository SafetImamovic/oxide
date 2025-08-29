use crate::{engine::FillMode, geometry::vertex::Vertex};

#[derive(Debug)]
pub struct PipelineManager
{
        pub render_pipeline: wgpu::RenderPipeline,
}

impl PipelineManager
{
        pub fn new(
                device: &wgpu::Device,
                config: &wgpu::SurfaceConfiguration,
                bind_groups: &[&wgpu::BindGroupLayout],
                depth_texture: &crate::texture::Texture,
                fill_mode: &FillMode,
        ) -> Self
        {
                Self {
                        render_pipeline: Self::render_pipeline(
                                device,
                                config,
                                bind_groups,
                                depth_texture,
                                fill_mode,
                        ),
                }
        }

        fn render_pipeline(
                device: &wgpu::Device,
                config: &wgpu::SurfaceConfiguration,
                bind_groups: &[&wgpu::BindGroupLayout],
                depth_texture: &crate::texture::Texture,
                fill_mode: &FillMode,
        ) -> wgpu::RenderPipeline
        {
                let polygon_mode = match &fill_mode
                {
                        FillMode::Fill => wgpu::PolygonMode::Fill,
                        FillMode::Wireframe =>
                        {
                                if device
                                        .features()
                                        .contains(wgpu::Features::POLYGON_MODE_LINE)
                                {
                                        wgpu::PolygonMode::Line
                                }
                                else
                                {
                                        wgpu::PolygonMode::Fill // fallback if unsupported
                                }
                        }
                        FillMode::Vertex => wgpu::PolygonMode::Point,
                };

                let shader = Self::load_shader_module(device);

                let render_pipeline_layout = Self::get_render_pipeline_layout(device, bind_groups);

                let vertex_buffer = Vertex::get_desc();

                device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                        label: Some("Render Pipeline"),
                        layout: Some(&render_pipeline_layout),
                        vertex: wgpu::VertexState {
                                module: &shader,
                                entry_point: Some("vs_main"), // 1.
                                buffers: &[vertex_buffer],    // 2.
                                compilation_options: wgpu::PipelineCompilationOptions::default(),
                        },
                        fragment: Some(wgpu::FragmentState {
                                // 3.
                                module: &shader,
                                entry_point: Some("fs_main"),
                                targets: &[Some(wgpu::ColorTargetState {
                                        // 4.
                                        format: config.format,
                                        blend: Some(wgpu::BlendState {
                                                color: wgpu::BlendComponent::OVER,
                                                alpha: wgpu::BlendComponent::OVER,
                                        }),
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
                                polygon_mode,
                                // Requires Features::DEPTH_CLIP_CONTROL
                                // Requires Features::CONSERVATIVE_RASTERIZATION
                                conservative: false,
                                unclipped_depth: false,
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

        fn get_render_pipeline_layout(
                device: &wgpu::Device,
                bind_groups: &[&wgpu::BindGroupLayout],
        ) -> wgpu::PipelineLayout
        {
                device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                        label: Some("Render Pipeline Layout"),
                        bind_group_layouts: bind_groups,
                        push_constant_ranges: &[],
                })
        }
}
