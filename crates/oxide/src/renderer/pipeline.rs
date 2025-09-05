use crate::engine::FillMode;
use crate::model::Vertex;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub enum PipelineKind
{
        Geometry,
        Texture,
        Lighting,
        PostProcess,
}

#[derive(Debug)]
pub struct PipelineManager
{
        pub render_pipelines: HashMap<PipelineKind, wgpu::RenderPipeline>,
}

impl PipelineManager
{
        pub fn new() -> Self
        {
                let map: HashMap<PipelineKind, wgpu::RenderPipeline> = HashMap::new();

                Self {
                        render_pipelines: map,
                }
        }

        pub fn get(
                &self,
                kind: PipelineKind,
        ) -> &wgpu::RenderPipeline
        {
                self.render_pipelines
                        .get(&kind)
                        .expect("Pipeline not found")
        }

        pub fn get_mut(
                &mut self,
                kind: PipelineKind,
        ) -> &mut wgpu::RenderPipeline
        {
                self.render_pipelines
                        .get_mut(&kind)
                        .expect("Pipeline not found")
        }

        /// Loads the shader module data from the `wgsl` file.
        pub fn load_shader_module(device: &wgpu::Device) -> wgpu::ShaderModule
        {
                device.create_shader_module(wgpu::ShaderModuleDescriptor {
                        label: Some("Shader"),
                        source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
                })
        }

        pub fn new_render_pipeline_layout(
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

        pub fn build_geometry_pipeline(
                &mut self,
                device: &wgpu::Device,
                config: &wgpu::SurfaceConfiguration,
                bind_groups: &[&wgpu::BindGroupLayout],
                fill_mode: &FillMode,
        )
        {
                let polygon_mode = match fill_mode
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
                                        wgpu::PolygonMode::Fill
                                }
                        }
                        FillMode::Vertex => wgpu::PolygonMode::Point,
                };

                let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                        label: Some("Geometry Shader"),
                        source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
                });

                let render_pipeline_layout =
                        device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                                label: Some("Geometry Pipeline Layout"),
                                bind_group_layouts: bind_groups,
                                push_constant_ranges: &[],
                        });

                let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                        label: Some("Geometry Pipeline"),
                        layout: Some(&render_pipeline_layout),
                        vertex: wgpu::VertexState {
                                module: &shader,
                                entry_point: Some("vs_main"),
                                buffers: &[crate::model::ModelVertex::desc()],
                                compilation_options: wgpu::PipelineCompilationOptions::default(),
                        },
                        fragment: Some(wgpu::FragmentState {
                                module: &shader,
                                entry_point: Some("fs_main"),
                                targets: &[Some(wgpu::ColorTargetState {
                                        format: config.format,
                                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                                        write_mask: wgpu::ColorWrites::ALL,
                                })],
                                compilation_options: wgpu::PipelineCompilationOptions::default(),
                        }),
                        primitive: wgpu::PrimitiveState {
                                topology: wgpu::PrimitiveTopology::TriangleList,
                                strip_index_format: None,
                                front_face: wgpu::FrontFace::Ccw,
                                cull_mode: Some(wgpu::Face::Back),
                                polygon_mode,
                                conservative: false,
                                unclipped_depth: false,
                        },
                        depth_stencil: Some(wgpu::DepthStencilState {
                                format: crate::texture::Texture::DEPTH_FORMAT,
                                depth_write_enabled: true,
                                depth_compare: wgpu::CompareFunction::Less,
                                stencil: wgpu::StencilState::default(),
                                bias: wgpu::DepthBiasState::default(),
                        }),
                        multisample: wgpu::MultisampleState::default(),
                        multiview: None,
                        cache: None,
                });

                self.render_pipelines
                        .insert(PipelineKind::Geometry, pipeline);
        }
}
