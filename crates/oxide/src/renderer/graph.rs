use std::{
        any::Any,
        sync::{Arc, Mutex},
};

use derivative::Derivative;

use crate::renderer::pipeline::{PipelineKind, PipelineManager};
use crate::resource::Resources;

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
                pipeline_manager: &PipelineManager,
        )
        {
                for pass in self.passes.iter_mut()
                {
                        if pass.enabled()
                        {
                                pass.record(&view, encoder, &pipeline_manager);
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
                pipeline_manager: &PipelineManager,
        );
}

#[derive(Debug)]
pub struct BackgroundPass
{
        pub name: String,
        pub enabled: bool,
        pub clear_color: wgpu::Color,
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
                pipeline_manager: &PipelineManager,
        )
        {
                // For a background pass, we typically don't need depth testing
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
                        depth_stencil_attachment: None, // Background doesn't need depth
                        occlusion_query_set: None,
                        timestamp_writes: None,
                });

                render_pass.set_pipeline(pipeline_manager.get(PipelineKind::Geometry));
        }
}

pub struct GeometryPass
{
        pub name: String,
        pub enabled: bool,
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
                                ui.label("LoadOp: Load");
                                ui.label("StoreOp: Store");
                                ui.label("Depth/stencil attachment: None");

                                if ui.button("Refresh Geometry").clicked()
                                {
                                        // This could trigger a refresh of
                                        // geometry data
                                }
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
                pipeline_manager: &PipelineManager,
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

                render_pass.set_pipeline(pipeline_manager.get(PipelineKind::Geometry));

                let resources = self.resources.lock().unwrap();

                for mesh in resources.meshes.iter()
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
