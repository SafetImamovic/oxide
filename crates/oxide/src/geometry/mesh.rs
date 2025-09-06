use crate::model::ModelVertex;
use cgmath::Matrix4;

pub enum Primitive
{
        Triangle,
        Square,
        Pentagon,
}

#[derive(Debug)]
pub struct MeshData
{
        pub name: String,
        pub vertices: Vec<ModelVertex>,
        pub indices: Vec<u32>,
        pub material_id: Option<usize>,
        pub transform: Matrix4<f32>,
}

#[derive(Debug)]
pub struct Mesh
{
        pub name: String,
        pub vertex_buffer: wgpu::Buffer,
        pub index_buffer: wgpu::Buffer,
        pub num_elements: u32,
        pub material: usize,
        pub transform_buffer: wgpu::Buffer,
        pub transform_bind_group: wgpu::BindGroup,
}
