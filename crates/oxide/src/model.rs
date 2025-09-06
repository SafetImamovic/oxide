use crate::geometry::mesh::{Mesh, MeshData};
use crate::material::{MaterialData, MaterialProperties};
use crate::resources::resource_path;
use std::ops::Range;
use std::path::PathBuf;
use wgpu::util::DeviceExt;
use wgpu::BindGroupDescriptor;

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
                use std::mem;
                wgpu::VertexBufferLayout {
                        array_stride: mem::size_of::<ModelVertex>() as wgpu::BufferAddress,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &[
                                wgpu::VertexAttribute {
                                        offset: 0,
                                        shader_location: 0,
                                        format: wgpu::VertexFormat::Float32x3,
                                },
                                wgpu::VertexAttribute {
                                        offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                                        shader_location: 1,
                                        format: wgpu::VertexFormat::Float32x2,
                                },
                                wgpu::VertexAttribute {
                                        offset: mem::size_of::<[f32; 5]>() as wgpu::BufferAddress,
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
                device: &wgpu::Device,
                queue: &wgpu::Queue,
                material_bind_group_layout: &wgpu::BindGroupLayout,
                transform_bind_group_layout: &wgpu::BindGroupLayout,
        ) -> Self
        {
                // Upload materials
                let gpu_materials = materials
                        .into_iter()
                        .map(|mat| {
                                // If the material has a texture path, load it, otherwise use a
                                // dummy 1x1 texture
                            let base_color_texture = if let Some(ref path) = mat.base_color_texture {
                                println!("Attempting to load texture: {}", path);
                                let full_path = resource_path(PathBuf::from(format!("textures\\{}", path)).to_str().unwrap(), Some("de_dust2"));
                                println!("Full path: {:?}", full_path);

                                #[cfg(not(target_arch = "wasm32"))]
                                if !full_path.exists() {
                                    println!("❌ TEXTURE FILE NOT FOUND: {:?}", full_path);
                                    crate::texture::Texture::create_dummy(device, queue)
                                } else {
                                    match crate::texture::Texture::from_bytes(
                                        device,
                                        queue,
                                        &std::fs::read(&full_path).unwrap(),
                                        &full_path.to_string_lossy(),
                                    ) {
                                        Ok(texture) => {
                                            println!("✅ Texture loaded successfully: {:?}", texture );
                                            texture
                                        }
                                        Err(e) => {
                                            println!("❌ Texture failed to load: {:?}", e);
                                            crate::texture::Texture::create_dummy(device, queue)
                                        }
                                    }
                                }


                                #[cfg(target_arch = "wasm32")]
                                crate::texture::Texture::create_dummy(device, queue)

                            } else {
                                crate::texture::Texture::create_dummy(device, queue)
                            };

                                // Load normal texture (optional)
                                let normal_texture = if let Some(path) = mat.normal_texture
                                {
                                        Some(crate::texture::Texture::from_bytes(
                                                device,
                                                queue,
                                                &std::fs::read(resource_path(&path, Some("de_dust2"))).unwrap(),
                                                &path,
                                        )
                                        .unwrap())
                                }
                                else
                                {
                                        None
                                };

                                // Load metallic roughness texture (optional)
                                let metallic_roughness_texture = if let Some(path) =
                                        mat.metallic_roughness_texture
                                {
                                        Some(crate::texture::Texture::from_bytes(
                                                device,
                                                queue,
                                                &std::fs::read(resource_path(&path, Some("de_dust2"))).unwrap(),
                                                &path,
                                        )
                                        .unwrap())
                                }
                                else
                                {
                                        None
                                };

                                // Create material properties uniform
                                let material_properties = MaterialProperties {
                                        base_color_factor: mat.base_color_factor,
                                        metallic_factor: mat.metallic_factor,
                                        roughness_factor: mat.roughness_factor,
                                        _padding: [0.0; 2],
                                };

                                let material_properties_buffer = device.create_buffer_init(
                                        &wgpu::util::BufferInitDescriptor {
                                                label: Some("Material Properties Buffer"),
                                                contents: bytemuck::cast_slice(&[
                                                        material_properties,
                                                ]),
                                                usage: wgpu::BufferUsages::UNIFORM,
                                        },
                                );

                                let material_bind_group =  device.create_bind_group(&BindGroupDescriptor {
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

                // Upload meshes
                let gpu_meshes = meshes
                        .into_iter()
                        .map(|m| {
                                // Create vertex and index buffers (existing code)
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

                                // Create transform buffer - convert cgmath Matrix4 to bytes
                                let transform_data: [[f32; 4]; 4] = [
                                        [
                                                m.transform.x.x,
                                                m.transform.x.y,
                                                m.transform.x.z,
                                                m.transform.x.w,
                                        ],
                                        [
                                                m.transform.y.x,
                                                m.transform.y.y,
                                                m.transform.y.z,
                                                m.transform.y.w,
                                        ],
                                        [
                                                m.transform.z.x,
                                                m.transform.z.y,
                                                m.transform.z.z,
                                                m.transform.z.w,
                                        ],
                                        [
                                                m.transform.w.x,
                                                m.transform.w.y,
                                                m.transform.w.z,
                                                m.transform.w.w,
                                        ],
                                ];

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
