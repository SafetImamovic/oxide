use egui::Context;
use egui_wgpu::Renderer;
use egui_wgpu::ScreenDescriptor;
use egui_winit::State;
use wgpu::CommandEncoder;
use wgpu::Device;
use wgpu::Queue;
use wgpu::StoreOp;
use wgpu::TextureFormat;
use wgpu::TextureView;
use winit::event::WindowEvent;
use winit::window::Window;

pub struct GuiRenderer
{
        state: State,
        renderer: Renderer,
        pub show_right_panel: bool,
        frame_started: bool,
}

impl GuiRenderer
{
        pub fn context(&self) -> &Context
        {
                self.state.egui_ctx()
        }

        pub fn new(
                device: &Device,
                output_color_format: TextureFormat,
                output_depth_format: Option<TextureFormat>,
                scale_factor: f32,
                msaa_samples: u32,
                window: &Window,
        ) -> GuiRenderer
        {
                let egui_context = Context::default();
                let egui_state = egui_winit::State::new(
                        egui_context,
                        egui::viewport::ViewportId::ROOT,
                        &window,
                        Some(window.scale_factor() as f32),
                        None,
                        Some(2 * 1024), // default dimension is 2048
                );
                let egui_renderer = Renderer::new(
                        device,
                        output_color_format,
                        output_depth_format,
                        msaa_samples,
                        true,
                );

                GuiRenderer {
                        show_right_panel: true,
                        state: egui_state,
                        renderer: egui_renderer,
                        frame_started: false,
                }
        }

        pub fn handle_input(
                &mut self,
                window: &Window,
                event: &WindowEvent,
        )
        {
                let _ = self.state.on_window_event(window, event);
        }

        pub fn ppp(
                &mut self,
                v: f32,
        )
        {
                self.context().set_pixels_per_point(1.0);
        }

        pub fn begin_frame(
                &mut self,
                window: &Window,
                config: &crate::config::Config,
        )
        {
                self.ppp(Self::current_pixels_per_point(window, config));

                let raw_input = self.state.take_egui_input(window);

                self.state.egui_ctx().begin_pass(raw_input);

                self.frame_started = true;
        }

        pub fn end_frame_and_draw(
                &mut self,
                device: &Device,
                queue: &Queue,
                encoder: &mut CommandEncoder,
                window: &Window,
                window_surface_view: &TextureView,
                screen_descriptor: ScreenDescriptor,
        )
        {
                if !self.frame_started
                {
                        panic!(
                                "begin_frame must be called before end_frame_and_draw can be called!"
                        );
                }

                let full_output = self.state.egui_ctx().end_pass();

                self.state
                        .handle_platform_output(window, full_output.platform_output);

                let tris = self.state.egui_ctx().tessellate(full_output.shapes, 1.0);
                for (id, image_delta) in &full_output.textures_delta.set
                {
                        self.renderer
                                .update_texture(device, queue, *id, image_delta);
                }
                self.renderer
                        .update_buffers(device, queue, encoder, &tris, &screen_descriptor);
                let rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                view: window_surface_view,
                                resolve_target: None,
                                ops: egui_wgpu::wgpu::Operations {
                                        load: egui_wgpu::wgpu::LoadOp::Load,
                                        store: StoreOp::Store,
                                },
                        })],
                        depth_stencil_attachment: None,

                        timestamp_writes: None,
                        label: Some("egui main render pass"),
                        occlusion_query_set: None,
                });

                self.renderer
                        .render(&mut rpass.forget_lifetime(), &tris, &screen_descriptor);
                for x in &full_output.textures_delta.free
                {
                        self.renderer.free_texture(x)
                }

                self.frame_started = false;
        }

        pub fn render(
                &mut self,
                config: &mut crate::Config,
        )
        {
                self.debug_window(config);
        }

        pub fn debug_window(
                &mut self,
                config: &mut crate::Config,
        )
        {
                let mut scale: f32 = config.gui_scale;

                let ctx = self.context().clone();

                egui::Area::new("nice".into())
                        .fixed_pos(egui::pos2(10.0, 10.0))
                        .show(&ctx, |ui| {
                                ui.label("Press [Tab] to toggle right menu");
                        });

                if self.show_right_panel
                {
                        egui::Window::new("Right Panel")
                                .anchor(egui::Align2::RIGHT_TOP, [0.0, 0.0])
                                .show(self.context(), |ui| {
                                        ui.label("Docked content");
                                });
                }

                /*
                egui::Window::new("Oxide Debug Window")
                        .resizable(true)
                        .vscroll(true)
                        .default_open(false)
                        .show(self.context(), |ui| {
                                ui.horizontal(ui_def);
                        });
                */

                config.gui_scale = scale;
        }

        #[cfg(target_arch = "wasm32")]
        pub fn current_pixels_per_point(
                window: &winit::window::Window,
                config: &crate::config::Config,
        ) -> f32
        {
                let ppp = web_sys::window().unwrap().device_pixel_ratio() as f32;

                log::info!("PPP: {ppp}");

                ppp
        }

        #[cfg(not(target_arch = "wasm32"))]
        pub fn current_pixels_per_point(
                window: &winit::window::Window,
                config: &crate::config::Config,
        ) -> f32
        {
                window.scale_factor() as f32 * config.gui_scale
        }
}
