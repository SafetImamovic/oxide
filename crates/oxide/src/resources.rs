use crate::geometry::mesh::Mesh;
use crate::model::{Model, ModelVertex};
use std::io::{BufReader, Cursor};
use std::path::{Path, PathBuf};
use wgpu::util::DeviceExt;

pub fn load_resources() -> PathBuf
{
        if let Ok(dir) = std::env::var("CARGO_MANIFEST_DIR")
        {
                let res = Path::new(&dir).join("resources");
                if res.exists()
                {
                        return res;
                }
        }

        if let Ok(dir) = std::env::var("EXAMPLE_RESOURCES")
        {
                return dir.into();
        }

        panic!("No resources folder found!");
}

#[cfg(not(target_arch = "wasm32"))]
fn resource_path(file_name: &str) -> std::path::PathBuf
{
        load_resources().join(file_name)
}

pub async fn load_string(file_name: &str) -> anyhow::Result<String>
{
        #[cfg(target_arch = "wasm32")]
        let txt = {
                let url = format_url(file_name);
                reqwest::get(url).await?.text().await?
        };

        #[cfg(not(target_arch = "wasm32"))]
        let txt = std::fs::read_to_string(resource_path(file_name))?;

        Ok(txt)
}

pub async fn load_binary(file_name: &str) -> anyhow::Result<Vec<u8>>
{
        #[cfg(target_arch = "wasm32")]
        let data = {
                let url = format_url(file_name);
                reqwest::get(url).await?.bytes().await?.to_vec()
        };

        #[cfg(not(target_arch = "wasm32"))]
        let data = std::fs::read(resource_path(file_name))?;

        Ok(data)
}

pub async fn load_texture(
        file_name: &str,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
) -> anyhow::Result<crate::texture::Texture>
{
        let data = load_binary(file_name).await?;
        crate::texture::Texture::from_bytes(device, queue, &data, file_name)
}

pub async fn load_model(
        file_name: &str,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        layout: &wgpu::BindGroupLayout,
) -> anyhow::Result<crate::model::Model>
{
        let obj_text = load_string(file_name).await?;
        let obj_cursor = Cursor::new(obj_text);
        let mut obj_reader = BufReader::new(obj_cursor);

        let (models, obj_materials) = tobj::load_obj_buf_async(
                &mut obj_reader,
                &tobj::LoadOptions {
                        triangulate: true,
                        single_index: true,
                        ..Default::default()
                },
                |p| async move {
                        let mat_text = load_string(&p).await.unwrap();
                        tobj::load_mtl_buf(&mut BufReader::new(Cursor::new(mat_text)))
                },
        )
        .await?;

        let mut materials = Vec::new();
        for m in obj_materials?
        {
                let diffuse_texture =
                        load_texture(&m.diffuse_texture.unwrap(), device, queue).await?;
                let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                        layout,
                        entries: &[
                                wgpu::BindGroupEntry {
                                        binding: 0,
                                        resource: wgpu::BindingResource::TextureView(
                                                &diffuse_texture.view,
                                        ),
                                },
                                wgpu::BindGroupEntry {
                                        binding: 1,
                                        resource: wgpu::BindingResource::Sampler(
                                                &diffuse_texture.sampler,
                                        ),
                                },
                        ],
                        label: None,
                });

                materials.push(crate::material::Material {
                        name: m.name,
                        diffuse_texture,
                        bind_group,
                })
        }

        let meshes = models
                .into_iter()
                .map(|m| {
                        let vertices = (0..m.mesh.positions.len() / 3)
                                .map(|i| {
                                        if m.mesh.normals.is_empty()
                                        {
                                                ModelVertex {
                                                        position: [
                                                                m.mesh.positions[i * 3],
                                                                m.mesh.positions[i * 3 + 1],
                                                                m.mesh.positions[i * 3 + 2],
                                                        ],
                                                        tex_coords: [
                                                                m.mesh.texcoords[i * 2],
                                                                1.0 - m.mesh.texcoords[i * 2 + 1],
                                                        ],
                                                        normal: [0.0, 0.0, 0.0],
                                                }
                                        }
                                        else
                                        {
                                                ModelVertex {
                                                        position: [
                                                                m.mesh.positions[i * 3],
                                                                m.mesh.positions[i * 3 + 1],
                                                                m.mesh.positions[i * 3 + 2],
                                                        ],
                                                        tex_coords: [
                                                                m.mesh.texcoords[i * 2],
                                                                1.0 - m.mesh.texcoords[i * 2 + 1],
                                                        ],
                                                        normal: [
                                                                m.mesh.normals[i * 3],
                                                                m.mesh.normals[i * 3 + 1],
                                                                m.mesh.normals[i * 3 + 2],
                                                        ],
                                                }
                                        }
                                })
                                .collect::<Vec<_>>();

                        let vertex_buffer =
                                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                                        label: Some(&format!("{:?} Vertex Buffer", file_name)),
                                        contents: bytemuck::cast_slice(&vertices),
                                        usage: wgpu::BufferUsages::VERTEX,
                                });
                        let index_buffer =
                                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                                        label: Some(&format!("{:?} Index Buffer", file_name)),
                                        contents: bytemuck::cast_slice(&m.mesh.indices),
                                        usage: wgpu::BufferUsages::INDEX,
                                });

                        Mesh {
                                name: file_name.to_string(),
                                vertex_buffer,
                                index_buffer,
                                num_elements: m.mesh.indices.len() as u32,
                                material: m.mesh.material_id.unwrap_or(0),
                        }
                })
                .collect::<Vec<_>>();

        Ok(Model {
                meshes,
                materials,
        })
}
