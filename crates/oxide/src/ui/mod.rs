use crate::ui::renderer::GuiRenderer;
use wgpu::{Device, TextureFormat};
use winit::window::Window;

pub mod renderer;

#[derive(Debug)]
pub struct UiSystem
{
        pub ui_scale: f32,
        pub renderer: GuiRenderer,
}

impl UiSystem
{
        pub fn new(
                device: &Device,
                output_color_format: &TextureFormat,
                output_depth_format: Option<TextureFormat>,
                msaa_samples: u32,
                window: &Window,
        ) -> Self
        {
                let renderer = GuiRenderer::new(
                        device,
                        output_color_format,
                        output_depth_format,
                        msaa_samples,
                        window,
                );

                Self {
                        ui_scale: 1.5,
                        renderer,
                }
        }
}
