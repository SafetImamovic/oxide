use cgmath::*;
use std::f32::consts::FRAC_PI_2;
use std::time::Duration;
use wgpu::util::DeviceExt;
use winit::dpi::PhysicalPosition;
use winit::event::*;
use winit::keyboard::KeyCode;

#[derive(Debug)]
pub struct Camera
{
        pub projection: Projection,
        pub core: CameraCore,
        pub controller: CameraController,
        pub uniform: CameraUniform,
        pub config: CameraConfig,
        pub locked_in: bool,
        pub show_dpad: bool,
}

#[derive(Debug)]
pub struct CameraConfig
{
        pub sensitivity: f32,
        pub speed: f32,
        pub aspect_ratio_correction: bool,
        pub initial_aspect: Option<f32>,
        pub aspect: f32,
        pub fovy: Deg<f32>,
}

impl Default for CameraConfig
{
        fn default() -> Self
        {
                Self {
                        sensitivity: 1.0,
                        speed: 1.0,
                        aspect_ratio_correction: true,
                        initial_aspect: Some(1.0),
                        aspect: 1.0,
                        fovy: Deg(60.0),
                }
        }
}

impl Camera
{
        pub fn ui(
                &mut self,
                ui: &mut egui::Ui,
        )
        {
                let mut aspect = self.config.aspect_ratio_correction;

                egui::CollapsingHeader::new("Camera Settings")
                        .default_open(true)
                        .show(ui, |ui| {
                                ui.group(|ui| {
                                        egui::Grid::new("config_grid")
                                                .num_columns(2)
                                                .spacing([40.0, 8.0]) // horizontal and vertical spacing
                                                .show(ui, |ui| {
                                                        ui.label("Sensitivity");
                                                        ui.add(egui::Slider::new(
                                                                &mut self.config.sensitivity,
                                                                0.0..=10.0,
                                                        )
                                                        .step_by(0.1));
                                                        ui.end_row();

                                                        ui.label("Speed");
                                                        ui.add(egui::Slider::new(
                                                                &mut self.config.speed,
                                                                0.0..=10.0,
                                                        )
                                                        .step_by(0.1));
                                                        ui.end_row();

                                                        ui.label("FOV Y");
                                                        ui.add(egui::Slider::new(
                                                                &mut self.config.fovy.0,
                                                                1.0..=179.0,
                                                        ));
                                                        ui.end_row();

                                                        ui.label("D-Pad");
                                                        ui.checkbox(&mut self.show_dpad, "");
                                                        ui.end_row();

                                                        ui.label("Locked In");
                                                        ui.checkbox(&mut self.locked_in, "");
                                                        ui.end_row();

                                                        ui.label("Aspect Ratio Correction");
                                                        ui.checkbox(&mut aspect, "");
                                                        ui.end_row();
                                                });
                                });

                                ui.group(|ui| {
                                        egui::Grid::new("transform_grid")
                                                .num_columns(3) // Label | Reset/empty | Value
                                                .spacing([12.0, 4.0])
                                                .show(ui, |ui| {
                                                        // --- Position ---
                                                        ui.label("Position");
                                                        if ui.button("Reset").clicked()
                                                        {
                                                                self.core.position =
                                                                        Point3::new(0.0, 0.0, 0.0);
                                                        }
                                                        ui.end_row();

                                                        ui.label("  X");
                                                        ui.label(""); // keep grid alignment
                                                        ui.add(egui::DragValue::new(
                                                                &mut self.core.position.x,
                                                        )
                                                        .speed(0.1));
                                                        ui.end_row();

                                                        ui.label("  Y");
                                                        ui.label("");
                                                        ui.add(egui::DragValue::new(
                                                                &mut self.core.position.y,
                                                        )
                                                        .speed(0.1));
                                                        ui.end_row();

                                                        ui.label("  Z");
                                                        ui.label("");
                                                        ui.add(egui::DragValue::new(
                                                                &mut self.core.position.z,
                                                        )
                                                        .speed(0.1));
                                                        ui.end_row();

                                                        // --- Rotation ---
                                                        ui.label("Rotation");
                                                        if ui.button("Reset").clicked()
                                                        {
                                                                self.core.yaw = Rad(0.0);
                                                                self.core.pitch = Rad(0.0);
                                                        }
                                                        ui.end_row();

                                                        ui.label("  Yaw");
                                                        ui.label("");
                                                        let mut yaw_deg =
                                                                self.core.yaw.0.to_degrees();
                                                        if ui.add(egui::DragValue::new(
                                                                &mut yaw_deg,
                                                        )
                                                        .speed(1.0))
                                                                .changed()
                                                        {
                                                                self.core.yaw =
                                                                        Rad::from(Deg(yaw_deg));
                                                        }
                                                        ui.end_row();

                                                        ui.label("  Pitch");
                                                        ui.label("");
                                                        let mut pitch_deg =
                                                                self.core.pitch.0.to_degrees();
                                                        if ui.add(egui::DragValue::new(
                                                                &mut pitch_deg,
                                                        )
                                                        .clamp_existing_to_range(true)
                                                        .range(-89.0..=89.0)
                                                        .speed(1.0))
                                                                .changed()
                                                        {
                                                                self.core.pitch =
                                                                        Rad::from(Deg(pitch_deg));
                                                        }
                                                        ui.end_row();
                                                });
                                });
                        });

                if aspect != self.config.aspect_ratio_correction
                {
                        log::info!("Aspect ratio correction changed");

                        if !aspect
                        {
                                self.config.aspect = self.projection.aspect;
                                self.projection.aspect = self.config.initial_aspect.unwrap();
                        }
                        else
                        {
                                self.projection.aspect = self.config.aspect;
                        }

                        self.config.aspect_ratio_correction = aspect;
                }

                self.projection.fovy = Deg(self.config.fovy.0).into();
        }

