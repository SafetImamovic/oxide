use wgpu::util::{BufferInitDescriptor, DeviceExt};

use crate::geometry::vertex::Vertex;

#[derive(Debug)]
pub struct Mesh
{
        pub vertices: &'static [Vertex],
        pub indices: &'static [u16],
}

impl Mesh
{
        pub fn get_num_indices(&self) -> u32
        {
                self.indices.len() as u32
        }

        pub fn new_vertex_buffer(
                &self,
                device: &wgpu::Device,
        ) -> wgpu::Buffer
        {
                device.create_buffer_init(&BufferInitDescriptor {
                        label: Some("Vertex Buffer"),
                        contents: bytemuck::cast_slice(self.vertices),
                        usage: wgpu::BufferUsages::VERTEX,
                })
        }

        pub fn new_index_buffer(
                &self,
                device: &wgpu::Device,
        ) -> wgpu::Buffer
        {
                device.create_buffer_init(&BufferInitDescriptor {
                        label: Some("Index Buffer"),
                        contents: bytemuck::cast_slice(self.indices),
                        usage: wgpu::BufferUsages::INDEX,
                })
        }
}
