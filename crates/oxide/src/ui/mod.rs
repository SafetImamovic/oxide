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
        let bsize = Vec2::new(96.0, 96.0);

        egui::Area::new(egui::Id::from("dpad"))
                .anchor(Align2::LEFT_BOTTOM, egui::vec2(10.0, -10.0)) // bottom-left corner
                .show(ctx, |ui| {
                        egui::Grid::new("dpad_grid")
                                .spacing(Vec2::new(0.0, 0.0))
                                .num_columns(3)
                                .show(ui, |ui| {
                                        let up = ui.add_sized(
                                                bsize,
                                                Button::new("UP").sense(egui::Sense::drag()),
                                        );
                                        controller.amount_up = if up.is_pointer_button_down_on()
                                        {
                                                1.0
                                        }
                                        else
                                        {
                                                0.0
                                        };

                                        let forward = ui.add_sized(
                                                bsize,
                                                Button::new("FORWARD").sense(egui::Sense::drag()),
                                        );
                                        controller.amount_forward = if forward
                                                .is_pointer_button_down_on()
                                        {
                                                1.0
                                        }
                                        else
                                        {
                                                0.0
                                        };

                                        let down = ui.add_sized(
                                                bsize,
                                                Button::new("DOWN").sense(egui::Sense::drag()),
                                        );
                                        controller.amount_down = if down.is_pointer_button_down_on()
                                        {
                                                1.0
                                        }
                                        else
                                        {
                                                0.0
                                        };

                                        ui.end_row();

                                        let left = ui.add_sized(
                                                bsize,
                                                Button::new("LEFT").sense(egui::Sense::drag()),
                                        );
                                        controller.amount_left = if left.is_pointer_button_down_on()
                                        {
                                                1.0
                                        }
                                        else
                                        {
                                                0.0
                                        };

                                        ui.label("");

                                        let right = ui.add_sized(
                                                bsize,
                                                Button::new("RIGHT").sense(egui::Sense::drag()),
                                        );
                                        controller.amount_right = if right
                                                .is_pointer_button_down_on()
                                        {
                                                1.0
                                        }
                                        else
                                        {
                                                0.0
                                        };

                                        ui.end_row();

                                        ui.label("");

                                        let backward = ui.add_sized(
                                                bsize,
                                                Button::new("BACKWARD").sense(egui::Sense::drag()),
                                        );
                                        controller.amount_backward = if backward
                                                .is_pointer_button_down_on()
                                        {
                                                1.0
                                        }
                                        else
                                        {
                                                0.0
                                        };

                                        ui.label("");
                                        ui.end_row();
                                });
                });
}
