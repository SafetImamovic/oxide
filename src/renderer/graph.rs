use std::{
        any::Any,
        sync::{Arc, Mutex},
};

use derivative::Derivative;
use egui_wgpu::Renderer;

use crate::{engine::FillMode, geometry::vertex::Vertex, resource::Resources};

#[derive(Derivative)]
#[derivative(Debug)]
pub struct RenderGraph
{
        #[derivative(Debug = "ignore")]
        pub passes: Vec<Box<dyn RenderPass>>,
}

impl RenderGraph
{
        pub fn add_pass(
                &mut self,
                pass: Box<dyn RenderPass>,
        )
        {
                self.passes.push(pass);
        }

        pub fn execute(
                &mut self,
                view: &wgpu::TextureView,
                encoder: &mut wgpu::CommandEncoder,
        )
        {
                for pass in self.passes.iter_mut()
                {
                        if pass.enabled()
                        {
                                pass.record(&view, encoder);
                        }
                }
        }

        pub fn passes_mut(&mut self) -> &mut Vec<Box<dyn RenderPass>>
        {
                &mut self.passes
        }
}

pub trait RenderPass
{
        fn name(&self) -> &str;

        fn as_any(&self) -> &dyn Any;

        fn as_any_mut(&mut self) -> &mut dyn Any;

        fn ui(
                &mut self,
                ui: &mut egui::Ui,
        );

        fn enabled(&mut self) -> bool;

        fn set_enabled(
                &mut self,
                value: bool,
        );

        fn record(
                &mut self,
                view: &wgpu::TextureView,
                encoder: &mut wgpu::CommandEncoder,
        );
}

#[derive(Debug)]
pub struct BackgroundPass
{
        pub name: String,
        pub enabled: bool,
        pub clear_color: wgpu::Color,
        pub pipeline: wgpu::RenderPipeline,
}

impl RenderPass for BackgroundPass
{
        fn name(&self) -> &str
        {
                self.name.as_str()
        }

        fn as_any(&self) -> &dyn Any
        {
                self
        }

        fn as_any_mut(&mut self) -> &mut dyn Any
        {
                self
        }

        fn ui(
                &mut self,
                ui: &mut egui::Ui,
        )
        {
                egui::CollapsingHeader::new(&self.name)
                        .default_open(true)
                        .show(ui, |ui| {
                                // Enable/disable pass

                                // Interactive color picker
                                let mut color = [
                                        self.clear_color.r as f32,
                                        self.clear_color.g as f32,
                                        self.clear_color.b as f32,
                                        self.clear_color.a as f32,
                                ];

                                ui.horizontal(|ui| {
                                        ui.label("Color");
                                        if ui.color_edit_button_rgba_unmultiplied(&mut color)
                                                .changed()
                                        {
                                                self.clear_color = wgpu::Color {
                                                        r: color[0] as f64,
                                                        g: color[1] as f64,
                                                        b: color[2] as f64,
                                                        a: color[3] as f64,
                                                };
                                        }
                                });

                                // Info fields
                                ui.label("LoadOp: Clear");
                                ui.label("StoreOp: Store");
                                ui.label("Depth/stencil attachment: None");
                        });
        }

        fn enabled(&mut self) -> bool
        {
                self.enabled
        }

        fn set_enabled(
                &mut self,
                value: bool,
        )
        {
                self.enabled = value;
        }

        fn record(
                &mut self,
                view: &wgpu::TextureView,
                encoder: &mut wgpu::CommandEncoder,
        )
        {
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some(self.name()),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                view,
                                resolve_target: None,
                                ops: wgpu::Operations {
                                        load: wgpu::LoadOp::Clear(self.clear_color),
                                        store: wgpu::StoreOp::Store,
                                },
                        })],
                        depth_stencil_attachment: None,
                        occlusion_query_set: None,
                        timestamp_writes: None,
                });

                render_pass.set_pipeline(&self.pipeline);
        }
}

pub struct GeometryPass
{
        pub name: String,
        pub enabled: bool,
        pub pipeline: wgpu::RenderPipeline,
        pub resources: Arc<Mutex<Resources>>,
}

impl RenderPass for GeometryPass
{
        fn name(&self) -> &str
        {
                self.name.as_str()
        }

        fn as_any(&self) -> &dyn Any
        {
                self
        }

        fn as_any_mut(&mut self) -> &mut dyn Any
        {
                self
        }

        fn ui(
                &mut self,
                ui: &mut egui::Ui,
        )
        {
                egui::CollapsingHeader::new(&self.name)
                        .default_open(true)
                        .show(ui, |ui| {
                                // Info fields
                                ui.label("LoadOp: Clear");
                                ui.label("StoreOp: Store");
                                ui.label("Depth/stencil attachment: None");
                        });
        }

        fn enabled(&mut self) -> bool
        {
                self.enabled
        }

        fn set_enabled(
                &mut self,
                value: bool,
        )
        {
                self.enabled = value
        }

        fn record(
                &mut self,
                view: &wgpu::TextureView,
                encoder: &mut wgpu::CommandEncoder,
        )
        {
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some(&self.name),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                view,
                                resolve_target: None,
                                ops: wgpu::Operations {
                                        load: wgpu::LoadOp::Load,
                                        store: wgpu::StoreOp::Store,
                                },
                        })],
                        depth_stencil_attachment: None,
                        occlusion_query_set: None,
                        timestamp_writes: None,
                });

                render_pass.set_pipeline(&self.pipeline);

                let resources = self.resources.lock().unwrap();

                for mesh in resources.meshes.values()
                {
                        render_pass.set_vertex_buffer(0, mesh.vertex_buffer().unwrap().slice(..));

                        render_pass.set_index_buffer(
                                mesh.index_buffer().unwrap().slice(..),
                                mesh.index_format,
                        );

                        render_pass.draw_indexed(0..mesh.get_index_count(), 0, 0..1);
                }
        }
}

impl GeometryPass
{
        pub fn rebuild_pipeline(
                &mut self,
                device: &wgpu::Device,
                config: &wgpu::SurfaceConfiguration,
                fill_mode: FillMode,
                bind_groups: &[&wgpu::BindGroupLayout],
        )
        {
                // Decide the polygon mode based on FillMode
                let polygon_mode = match fill_mode
                {
                        FillMode::Fill => wgpu::PolygonMode::Fill,
                        FillMode::Wireframe => wgpu::PolygonMode::Line,
                        FillMode::Vertex => wgpu::PolygonMode::Point,
                };

                let shader = crate::renderer::pipeline::PipelineManager::load_shader_module(device);

                let render_pipeline_layout =
                        crate::renderer::pipeline::PipelineManager::get_render_pipeline_layout(
                                device,
                                bind_groups,
                        );

                let vertex_buffer = Vertex::get_desc();

                // Recreate the pipeline
                self.pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
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
                });
        }
}
