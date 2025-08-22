use cgmath::SquareMatrix;
use wgpu::util::DeviceExt;
use winit::{
        event::{ElementState, KeyEvent, WindowEvent},
        keyboard::{KeyCode, PhysicalKey},
};

pub struct Camera
{
        pub eye: cgmath::Point3<f32>,
        pub target: cgmath::Point3<f32>,
        pub up: cgmath::Vector3<f32>,
        pub aspect: f32,
        pub fovy: f32,
        pub znear: f32,
        pub zfar: f32,
}

impl Camera
{
        pub fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32>
        {
                let view = cgmath::Matrix4::look_at_rh(self.eye, self.target, self.up);

                let proj = cgmath::perspective(
                        cgmath::Deg(self.fovy),
                        self.aspect,
                        self.znear,
                        self.zfar,
                );

                OPENGL_TO_WGPU_MATRIX * proj * view
        }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform
{
        view_proj: [[f32; 4]; 4],
}

#[allow(clippy::new_without_default)]
impl CameraUniform
{
        pub fn new() -> Self
        {
                Self {
                        view_proj: cgmath::Matrix4::identity().into(),
                }
        }

        pub fn update_view_proj(
                &mut self,
                camera: &Camera,
        )
        {
                self.view_proj = camera.build_view_projection_matrix().into();
        }

        pub fn new_buffer(
                &self,
                device: &wgpu::Device,
        ) -> wgpu::Buffer
        {
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Camera Buffer"),
                        contents: bytemuck::cast_slice(&[*self]),
                        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                })
        }

        pub fn new_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout
        {
                device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                        entries: &[wgpu::BindGroupLayoutEntry {
                                binding: 0,
                                visibility: wgpu::ShaderStages::VERTEX,
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

        pub fn new_bind_group(
                device: &wgpu::Device,
                camera_bind_group_layout: &wgpu::BindGroupLayout,
                camera_buffer: &wgpu::Buffer,
        ) -> wgpu::BindGroup
        {
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

pub struct Controller
{
        pub speed: f32,
        pub is_forward_pressed: bool,
        pub is_backward_pressed: bool,
        pub is_left_pressed: bool,
        pub is_right_pressed: bool,
        pub is_top_pressed: bool,
        pub is_down_pressed: bool,
}

impl Controller
{
        pub fn new(speed: f32) -> Self
        {
                Self {
                        speed,
                        is_forward_pressed: false,
                        is_backward_pressed: false,
                        is_left_pressed: false,
                        is_right_pressed: false,
                        is_top_pressed: false,
                        is_down_pressed: false,
                }
        }

        pub fn process_events(
                &mut self,
                event: &WindowEvent,
        ) -> bool
        {
                match event
                {
                        WindowEvent::KeyboardInput {
                                event:
                                        KeyEvent {
                                                state,
                                                physical_key: PhysicalKey::Code(keycode),
                                                ..
                                        },
                                ..
                        } =>
                        {
                                let is_pressed = *state == ElementState::Pressed;
                                match keycode
                                {
                                        KeyCode::KeyW | KeyCode::ArrowUp =>
                                        {
                                                self.is_forward_pressed = is_pressed;
                                                true
                                        }
                                        KeyCode::KeyA | KeyCode::ArrowLeft =>
                                        {
                                                self.is_left_pressed = is_pressed;
                                                true
                                        }
                                        KeyCode::KeyS | KeyCode::ArrowDown =>
                                        {
                                                self.is_backward_pressed = is_pressed;
                                                true
                                        }
                                        KeyCode::KeyD | KeyCode::ArrowRight =>
                                        {
                                                self.is_right_pressed = is_pressed;
                                                true
                                        }
                                        KeyCode::KeyQ =>
                                        {
                                                self.is_down_pressed = is_pressed;
                                                true
                                        }
                                        KeyCode::KeyE =>
                                        {
                                                self.is_top_pressed = is_pressed;
                                                true
                                        }
                                        _ => false,
                                }
                        }
                        _ => false,
                }
        }

        pub fn update_camera(
                &self,
                camera: &mut Camera,
        )
        {
                use cgmath::{InnerSpace, Vector3};

                let forward = (camera.target - camera.eye).normalize();

                let right = forward.cross(camera.up).normalize();

                let up = right.cross(forward).normalize();

                let mut delta = Vector3::new(0.0, 0.0, 0.0);

                if self.is_forward_pressed
                {
                        delta += forward * self.speed;
                }
                if self.is_backward_pressed
                {
                        delta -= forward * self.speed;
                }
                if self.is_right_pressed
                {
                        delta += right * self.speed;
                }
                if self.is_left_pressed
                {
                        delta -= right * self.speed;
                }
                if self.is_top_pressed
                {
                        delta += up * self.speed;
                }
                if self.is_down_pressed
                {
                        delta -= up * self.speed;
                }

                // Move both eye and target so direction stays constant
                camera.eye += delta;
                camera.target += delta;
        }
}

pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::from_cols(
        cgmath::Vector4::new(1.0, 0.0, 0.0, 0.0),
        cgmath::Vector4::new(0.0, 1.0, 0.0, 0.0),
        cgmath::Vector4::new(0.0, 0.0, 0.5, 0.0),
        cgmath::Vector4::new(0.0, 0.0, 0.5, 1.0),
);
