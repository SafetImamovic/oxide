use crate::geometry::mesh::{Mesh, MeshData};
use crate::material::{MaterialData, MaterialProperties};
use std::ops::Range;
use wgpu::util::DeviceExt;

pub trait Vertex
{
        fn desc() -> wgpu::VertexBufferLayout<'static>;
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ModelVertex
{
        pub position: [f32; 3],
        pub tex_coords: [f32; 2],
        pub normal: [f32; 3],
}

impl Vertex for ModelVertex
{
        fn desc() -> wgpu::VertexBufferLayout<'static>
        {
                wgpu::VertexBufferLayout {
                        array_stride: size_of::<ModelVertex>() as wgpu::BufferAddress,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &[
                                wgpu::VertexAttribute {
                                        offset: 0,
                                        shader_location: 0,
                                        format: wgpu::VertexFormat::Float32x3,
                                },
                                wgpu::VertexAttribute {
                                        offset: size_of::<[f32; 3]>() as wgpu::BufferAddress,
                                        shader_location: 1,
                                        format: wgpu::VertexFormat::Float32x2,
                                },
                                wgpu::VertexAttribute {
                                        offset: size_of::<[f32; 5]>() as wgpu::BufferAddress,
                                        shader_location: 2,
                                        format: wgpu::VertexFormat::Float32x3,
                                },
                        ],
                }
        }
}

#[derive(Debug)]
pub struct Model
{
        pub meshes: Vec<Mesh>,
        pub materials: Vec<crate::material::Material>,
}

impl Model
{
        pub fn from_data(
                meshes: Vec<MeshData>,
                materials: Vec<MaterialData>,
                images: Vec<gltf::image::Data>,
                device: &wgpu::Device,
                queue: &wgpu::Queue,
                material_bind_group_layout: &wgpu::BindGroupLayout,
                transform_bind_group_layout: &wgpu::BindGroupLayout,
        ) -> Self
        {
                // Convert GLB images to GPU textures
                let gpu_textures: Vec<crate::texture::Texture> = images
                .iter()
                .enumerate()
                .map(|(index, image)| {
                    log::info!("IMAGE {} INFO: {:?} ({}x{})", index, image.format, image.width, image.height);

                    let size = wgpu::Extent3d {
                        width: image.width,
                        height: image.height,
                        depth_or_array_layers: 1,
                    };

                    // Determine bytes per pixel and convert if necessary
                    let (final_pixels, bytes_per_pixel, target_format) = match image.format {
                        gltf::image::Format::R8G8B8A8 => {
                            // Already RGBA, use as-is
                            (image.pixels.clone(), 4, wgpu::TextureFormat::Rgba8UnormSrgb)
                        }
                        gltf::image::Format::R8G8B8 => {
                            // Convert RGB to RGBA
                            let mut rgba_data = Vec::with_capacity(image.pixels.len() * 4 / 3);
                            for chunk in image.pixels.chunks_exact(3) {
                                rgba_data.extend_from_slice(chunk);
                                rgba_data.push(255); // Add full alpha
                            }
                            (rgba_data, 4, wgpu::TextureFormat::Rgba8UnormSrgb)
                        }
                        gltf::image::Format::R8G8 => {
                            // R8G8 format (2 bytes per pixel) - use appropriate texture format
                            // Convert to RGBA if needed, or use a two-channel format
                            let mut rgba_data = Vec::with_capacity(image.pixels.len() * 2);
                            for chunk in image.pixels.chunks_exact(2) {
                                rgba_data.extend_from_slice(chunk);
                                rgba_data.push(0); // Add blue channel
                                rgba_data.push(255); // Add alpha channel
                            }
                            (rgba_data, 4, wgpu::TextureFormat::Rgba8UnormSrgb)
                        }
                        _ => {
                            log::warn!("Unknown image format {:?}, defaulting to RGBA", image.format);
                            (image.pixels.clone(), 4, wgpu::TextureFormat::Rgba8UnormSrgb)
                        }
                    };

                    let texture = device.create_texture(&wgpu::TextureDescriptor {
                        label: Some(&format!("GLB Texture {}", index)),
                        size,
                        mip_level_count: 1,
                        sample_count: 1,
                        dimension: wgpu::TextureDimension::D2,
                        format: target_format,
                        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                        view_formats: &[],
                    });

                    // Calculate bytes per row with proper alignment
                    let unpadded_bytes_per_row: usize = bytes_per_pixel as usize * image.width as usize;
                    let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as usize;
                    let padded_bytes_per_row = ((unpadded_bytes_per_row + align - 1) / align) * align;

                    log::debug!("Texture {}: {}x{}, bpp: {}, unpadded: {}, padded: {}",
            index, image.width, image.height, bytes_per_pixel,
            unpadded_bytes_per_row, padded_bytes_per_row);

                    // Verify the final data size matches expectations
                    let expected_size = unpadded_bytes_per_row * image.height as usize;
                    assert_eq!(
                        final_pixels.len(),
                        expected_size,
                        "Image {}: Expected {} bytes, got {} bytes",
                        index, expected_size, final_pixels.len()
                    );

                    // If padding is needed, create padded data
                    let upload_data = if padded_bytes_per_row > unpadded_bytes_per_row {
                        let mut padded_data = Vec::with_capacity(padded_bytes_per_row * image.height as usize);

                        for y in 0..image.height as usize {
                            let row_start = y * unpadded_bytes_per_row;
                            let row_end = row_start + unpadded_bytes_per_row;

                            // Add the actual row data
                            padded_data.extend_from_slice(&final_pixels[row_start..row_end]);

                            // Add padding zeros
                            padded_data.resize(padded_data.len() + (padded_bytes_per_row - unpadded_bytes_per_row), 0);
                        }
                        padded_data
                    } else {
                        final_pixels
                    };

                    queue.write_texture(
                        wgpu::TexelCopyTextureInfo {
                            texture: &texture,
                            mip_level: 0,
                            origin: wgpu::Origin3d::ZERO,
                            aspect: wgpu::TextureAspect::All,
                        },
                        &upload_data,
                        wgpu::TexelCopyBufferLayout {
                            offset: 0,
                            bytes_per_row: Some(padded_bytes_per_row as u32),
                            rows_per_image: Some(image.height),
                        },
                        size,
                    );

                    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
                    let sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());

                    crate::texture::Texture {
                        texture,
                        view,
                        sampler,
                    }
                })
                .collect();

