/// Vertex struct.
///
/// Uses C-compatible memory layout (`#[repr(C)]`)
/// so it can be safely shared with GPU graphics APIs.
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex
{
        pub position: [f32; 3],
        pub tex_coords: [f32; 2],
}

impl Vertex
{
        pub fn get_desc() -> wgpu::VertexBufferLayout<'static>
        {
                wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &[
                                wgpu::VertexAttribute {
                                        offset: 0,
                                        shader_location: 0,
                                        format: wgpu::VertexFormat::Float32x3,
                                },
                                wgpu::VertexAttribute {
                                        offset: std::mem::size_of::<[f32; 3]>()
                                                as wgpu::BufferAddress,
                                        shader_location: 1,
                                        format: wgpu::VertexFormat::Float32x2,
                                },
                        ],
                }
        }
}

pub const TRIANGLE: &[Vertex] = &[
        Vertex {
                position: [-0.0868241, 0.49240386, 0.0],
                tex_coords: [0.4131759, 0.00759614],
        }, // A
        Vertex {
                position: [-0.49513406, 0.06958647, 0.0],
                tex_coords: [0.0048659444, 0.43041354],
        }, // B
        Vertex {
                position: [-0.21918549, -0.44939706, 0.0],
                tex_coords: [0.28081453, 0.949397],
        }, // C
        Vertex {
                position: [0.35966998, -0.3473291, 0.0],
                tex_coords: [0.85967, 0.84732914],
        }, // D
        Vertex {
                position: [0.44147372, 0.2347359, 0.0],
                tex_coords: [0.9414737, 0.2652641],
        }, // E
];

pub const INDICES: &[u16] = &[0, 1, 4, 1, 2, 4, 2, 3, 4];

pub const SQUARE: &[Vertex] = &[
        Vertex {
                position: [-0.5, 0.5, 0.0],
                tex_coords: [0.0, 0.0],
        }, // A
        Vertex {
                position: [-0.5, -0.5, 0.0],
                tex_coords: [0.0, 1.0],
        }, // B
        Vertex {
                position: [0.5, -0.5, 0.0],
                tex_coords: [1.0, 1.0],
        }, // C
        Vertex {
                position: [0.5, 0.5, 0.0],
                tex_coords: [1.0, 0.0],
        }, // D
];

pub const SQ_INDICES: &[u16] = &[0, 1, 2, 0, 2, 3];

pub const SQUARE_2: &[Vertex] = &[
        Vertex {
                position: [-0.5, 0.5, 0.0],
                tex_coords: [0.0, 0.0],
        }, // A
        Vertex {
                position: [-0.5, -0.5, 0.0],
                tex_coords: [1.0, 0.0],
        }, // B
        Vertex {
                position: [0.5, -0.5, 0.0],
                tex_coords: [1.0, 1.0],
        }, // C
        Vertex {
                position: [0.5, 0.5, 0.0],
                tex_coords: [0.0, 1.0],
        }, // D
];

pub const SQ_INDICES_2: &[u16] = &[0, 1, 2, 0, 2, 3];
