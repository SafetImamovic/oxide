use crate::camera::CameraController;
use crate::ui::renderer::GuiRenderer;
use egui::{Align2, Button, Vec2};
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
                        ui_scale: 1.2,
                        renderer,
                }
        }
}

pub fn draw_dpad(
        ctx: &egui::Context,
        controller: &mut CameraController,
)
{
        let pointer_down = ctx.input(|i| i.pointer.primary_down());
        let bsize = Vec2::new(32.0, 32.0);

        egui::Area::new(egui::Id::from("dpad"))
                .anchor(Align2::LEFT_BOTTOM, egui::vec2(10.0, -10.0)) // bottom-left corner
                .show(ctx, |ui| {
                        // Forward
                        ui.horizontal_centered(|ui| {
                                let up = ui.add_sized(
                                        bsize,
                                        Button::new("↑").sense(egui::Sense::drag()),
                                );
                                controller.amount_forward = if up.is_pointer_button_down_on()
                                        || (up.hovered() && pointer_down)
                                {
                                        1.0
                                }
                                else
                                {
                                        0.0
                                };
                        });

                        // Left / Right
                        ui.horizontal(|ui| {
                                let left = ui.add_sized(
                                        bsize,
                                        Button::new("←").sense(egui::Sense::drag()),
                                );
                                controller.amount_left = if left.is_pointer_button_down_on()
                                        || (left.hovered() && pointer_down)
                                {
                                        1.0
                                }
                                else
                                {
                                        0.0
                                };

                                ui.add_space(8.0);

                                let right = ui.add_sized(
                                        bsize,
                                        Button::new("→").sense(egui::Sense::drag()),
                                );
                                controller.amount_right = if right.is_pointer_button_down_on()
                                        || (right.hovered() && pointer_down)
                                {
                                        1.0
                                }
                                else
                                {
                                        0.0
                                };
                        });

                        // Backward
                        ui.horizontal_centered(|ui| {
                                let down = ui.add_sized(
                                        bsize,
                                        Button::new("↓").sense(egui::Sense::drag()),
                                );
                                controller.amount_backward = if down.is_pointer_button_down_on()
                                        || (down.hovered() && pointer_down)
                                {
                                        1.0
                                }
                                else
                                {
                                        0.0
                                };
                        });
                });
}
