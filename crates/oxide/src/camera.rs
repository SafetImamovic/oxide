use wgpu::util::DeviceExt;
use wgpu::Device;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::keyboard::{KeyCode, PhysicalKey};

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::from_cols(
    cgmath::Vector4::new(1.0, 0.0, 0.0, 0.0),
    cgmath::Vector4::new(0.0, 1.0, 0.0, 0.0),
    cgmath::Vector4::new(0.0, 0.0, 0.5, 0.0),
    cgmath::Vector4::new(0.0, 0.0, 0.5, 1.0),
);

#[derive(Debug)]
pub struct Camera
{
        pub uniform: CameraUniform,
        pub core: CameraCore,
        pub controller: CameraController,
}

impl Camera
{
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

        pub fn update_camera(&mut self)
        {
                use cgmath::InnerSpace;
                let forward = &self.core.target - &self.core.eye;
                let forward_norm = forward.normalize();
                let forward_mag = forward.magnitude();

                // Prevents glitching when the camera gets too close to the
                // center of the scene.
                if self.controller.is_forward_pressed && forward_mag > self.controller.speed
                {
                        self.core.eye += forward_norm * self.controller.speed;
                }
                if self.controller.is_backward_pressed
                {
                        self.core.eye -= forward_norm * self.controller.speed;
                }

                let right = forward_norm.cross(self.core.up);

                // Redo radius calc in case the forward/backward is pressed.
                let forward = self.core.target - self.core.eye;
                let forward_mag = forward.magnitude();

                if self.controller.is_right_pressed
                {
                        // Rescale the distance between the target and the eye so
                        // that it doesn't change. The eye, therefore, still
                        // lies on the circle made by the target and eye.
                        self.core.eye = self.core.target
                                - (forward + right * self.controller.speed).normalize()
                                        * forward_mag;
                }
                if self.controller.is_left_pressed
                {
                        self.core.eye = self.core.target
                                - (forward - right * self.controller.speed).normalize()
                                        * forward_mag;
                }
        }

        pub fn update(
                &mut self,
                device: &Device,
                queue: &wgpu::Queue,
        )
        {
                self.update_camera();

                self.uniform.update_view_proj(&self.core);

                queue.write_buffer(
                        &self.get_buffer(device),
                        0,
                        bytemuck::cast_slice(&[self.uniform]),
                );
        }

        pub fn get_bind_group_layout(
                &self,
                device: &wgpu::Device,
        ) -> wgpu::BindGroupLayout
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

        pub fn get_bind_group(
                &self,
                device: &Device,
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
        pub view_proj: [[f32; 4]; 4],
}

impl CameraUniform
{
        pub fn new() -> Self
        {
                use cgmath::SquareMatrix;
                Self {
                        view_proj: cgmath::Matrix4::identity().into(),
                }
        }

        pub fn update_view_proj(
                &mut self,
                camera: &CameraCore,
        )
        {
                self.view_proj = camera.build_view_projection_matrix().into();
        }
}

#[derive(Debug)]
pub struct CameraCore
{
        pub eye: cgmath::Point3<f32>,
        pub target: cgmath::Point3<f32>,
        pub up: cgmath::Vector3<f32>,
        pub aspect: f32,
        pub fovy: f32,
        pub znear: f32,
        pub zfar: f32,
}

impl CameraCore
{
        fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32>
        {
                // 1.
                let view = cgmath::Matrix4::look_at_rh(self.eye, self.target, self.up);
                // 2.
                let proj = cgmath::perspective(
                        cgmath::Deg(self.fovy),
                        self.aspect,
                        self.znear,
                        self.zfar,
                );

                OPENGL_TO_WGPU_MATRIX * proj * view
        }
}

#[derive(Debug)]
pub struct CameraController
{
        pub speed: f32,
        pub is_forward_pressed: bool,
        pub is_backward_pressed: bool,
        pub is_left_pressed: bool,
        pub is_right_pressed: bool,
}

impl CameraController
{
        pub fn new(speed: f32) -> Self
        {
                Self {
                        speed,
                        is_forward_pressed: false,
                        is_backward_pressed: false,
                        is_left_pressed: false,
                        is_right_pressed: false,
                }
        }

        pub fn process_events(
                &mut self,
                event: &WindowEvent,
        ) -> bool
        {
                log::info!("CALLED !!!! {:?}", event);
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
                                        _ => false,
                                }
                        }
                        _ => false,
                }
        }
}