        pub fn new() -> Self
        {
                let core = CameraCore::new((0.0, 5.0, 10.0), Deg(-90.0), Deg(-20.0));

                let projection = Projection::new(Deg(60.0), 0.1, 100.0);

                let config = CameraConfig::default();

                let controller = CameraController::new();

                let mut uniform = CameraUniform::new();

                uniform.update_view_proj(&core, &projection);

                Self {
                        projection,
                        core,
                        controller,
                        uniform,
                        config,
                        locked_in: true,
                        show_dpad: false,
                }
        }

        pub fn update(
                &mut self,
                dt: Duration,
        )
        {
                self.controller
                        .update_camera(&mut self.core, dt, &self.config);
                self.uniform.update_view_proj(&self.core, &self.projection);
        }

        pub fn get_buffer(
                &self,
                device: &wgpu::Device,
        ) -> wgpu::Buffer
        {
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Camera Buffer"),
                        contents: bytemuck::cast_slice(&[self.uniform]),
                        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                })
        }

        pub fn get_bind_group_layout(
                &self,
                device: &wgpu::Device,
        ) -> wgpu::BindGroupLayout
        {
                device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                        entries: &[wgpu::BindGroupLayoutEntry {
                                binding: 0,
                                visibility: wgpu::ShaderStages::VERTEX
                                        | wgpu::ShaderStages::FRAGMENT,
                                ty: wgpu::BindingType::Buffer {
                                        ty: wgpu::BufferBindingType::Uniform,
                                        has_dynamic_offset: false,
                                        min_binding_size: None,
                                },
                                count: None,
                        }],
                        label: Some("camera_bind_group_layout"),
                })
        }

        pub fn get_bind_group(
                &self,
                device: &wgpu::Device,
        ) -> wgpu::BindGroup
        {
                let camera_buffer = self.get_buffer(device);
                let camera_bind_group_layout = self.get_bind_group_layout(device);

                device.create_bind_group(&wgpu::BindGroupDescriptor {
                        layout: &camera_bind_group_layout,
                        entries: &[wgpu::BindGroupEntry {
                                binding: 0,
                                resource: camera_buffer.as_entire_binding(),
                        }],
                        label: Some("camera_bind_group"),
                })
        }
}

