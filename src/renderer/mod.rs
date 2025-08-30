use wgpu::util::DeviceExt;

pub mod resource;
pub mod shader;
pub mod graph;
pub mod pipeline;
pub mod renderer;
pub mod camera;

/// The RenderContext manages all rendering operations and resources.
#[derive(Debug)]
pub struct RenderContext {
    pub camera: Option<camera::Camera>,
    pub camera_uniform: camera::CameraUniform,
    pub camera_buffer: Option<wgpu::Buffer>,
    pub camera_bind_group: Option<wgpu::BindGroup>,
}

impl RenderContext {
    pub fn new() -> Self {
        Self {
            camera: None,
            camera_uniform: camera::CameraUniform::new(),
            camera_buffer: None,
            camera_bind_group: None,
        }
    }

    pub fn initialize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        // Create camera
        let mut camera = camera::Camera::new(width, height);

        // Create camera uniform
        let mut camera_uniform = camera::CameraUniform::new();
        camera_uniform.update_view_proj(&camera);

        // Create camera buffer
        let camera_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: bytemuck::cast_slice(&[camera_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );


        // Create camera bind group layout
        let camera_bind_group_layout = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }
                ],
                label: Some("camera_bind_group_layout"),
            }
        );

        // Create camera bind group
        let camera_bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                layout: &camera_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: camera_buffer.as_entire_binding(),
                    }
                ],
                label: Some("camera_bind_group"),
            }
        );

        self.camera = Some(camera);
        self.camera_uniform = camera_uniform;
        self.camera_buffer = Some(camera_buffer);
        self.camera_bind_group = Some(camera_bind_group);
    }

    pub fn update_camera(&mut self, queue: &wgpu::Queue) {
        if let Some(camera) = &self.camera {
            self.camera_uniform.update_view_proj(camera);
            queue.write_buffer(
                self.camera_buffer.as_ref().unwrap(),
                0,
                bytemuck::cast_slice(&[self.camera_uniform]),
            );
        }
    }
}
