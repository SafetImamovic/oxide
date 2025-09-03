use std::sync::Arc;

#[derive(Debug)]
pub struct SurfaceManager
{
        pub surface: wgpu::Surface<'static>,
        pub configuration: wgpu::SurfaceConfiguration,
        pub depth: wgpu::TextureView,
        pub capabilities: wgpu::SurfaceCapabilities,
        pub is_surface_configured: bool,
}

impl SurfaceManager
{
        pub fn new(
                instance: &wgpu::Instance,
                window: Arc<winit::window::Window>,
                adapter: &wgpu::Adapter,
                device: &wgpu::Device,
        ) -> anyhow::Result<Self>
        {
                let surface = instance.create_surface(window.clone())?;

                let size = window.clone().inner_size();

                let capabilities = surface.get_capabilities(adapter);

                let format = capabilities.formats[0]; // usually SRGB

                let configuration = wgpu::SurfaceConfiguration {
                        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                        format,
                        width: size.width,
                        height: size.height,
                        present_mode: wgpu::PresentMode::Fifo, // vsync
                        desired_maximum_frame_latency: 2,
                        alpha_mode: capabilities.alpha_modes[0],
                        view_formats: vec![],
                };

                surface.configure(device, &configuration);

                let depth = Self::create_depth_texture(device, &configuration);

                Ok(Self {
                        surface,
                        configuration,
                        depth,
                        capabilities,
                        is_surface_configured: false,
                })
        }

        pub fn resize(
                &mut self,
                device: &wgpu::Device,
                new_size: winit::dpi::PhysicalSize<u32>,
        )
        {
                if new_size.width > 0 && new_size.height > 0
                {
                        self.configuration.width = new_size.width;
                        self.configuration.height = new_size.height;
                        self.surface.configure(device, &self.configuration);
                        self.depth = Self::create_depth_texture(device, &self.configuration);
                }
        }

        pub fn acquire_frame(
                &self,
                device: &wgpu::Device,
        ) -> anyhow::Result<(wgpu::SurfaceTexture, wgpu::TextureView, wgpu::CommandEncoder)>
        {
                let output = self.surface.get_current_texture().map_err(|e| match e
                {
                        wgpu::SurfaceError::Outdated => anyhow::anyhow!("Surface outdated"),
                        e => anyhow::anyhow!(e),
                })?;

                let view = output
                        .texture
                        .create_view(&wgpu::TextureViewDescriptor::default());

                let encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some("Main Render Encoder"),
                });

                Ok((output, view, encoder))
        }

        fn create_depth_texture(
                device: &wgpu::Device,
                config: &wgpu::SurfaceConfiguration,
        ) -> wgpu::TextureView
        {
                let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
                        label: Some("Depth Texture"),
                        size: wgpu::Extent3d {
                                width: config.width,
                                height: config.height,
                                depth_or_array_layers: 1,
                        },
                        mip_level_count: 1,
                        sample_count: 1,
                        dimension: wgpu::TextureDimension::D2,
                        format: wgpu::TextureFormat::Depth24PlusStencil8,
                        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                        view_formats: &[],
                });
                depth_texture.create_view(&wgpu::TextureViewDescriptor::default())
        }
}
