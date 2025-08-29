use wgpu::util::{BufferInitDescriptor, DeviceExt};

use crate::geometry::vertex::Vertex;

#[derive(Debug)]
pub struct Mesh
{
        name: String,
        // CPU-Side data
        vertices: Vec<Vertex>,
        indices: Vec<u16>,

        // GPU-Side data
        vertex_buffer: Option<wgpu::Buffer>,
        index_buffer: Option<wgpu::Buffer>,
        index_count: u32,
        vertex_count: u32,
        pub index_format: wgpu::IndexFormat,

        needs_upload: bool,
}

impl Mesh
{
        //---------------------- Public ------------------------

        pub fn new(
                name: impl Into<String>,
                vertices: Vec<Vertex>,
                indices: Vec<u16>,
        ) -> Self
        {
                let vertex_count = vertices.len() as u32;
                let index_count = indices.len() as u32;

                Self {
                        name: name.into(),
                        vertices,
                        indices,
                        vertex_buffer: None,
                        index_buffer: None,
                        vertex_count,
                        index_count,
                        index_format: wgpu::IndexFormat::Uint16,
                        needs_upload: true,
                }
        }

        pub fn needs_upload(&self) -> bool
        {
                self.needs_upload
        }

        pub fn get_index_count(&self) -> u32
        {
                self.index_count
        }

        pub fn get_vertex_count(&self) -> u32
        {
                self.vertex_count
        }

        pub fn vertex_buffer(&self) -> anyhow::Result<&wgpu::Buffer>
        {
                self.vertex_buffer
                        .as_ref()
                        .ok_or_else(|| anyhow::anyhow!("Vertex buffer not initialized"))
        }

        pub fn index_buffer(&self) -> anyhow::Result<&wgpu::Buffer>
        {
                self.index_buffer
                        .as_ref()
                        .ok_or_else(|| anyhow::anyhow!("Index buffer not initialized"))
        }

        /// Uploads CPU data to GPU buffers. Safe to call multiple times.
        pub fn upload(
                &mut self,
                device: &wgpu::Device,
                usage: wgpu::BufferUsages,
        )
        {
                if !self.needs_upload
                {
                        return;
                }

                log::info!("Vertex and Index buffers created for mesh::{}", self.name);

                self.vertex_buffer = Some(device.create_buffer_init(&BufferInitDescriptor {
                        label: Some(&format!("mesh::{}::vertex_buffer", self.name)),
                        contents: bytemuck::cast_slice(&self.vertices),
                        usage: wgpu::BufferUsages::VERTEX | usage,
                }));

                self.index_buffer = Some(device.create_buffer_init(&BufferInitDescriptor {
                        label: Some(&format!("mesh::{}::index_buffer", self.name)),
                        contents: bytemuck::cast_slice(&self.indices),
                        usage: wgpu::BufferUsages::INDEX | usage,
                }));

                self.needs_upload = false;
        }
}
