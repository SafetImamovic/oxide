use crate::camera::Camera;
use crate::engine::FillMode;
use crate::renderer::graph::RenderGraph;
use derivative::Derivative;
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

#[derive(Derivative)]
#[derivative(Debug)]
pub struct GuiRenderer
{
        #[derivative(Debug = "ignore")]
        state: State,

        #[derivative(Debug = "ignore")]
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
                output_color_format: &TextureFormat,
                output_depth_format: Option<TextureFormat>,
                msaa_samples: u32,
                window: &Window,
        ) -> GuiRenderer
        {
                let egui_context = Context::default();

                let egui_state = State::new(
                        egui_context,
                        egui::viewport::ViewportId::ROOT,
                        &window,
                        Some(window.scale_factor() as f32),
                        None,
                        Some(2 * 1024), // default dimension is 2048
                );

                let egui_renderer = Renderer::new(
                        device,
                        *output_color_format,
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
                self.context().set_pixels_per_point(v);
        }

        pub fn begin_frame(
                &mut self,
                window: &Window,
                ui_scale: &mut f32,
        )
        {
                self.ppp(self.current_pixels_per_point(window, ui_scale));

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
                        .handle_platform_output(&window, full_output.platform_output);

                let tris = self
                        .state
                        .egui_ctx()
                        .tessellate(full_output.shapes, self.context().pixels_per_point());

                //log::info!("Triangles Pre: {}", tris.len());
                //log::info!("Textures alive Pre: {}", full_output.textures_delta.set.len());

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
                                ops: wgpu::Operations {
                                        load: wgpu::LoadOp::Load,
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

                //log::info!("Triangles Post: {}", tris.len());
                //log::info!("Textures alive Pre: {}",
                // full_output.textures_delta.set.len());
        }

        pub fn render(
                &mut self,
                graph: &mut RenderGraph,
                ui_scale: &mut f32,
                fill_mode: &mut FillMode,
                features: wgpu::Features,
                camera: &mut Camera,
        )
        {
                self.debug_window(graph, ui_scale, fill_mode, features, camera);
        }

        pub fn debug_window(
                &mut self,
                graph: &mut RenderGraph,
                ui_scale: &mut f32,
                fill_mode: &mut FillMode,
                features: wgpu::Features,
                camera: &mut Camera,
        )
        {
                let mut temp_fill_mode = *fill_mode;
                let mut scale: f32 = *ui_scale;

                egui::Area::new("nice".into())
                        .fixed_pos(egui::pos2(10.0, 10.0))
                        .show(self.context(), |ui| {
                                ui.label("Press [Tab] to toggle right menu");
                        });

                if self.show_right_panel
                {
                        egui::SidePanel::right("Right Panel").resizable(true).show(self.context(), |ui| {
                                egui::ScrollArea::new(true).show(ui, |ui| {
                                        // UI scale controls
                                        ui.horizontal(|ui| {
                                                if ui.button(egui::RichText::new("[   -   ]").strong().text_style(egui::TextStyle::Monospace))
                                                    .clicked()
                                                {
                                                        scale = (scale - 0.1).max(0.5);
                                                }
                                                if ui.button(egui::RichText::new("[   +   ]").strong().text_style(egui::TextStyle::Monospace))
                                                    .clicked()
                                                {
                                                        scale = (scale + 0.1).min(3.0);
                                                }
                                                ui.label(format!("UI Scale: {:.1}", scale));
                                        });

                                        // Fill mode
                                        egui::ComboBox::from_label("Fill Mode")
                                            .selected_text(format!("{:?}", temp_fill_mode))
                                            .show_ui(ui, |ui| {
                                                    ui.selectable_value(
                                                            &mut temp_fill_mode,
                                                            FillMode::Fill,
                                                            "Fill",
                                                    );
                                                    if features
                                                        .contains(wgpu::Features::POLYGON_MODE_LINE)
                                                    {
                                                            ui.selectable_value(
                                                                    &mut temp_fill_mode,
                                                                    FillMode::Wireframe,
                                                                    "Wireframe",
                                                            );
                                                    }
                                                    if features.contains(
                                                            wgpu::Features::POLYGON_MODE_POINT,
                                                    )
                                                    {
                                                            ui.selectable_value(
                                                                    &mut temp_fill_mode,
                                                                    FillMode::Vertex,
                                                                    "Vertex",
                                                            );
                                                    }
                                            });

                                        camera.ui(ui);

                                        // Collapsible section for passes
                                        egui::CollapsingHeader::new("Render Pass Graph")
                                            .default_open(false)
                                            .show(ui, |ui| {
                                                    let len = graph.passes.len();
                                                    let mut move_req: Option<(usize, isize)> = None;

                                                    for (i, pass) in graph.passes.iter_mut().enumerate()
                                                    {
                                                            let mut enabled = pass.enabled();

                                                            ui.horizontal(|ui| {
                                                                    pass.ui(ui);

                                                                    ui.with_layout(
                                                                            egui::Layout::right_to_left(egui::Align::Center),
                                                                            |ui| {
                                                                                    ui.checkbox(&mut enabled, "Enabled");

                                                                                    if ui.button(egui::RichText::new("[ v ]").strong().text_style(egui::TextStyle::Monospace)).clicked() && i + 1 < len {
                                                                                            move_req = Some((i, 1));
                                                                                    }
                                                                                    if ui.button(egui::RichText::new("[ ^ ]").strong().text_style(egui::TextStyle::Monospace)).clicked() && i > 0 {
                                                                                            move_req = Some((i, -1));
                                                                                    }
                                                                            },
                                                                    );
                                                            });

                                                            ui.separator();
                                                            pass.set_enabled(enabled);
                                                    }

                                                    if let Some((i, d)) = move_req
                                                    {
                                                            let j = (i as isize + d) as usize;
                                                            graph.passes.swap(i, j);
                                                    }
                                            });
                                });

                        });
                }

                *ui_scale = scale;
                if *fill_mode != temp_fill_mode
                {
                        *fill_mode = temp_fill_mode;
                }
        }

        #[cfg(target_arch = "wasm32")]
        pub fn current_pixels_per_point(
                &self,
                _window: &winit::window::Window,
                ui_scale: &mut f32,
        ) -> f32
        {
                web_sys::window().unwrap().device_pixel_ratio() as f32 * *ui_scale
        }

        #[cfg(not(target_arch = "wasm32"))]
        pub fn current_pixels_per_point(
                &self,
                window: &Window,
                ui_scale: &mut f32,
        ) -> f32
        {
                window.scale_factor() as f32 * ui_scale.clone()
        }
}
