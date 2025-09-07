#[derive(Debug)]
pub struct MaterialData
{
        pub name: String,
        pub diffuse_texture: Option<String>,
        pub base_color_texture: Option<String>,
        pub normal_texture: Option<String>,
        pub metallic_roughness_texture: Option<String>,
        pub base_color_factor: [f32; 4],
        pub metallic_factor: f32,
        pub roughness_factor: f32,
        pub base_color_texture_index: Option<usize>,
        pub normal_texture_index: Option<usize>,
        pub metallic_roughness_texture_index: Option<usize>,
}

impl Default for MaterialData
{
        fn default() -> Self
        {
                Self {
                        name: "default".to_string(),
                        diffuse_texture: None,
                        base_color_texture: None,
                        normal_texture: None,
                        metallic_roughness_texture: None,
                        base_color_factor: [1.0, 1.0, 1.0, 1.0],
                        metallic_factor: 1.0,
                        roughness_factor: 1.0,
                        base_color_texture_index: None,
                        normal_texture_index: None,
                        metallic_roughness_texture_index: None,
                }
        }
}

#[derive(Debug)]
pub struct Material
{
        pub name: String,
        pub base_color_texture: crate::texture::Texture,
        pub normal_texture: Option<crate::texture::Texture>,
        pub metallic_roughness_texture: Option<crate::texture::Texture>,
        pub base_color_factor: [f32; 4],
        pub metallic_factor: f32,
        pub roughness_factor: f32,
        pub material_bind_group: wgpu::BindGroup,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MaterialProperties
{
        pub base_color_factor: [f32; 4],
        pub metallic_factor: f32,
        pub roughness_factor: f32,
        // Padding to meet WGSL alignment requirements (16 bytes)
        pub _padding: [f32; 2],
}

pub fn create_material_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout
{
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                        // Base color texture
                        wgpu::BindGroupLayoutEntry {
                                binding: 0,
                                visibility: wgpu::ShaderStages::FRAGMENT,
                                ty: wgpu::BindingType::Texture {
                                        sample_type: wgpu::TextureSampleType::Float {
                                                filterable: true,
                                        },
                                        view_dimension: wgpu::TextureViewDimension::D2,
                                        multisampled: false,
                                },
                                count: None,
                        },
                        // Base color sampler
                        wgpu::BindGroupLayoutEntry {
                                binding: 1,
                                visibility: wgpu::ShaderStages::FRAGMENT,
                                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                                count: None,
                        },
                        // Material properties uniform
                        wgpu::BindGroupLayoutEntry {
                                binding: 2,
                                visibility: wgpu::ShaderStages::FRAGMENT,
                                ty: wgpu::BindingType::Buffer {
                                        ty: wgpu::BufferBindingType::Uniform,
                                        has_dynamic_offset: false,
                                        min_binding_size: None,
                                },
                                count: None,
                        },
                ],
                label: Some("material_bind_group_layout"),
        })
}