// We need this for Rust to store our data correctly for the shaders
#[repr(C)]
// This is so we can store this in a buffer
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform
{
        // We can't use cgmath with bytemuck directly, so we'll have
        // to convert the Matrix4 into a 4x4 f32 array
        pub view_position: [f32; 4],
        pub view_proj: [[f32; 4]; 4],
}

impl CameraUniform
{
        pub fn new() -> Self
        {
                use cgmath::SquareMatrix;
                Self {
                        view_proj: Matrix4::identity().into(),
                        view_position: [0.0; 4],
                }
        }

        pub fn update_view_proj(
                &mut self,
                camera: &CameraCore,
                projection: &Projection,
        )
        {
                self.view_position = camera.position.to_homogeneous().into();
                self.view_proj = (projection.calc_matrix() * camera.calc_matrix()).into();
        }
}

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: Matrix4<f32> = Matrix4::from_cols(
    Vector4::new(1.0, 0.0, 0.0, 0.0),
    Vector4::new(0.0, 1.0, 0.0, 0.0),
    Vector4::new(0.0, 0.0, 0.5, 0.0),
    Vector4::new(0.0, 0.0, 0.5, 1.0),
);

const SAFE_FRAC_PI_2: f32 = FRAC_PI_2 - 0.0001;

#[derive(Debug)]
pub struct CameraCore
{
        pub position: Point3<f32>,
        pub yaw: Rad<f32>,
        pub pitch: Rad<f32>,
}

impl CameraCore
{
        pub fn new<V: Into<Point3<f32>>, Y: Into<Rad<f32>>, P: Into<Rad<f32>>>(
                position: V,
                yaw: Y,
                pitch: P,
        ) -> Self
        {
                Self {
                        position: position.into(),
                        yaw: yaw.into(),
                        pitch: pitch.into(),
                }
        }

        pub fn calc_matrix(&self) -> Matrix4<f32>
        {
                let (sin_pitch, cos_pitch) = self.pitch.0.sin_cos();
                let (sin_yaw, cos_yaw) = self.yaw.0.sin_cos();

                Matrix4::look_to_rh(
                        self.position,
                        Vector3::new(cos_pitch * cos_yaw, sin_pitch, cos_pitch * sin_yaw)
                                .normalize(),
                        Vector3::unit_y(),
                )
        }
}

#[derive(Debug)]
pub struct Projection
{
        pub aspect: f32,
        pub fovy: Rad<f32>,
        pub znear: f32,
        pub zfar: f32,
}

impl Projection
{
        pub fn new<F: Into<Rad<f32>>>(
                fovy: F,
                znear: f32,
                zfar: f32,
        ) -> Self
        {
                Self {
                        aspect: 1.0,
                        fovy: fovy.into(),
                        znear,
                        zfar,
                }
        }

        pub fn resize(
                &mut self,
                width: u32,
                height: u32,
        )
        {
                self.aspect = width as f32 / height as f32;
        }

        pub fn calc_matrix(&self) -> Matrix4<f32>
        {
                OPENGL_TO_WGPU_MATRIX * perspective(self.fovy, self.aspect, self.znear, self.zfar)
        }
}

#[derive(Debug)]
pub struct CameraController
{
        pub amount_left: f32,
        pub amount_right: f32,
        pub amount_forward: f32,
        pub amount_backward: f32,
        pub amount_up: f32,
        pub amount_down: f32,
        pub rotate_horizontal: f32,
        pub rotate_vertical: f32,
        pub scroll: f32,
}

impl CameraController
{
        pub fn new() -> Self
        {
                Self {
                        amount_left: 0.0,
                        amount_right: 0.0,
                        amount_forward: 0.0,
                        amount_backward: 0.0,
                        amount_up: 0.0,
                        amount_down: 0.0,
                        rotate_horizontal: 0.0,
                        rotate_vertical: 0.0,
                        scroll: 0.0,
                }
        }

