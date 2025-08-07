use egui::{Context, ViewportId};
use egui_wgpu::Renderer;
use egui_winit::State;
use wgpu::Device;
use wgpu::TextureFormat;
use winit::event::WindowEvent;
use winit::window::Theme;
use winit::window::Window;

pub struct GuiRenderer
{
        pub context: Context,
        state: State,
        renderer: Renderer,
}

impl GuiRenderer
{
        pub fn new(device: &Device,
                   output_color_format: &TextureFormat,
                   msaa_samples: u32,
                   window: &Window)
                   -> anyhow::Result<GuiRenderer>
        {
                let egui_context = Context::default();

                let egui_state = egui_winit::State::new(egui_context.clone(),
                                                        ViewportId::from_hash_of(window.id()),
                                                        &window,
                                                        Some(window.scale_factor() as f32),
                                                        Some(Theme::Dark),
                                                        None);

                let egui_renderer = egui_wgpu::Renderer::new(device,
                                                             *output_color_format,
                                                             None,
                                                             msaa_samples,
                                                             false);

                Ok(GuiRenderer { context: egui_context,
                                 state: egui_state,
                                 renderer: egui_renderer })
        }

        pub fn handle_input(&mut self, window: &Window, event: &WindowEvent)
        {
                let _ = self.state.on_window_event(&window, &event);
        }

        pub fn draw(&mut self,
                    device: &wgpu::Device,
                    queue: &wgpu::Queue,
                    encoder: &mut wgpu::CommandEncoder,
                    window: &Window,
                    window_surface_view: &wgpu::TextureView,
                    screen_descriptor: egui_wgpu::ScreenDescriptor,
                    run_ui: &mut impl FnMut(&egui::Context))
        {
                // Handle input and run UI
                let raw_input = self.state.take_egui_input(window);
                let full_output = self.context.run(raw_input, |_ui| {
                                                      run_ui(&self.context);
                                              });
                self.state
                    .handle_platform_output(window, full_output.platform_output);

                // Process paint jobs and textures
                let tris = self.context
                               .tessellate(full_output.shapes, full_output.pixels_per_point);
                for (id, image_delta) in &full_output.textures_delta.set
                {
                        self.renderer
                            .update_texture(device, queue, *id, image_delta);
                }
                self.renderer
                    .update_buffers(device, queue, encoder, &tris, &screen_descriptor);

                // Create render pass with forget_lifetime()
                let  rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("egui main render pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: window_surface_view,
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

                // SAFETY: We ensure all resources live long enough
                let mut rpass_static = unsafe { rpass.forget_lifetime() };

                self.renderer
                    .render(&mut rpass_static, &tris, &screen_descriptor);

                // Free textures
                for x in &full_output.textures_delta.free
                {
                        self.renderer.free_texture(x);
                }
        }
}
