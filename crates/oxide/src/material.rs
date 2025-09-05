pub struct Material
{
        pub name: String,
        pub diffuse_texture: crate::texture::Texture,
        pub bind_group: wgpu::BindGroup,
}
