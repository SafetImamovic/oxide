pub enum Primitive
{
        Triangle,
        Square,
        Pentagon,
}

#[derive(Debug)]
pub struct Mesh
{
        pub name: String,
        pub vertex_buffer: wgpu::Buffer,
        pub index_buffer: wgpu::Buffer,
        pub num_elements: u32,
        pub material: usize,
}
