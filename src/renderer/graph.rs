use derivative::Derivative;
use egui_wgpu::Renderer;

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

        fn ui(&self);

        fn enabled(&self) -> bool;

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

        fn ui(&self)
        {
                log::info!("UI for BackgroundPass.");
        }

        fn enabled(&self) -> bool
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

