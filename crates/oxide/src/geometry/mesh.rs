use wgpu::util::{BufferInitDescriptor, DeviceExt};

use crate::geometry::{primitives::*, vertex::Vertex};

pub enum Primitive
{
        Triangle,
        Square,
        Pentagon,
}

#[derive(Debug)]
pub struct Mesh
{
        pub position: cgmath::Point3<f32>,
        pub direction: cgmath::Vector3<f32>,

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

        pub needs_upload: bool,
}

impl Mesh
{
        pub fn new(
                name: impl Into<String>,
                vertices: Vec<Vertex>,
                indices: Vec<u16>,
        ) -> Self
        {
                let vertex_count = vertices.len() as u32;
                let index_count = indices.len() as u32;

                Self {
                        position: cgmath::Point3::new(0.0, 0.0, 0.0),
                        direction: cgmath::Vector3::new(0.0, 0.0, 0.0),
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

        pub fn basic(
                name: impl Into<String>,
                primitive: Primitive,
        ) -> Self
        {
                match primitive
                {
                        Primitive::Triangle => Self::new(name, TRI_V.to_vec(), TRI_I.to_vec()),
                        Primitive::Square => Self::new(name, SQ_V.to_vec(), SQ_I.to_vec()),
                        Primitive::Pentagon => Self::new(name, PENT_V.to_vec(), PENT_I.to_vec()),
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

                for i in self.vertices.iter_mut()
                {
                        i.position[0] += self.direction.x;
                        i.position[1] += self.direction.y;
                        i.position[2] += self.direction.z;
                }

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

        /// Convenience: builds a regular n-gon at the origin in the XY plane
        /// with the given radius. This version constructs `Vertex`
        /// values in a minimal, generic way. Adjust if your
        /// `Vertex` has a different layout.
        pub fn generate_n_gon(
                sides: u32,
                radius: f32,
        ) -> Self
        {
                // Adjust this closure to match your Vertex layout if needed.
                Self::generate_n_gon_with_radius(sides, radius, |pos, _i| Vertex {
                        // If your Vertex doesn't have these fields or Default, adapt accordingly.
                        position: pos,
                        ..Default::default()
                })
        }

        /// Back-compat wrapper: same as before, assumes radius = 1.0.
        /// Use `generate_n_gon_with_radius` if you need a custom radius.
        pub fn generate_n_gon_with<F>(
                sides: u32,
                mut make_vertex: F,
        ) -> Self
        where
                F: FnMut([f32; 3], usize) -> Vertex,
        {
                Self::generate_n_gon_with_radius(sides, 1.0, move |pos, i| make_vertex(pos, i))
        }

        /// Flexible generator that lets you decide how to build each `Vertex`,
        /// with a custom radius.
        /// - sides >= 3
        /// - Center at origin, XY plane, Z = 0
        /// - CCW winding (positive radius). Negative radius flips winding.
        pub fn generate_n_gon_with_radius<F>(
                sides: u32,
                radius: f32,
                mut make_vertex: F,
        ) -> Self
        where
                F: FnMut([f32; 3], usize) -> Vertex,
        {
                assert!(sides >= 3, "n-gon must have at least 3 sides");
                assert!(
                        (sides as usize) + 1 <= (u16::MAX as usize),
                        "too many sides for u16 index buffer"
                );
                assert!(radius.is_finite(), "radius must be a finite number");

                let n = sides as usize;

                // Center + ring
                let mut vertices = Vec::with_capacity(n + 1);
                vertices.push(make_vertex([0.0, 0.0, 0.0], 0));

                let angle_step = 2.0_f32 * std::f32::consts::PI / (sides as f32);
                for i in 0..n
                {
                        let angle = (i as f32) * angle_step;
                        let x = radius * angle.cos();
                        let y = radius * angle.sin();
                        vertices.push(make_vertex([x, y, 0.0], i + 1));
                }

                // Triangle fan from center (index 0)
                let mut indices = Vec::<u16>::with_capacity(n * 3);
                for i in 0..n
                {
                        let b = (i + 1) as u16;
                        let c = ((i + 1) % n + 1) as u16;
                        indices.extend_from_slice(&[0, b, c]);
                }

                Mesh {
                        position: cgmath::Point3::new(0.0, 0.0, 0.0),
                        name: format!("n-gon-{}-r{}", sides, radius),
                        direction: cgmath::Vector3::new(0.0, 0.0, 0.0),
                        vertices,
                        indices,
                        vertex_buffer: None,
                        index_buffer: None,
                        index_count: (n as u32) * 3,
                        vertex_count: (n as u32) + 1,
                        index_format: wgpu::IndexFormat::Uint16,
                        needs_upload: true,
                }
        }
}
