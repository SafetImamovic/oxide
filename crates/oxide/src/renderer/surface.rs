use std::sync::Arc;
use winit::dpi::PhysicalSize;

#[derive(Debug)]
pub struct SurfaceManager
{
        pub surface: wgpu::Surface<'static>,
        pub configuration: wgpu::SurfaceConfiguration,
        pub capabilities: wgpu::SurfaceCapabilities,
        pub is_surface_configured: bool,
}

impl SurfaceManager
{
        pub fn new(
                instance: &wgpu::Instance,
                window: Arc<winit::window::Window>,
                size: &PhysicalSize<u32>,
                adapter: &wgpu::Adapter,
        ) -> anyhow::Result<Self>
        {
                let surface = instance.create_surface(window.clone())?;

                let capabilities = surface.get_capabilities(adapter);

                let format = capabilities.formats[0];

                let configuration = Self::get_config(&size, &capabilities, format);

                //let depth = Self::create_depth_texture(device, &configuration);

                Ok(Self {
                        surface,
                        configuration,
                        capabilities,
                        is_surface_configured: false,
                })
        }

        pub fn build_configuration(
                &mut self,
                size: &PhysicalSize<u32>,
        )
        {
                self.configuration =
                        Self::get_config(size, &self.capabilities, self.configuration.format);
        }

        pub fn get_config(
                size: &PhysicalSize<u32>,
                capabilities: &wgpu::SurfaceCapabilities,
                format: wgpu::TextureFormat,
        ) -> wgpu::SurfaceConfiguration
        {
                wgpu::SurfaceConfiguration {
                        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                        format,
                        width: size.width,
                        height: size.height,
                        present_mode: wgpu::PresentMode::Fifo, // vsync
                        desired_maximum_frame_latency: 2,
                        alpha_mode: capabilities.alpha_modes[0],
                        view_formats: vec![],
                }
        }

        /// Get the surface texture ONCE per frame
        ///
        /// Returns the next texture to be presented by the swapchain for
        /// drawing.
        ///
        /// In order to present the SurfaceTexture returned by this method,
        /// first a Queue::submit needs to be done with some work rendering to
        /// this texture. Then SurfaceTexture::present needs to be
        /// called.
        ///
        /// If a SurfaceTexture referencing this surface is alive when the
        /// swapchain is recreated, recreating the swapchain will panic
        pub fn acquire_frame(
                &self,
                device: &wgpu::Device,
        ) -> anyhow::Result<Option<(wgpu::SurfaceTexture, wgpu::TextureView, wgpu::CommandEncoder)>>
        {
                if !self.is_surface_configured
                {
                        return Ok(None);
                }

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

                Ok(Some((output, view, encoder)))
        }
}