                // Upload materials
                let gpu_materials = materials
        .into_iter()
        .map(|mat| {
            // Choose base color texture from GLB images
            let base_color_texture = mat
                .base_color_texture_index
                .and_then(|idx| gpu_textures.get(idx).cloned())
                .unwrap_or_else(|| crate::texture::Texture::create_dummy(device, queue));

            let normal_texture = mat
                .normal_texture_index
                .and_then(|idx| gpu_textures.get(idx).cloned());

            let metallic_roughness_texture = mat
                .metallic_roughness_texture_index
                .and_then(|idx| gpu_textures.get(idx).cloned());

            // Material uniform
            let material_properties = MaterialProperties {
                base_color_factor: mat.base_color_factor,
                metallic_factor: mat.metallic_factor,
                roughness_factor: mat.roughness_factor,
                _padding: [0.0; 2],
            };

            let material_properties_buffer = device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("Material Properties Buffer"),
                    contents: bytemuck::cast_slice(&[material_properties]),
                    usage: wgpu::BufferUsages::UNIFORM,
                },
            );

            let material_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: material_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&base_color_texture.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&base_color_texture.sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: material_properties_buffer.as_entire_binding(),
                    },
                ],
                label: Some(&format!("{} Material Bind Group", mat.name)),
            });

            crate::material::Material {
                name: mat.name,
                base_color_texture,
                normal_texture,
                metallic_roughness_texture,
                base_color_factor: mat.base_color_factor,
                metallic_factor: mat.metallic_factor,
                roughness_factor: mat.roughness_factor,
                material_bind_group,
            }
        })
        .collect::<Vec<_>>();

                // Mesh upload stays the same
                let gpu_meshes = meshes
                        .into_iter()
                        .map(|m| {
                                let vertex_buffer = device.create_buffer_init(
                                        &wgpu::util::BufferInitDescriptor {
                                                label: Some(&format!("{} Vertex Buffer", m.name)),
                                                contents: bytemuck::cast_slice(&m.vertices),
                                                usage: wgpu::BufferUsages::VERTEX,
                                        },
                                );

                                let index_buffer = device.create_buffer_init(
                                        &wgpu::util::BufferInitDescriptor {
                                                label: Some(&format!("{} Index Buffer", m.name)),
                                                contents: bytemuck::cast_slice(&m.indices),
                                                usage: wgpu::BufferUsages::INDEX,
                                        },
                                );

                                let transform_data: [[f32; 4]; 4] = m.transform.into();

                                let transform_buffer = device.create_buffer_init(
                                        &wgpu::util::BufferInitDescriptor {
                                                label: Some(&format!(
                                                        "{} Transform Buffer",
                                                        m.name
                                                )),
                                                contents: bytemuck::cast_slice(&transform_data),
                                                usage: wgpu::BufferUsages::UNIFORM
                                                        | wgpu::BufferUsages::COPY_DST,
                                        },
                                );

                                let transform_bind_group =
                                        device.create_bind_group(&wgpu::BindGroupDescriptor {
                                                layout: transform_bind_group_layout,
                                                entries: &[wgpu::BindGroupEntry {
                                                        binding: 0,
                                                        resource: transform_buffer
                                                                .as_entire_binding(),
                                                }],
                                                label: Some(&format!(
                                                        "{} Transform Bind Group",
                                                        m.name
                                                )),
                                        });

                                Mesh {
                                        name: m.name,
                                        vertex_buffer,
                                        index_buffer,
                                        num_elements: m.indices.len() as u32,
                                        material: m.material_id.unwrap_or(0),
                                        transform_buffer,
                                        transform_bind_group,
                                }
                        })
                        .collect::<Vec<_>>();

                Model {
                        meshes: gpu_meshes,
                        materials: gpu_materials,
                }
        }
}

pub trait DrawModel<'a>
{
        fn draw_mesh(
                &mut self,
                mesh: &'a Mesh,
        );
        fn draw_mesh_instanced(
                &mut self,
                mesh: &'a Mesh,
                instances: Range<u32>,
        );
}
impl<'a, 'b> DrawModel<'b> for wgpu::RenderPass<'a>
where
        'b: 'a,
{
        fn draw_mesh(
                &mut self,
                mesh: &'b Mesh,
        )
        {
                self.draw_mesh_instanced(mesh, 0..1);
        }

        fn draw_mesh_instanced(
                &mut self,
                mesh: &'b Mesh,
                instances: Range<u32>,
        )
        {
                self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                self.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                self.draw_indexed(0..mesh.num_elements, 0, instances);
        }
}