        pub fn handle_key(
                &mut self,
                key: KeyCode,
                pressed: bool,
        ) -> bool
        {
                let amount = if pressed { 1.0 } else { 0.0 };
                match key
                {
                        KeyCode::KeyW | KeyCode::ArrowUp =>
                        {
                                self.amount_forward = amount;
                                true
                        }
                        KeyCode::KeyS | KeyCode::ArrowDown =>
                        {
                                self.amount_backward = amount;
                                true
                        }
                        KeyCode::KeyA | KeyCode::ArrowLeft =>
                        {
                                self.amount_left = amount;
                                true
                        }
                        KeyCode::KeyD | KeyCode::ArrowRight =>
                        {
                                self.amount_right = amount;
                                true
                        }
                        KeyCode::Space =>
                        {
                                self.amount_up = amount;
                                true
                        }
                        KeyCode::ShiftLeft =>
                        {
                                self.amount_down = amount;
                                true
                        }
                        _ => false,
                }
        }

        pub fn handle_mouse(
                &mut self,
                mouse_dx: f64,
                mouse_dy: f64,
        )
        {
                self.rotate_horizontal = mouse_dx as f32;
                self.rotate_vertical = mouse_dy as f32;
        }

        pub fn handle_scroll(
                &mut self,
                delta: &MouseScrollDelta,
        )
        {
                self.scroll = match delta
                {
                        // I'm assuming a line is about 100 pixels
                        MouseScrollDelta::LineDelta(_, scroll) => -scroll * 0.5,
                        MouseScrollDelta::PixelDelta(PhysicalPosition {
                                y: scroll, ..
                        }) => -*scroll as f32,
                };
        }

        pub fn update_camera(
                &mut self,
                camera: &mut CameraCore,
                dt: Duration,
                config: &CameraConfig,
        )
        {
                let dt = dt.as_secs_f32();

                // Move forward/backward and left/right
                let (yaw_sin, yaw_cos) = camera.yaw.0.sin_cos();
                let forward = Vector3::new(yaw_cos, 0.0, yaw_sin).normalize();
                let right = Vector3::new(-yaw_sin, 0.0, yaw_cos).normalize();
                camera.position +=
                        forward * (self.amount_forward - self.amount_backward) * config.speed * dt;
                camera.position +=
                        right * (self.amount_right - self.amount_left) * config.speed * dt;

                // Move in/out (aka. "zoom")
                // Note: this isn't an actual zoom. The camera's position
                // changes when zooming. I've added this to make it easier
                // to get closer to an object you want to focus on.
                let (pitch_sin, pitch_cos) = camera.pitch.0.sin_cos();
                let scrollward = Vector3::new(pitch_cos * yaw_cos, pitch_sin, pitch_cos * yaw_sin)
                        .normalize();
                camera.position +=
                        scrollward * self.scroll * config.speed * config.sensitivity * dt;
                self.scroll = 0.0;

                // Move up/down. Since we don't use roll, we can just
                // modify the y coordinate directly.
                camera.position.y += (self.amount_up - self.amount_down) * config.speed * dt;

                // Rotate
                camera.yaw += Rad(self.rotate_horizontal) * config.sensitivity * dt;
                camera.pitch += Rad(-self.rotate_vertical) * config.sensitivity * dt;

                // If process_mouse isn't called every frame, these values
                // will not get set to zero, and the camera will rotate
                // when moving in a non cardinal direction.
                self.rotate_horizontal = 0.0;
                self.rotate_vertical = 0.0;

                // Keep the camera's angle from going too high/low.
                if camera.pitch < -Rad(SAFE_FRAC_PI_2)
                {
                        camera.pitch = -Rad(SAFE_FRAC_PI_2);
                }
                else if camera.pitch > Rad(SAFE_FRAC_PI_2)
                {
                        camera.pitch = Rad(SAFE_FRAC_PI_2);
                }
        }
}
