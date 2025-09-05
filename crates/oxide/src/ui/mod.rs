use crate::camera::CameraController;
use crate::ui::renderer::GuiRenderer;
use egui::{Align2, Area, Button, Frame, Id, RichText, Rounding, Sense, Vec2};
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

#[derive(Debug, Clone)]
pub struct MovementUiConfig
{
        /// Size of each arrow button in logical points.
        pub button_size: f32,
        /// Spacing between buttons.
        pub spacing: f32,
        /// Opacity of the control background (0..=1).
        pub bg_opacity: f32,
}

impl Default for MovementUiConfig
{
        fn default() -> Self
        {
                Self {
                        button_size: 40.0,
                        spacing: 6.0,
                        bg_opacity: 0.85,
                }
        }
}

pub fn show_movement_controls(
        ctx: &egui::Context,
        id: Id,
        cfg: &MovementUiConfig,
        controller: &mut CameraController,
)
{
        Area::new(id)
                .order(egui::Order::Foreground)
                .anchor(Align2::LEFT_BOTTOM, [16.0, -16.0])
                .interactable(true)
                .show(ctx, |ui| {
                        let frame = Frame::canvas(ui.style())
                                .rounding(Rounding::same(16))
                                .fill(ui.visuals().panel_fill.linear_multiply(cfg.bg_opacity))
                                .inner_margin(egui::Margin::same(10))
                                .stroke(ui.visuals().widgets.active.bg_stroke);

                        frame.show(ui, |ui| {
                                ui.spacing_mut().item_spacing = Vec2::splat(cfg.spacing);

                                let bsize = Vec2::splat(cfg.button_size);
                                let arrow = |s: &str| {
                                        RichText::new(s).strong();
                                };

                                // Up (forward)
                                ui.horizontal_centered(|ui| {
                                        let up = ui.add_sized(
                                                bsize,
                                                Button::new(arrow("↑")).sense(Sense::click()),
                                        );
                                        controller.amount_forward = if up
                                                .is_pointer_button_down_on()
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
                                                Button::new(arrow("←")).sense(Sense::click()),
                                        );
                                        controller.amount_left = if left.is_pointer_button_down_on()
                                        {
                                                1.0
                                        }
                                        else
                                        {
                                                0.0
                                        };

                                        ui.add_space(cfg.spacing);

                                        let right = ui.add_sized(
                                                bsize,
                                                Button::new(arrow("→")).sense(Sense::click()),
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
                                });

                                // Down (backward)
                                ui.horizontal_centered(|ui| {
                                        let down = ui.add_sized(
                                                bsize,
                                                Button::new(arrow("↓")).sense(Sense::click()),
                                        );
                                        controller.amount_backward = if down
                                                .is_pointer_button_down_on()
                                        {
                                                1.0
                                        }
                                        else
                                        {
                                                0.0
                                        };
                                });
                        });
                });
}
