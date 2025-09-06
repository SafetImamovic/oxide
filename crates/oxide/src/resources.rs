use crate::geometry::mesh::{Mesh, MeshData};
use crate::material::MaterialData;
use crate::model::{Model, ModelVertex};
use cgmath::{Matrix4, Quaternion, SquareMatrix, Vector3};
use gltf::accessor::Dimensions::Vec3;
use gltf::image::Data;
use std::io::{BufReader, Cursor};
use std::ops::Index;
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

#[cfg(target_arch = "wasm32")]
fn format_url(file_name: &str) -> reqwest::Url
{
        let window = web_sys::window().unwrap();
        let location = window.location();
        let mut origin = location.origin().unwrap();
        if !origin.ends_with("learn-wgpu")
        {
                origin = format!("{}/learn-wgpu", origin);
        }
        let base = reqwest::Url::parse(&format!("{}/", origin,)).unwrap();
        base.join(file_name).unwrap()
}

#[cfg(not(target_arch = "wasm32"))]
pub fn resource_path(file_name: &str) -> std::path::PathBuf
{
        load_resources().join(file_name)
}

pub fn resource_path_relative(
        base_file: &Path,
        file_name: &str,
) -> PathBuf
{
        let base_dir = base_file.parent().unwrap_or_else(|| Path::new("."));
        base_dir.join(file_name)
}

pub async fn load_model(
        file_name: &str,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        material_bind_group_layout: &wgpu::BindGroupLayout,
        transform_bind_group_layout: &wgpu::BindGroupLayout,
) -> anyhow::Result<Model>
{
        let path = resource_path(file_name);

        let (meshes, materials) = if file_name.ends_with(".obj")
        {
                log::info!("Not implemented yet!");
                load_gltf(path.to_str().unwrap())?
        }
        else if file_name.ends_with(".gltf")
        {
                load_gltf(path.to_str().unwrap())?
        }
        else
        {
                anyhow::bail!("Unsupported format: {}", file_name);
        };

        Ok(Model::from_data(
                meshes,
                materials,
                device,
                queue,
                material_bind_group_layout,
                transform_bind_group_layout,
        ))
}

pub fn create_transform_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout
{
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                        },
                        count: None,
                }],
                label: Some("transform_bind_group_layout"),
        })
}

pub fn load_gltf(path: &str) -> anyhow::Result<(Vec<MeshData>, Vec<MaterialData>)>
{
        log::info!("Loading glTF from {:?}", path);

        let base_path = Path::new(path)
                .parent()
                .unwrap_or_else(|| Path::new("."))
                .to_path_buf();

        let (doc, buffers, images) = gltf::import(path)?;
        // Check for embedded images
        println!("Found {} embedded images", images.len());

        for (i, image) in images.iter().enumerate()
        {
                let format = match image.format
                {
                        gltf::image::Format::R8 => "R8",
                        gltf::image::Format::R8G8 => "R8G8",
                        gltf::image::Format::R8G8B8 => "R8G8B8",
                        gltf::image::Format::R8G8B8A8 => "R8G8B8A8",
                        gltf::image::Format::R16 => "R16",
                        gltf::image::Format::R16G16 => "R16G16",
                        gltf::image::Format::R16G16B16 => "R16G16B16",
                        gltf::image::Format::R16G16B16A16 => "R16G16B16A16",
                        _ => "unknown",
                };
                println!("  Image {}: - Format: {} - {} bytes", i, format, image.pixels.len());
        }

        let mut meshes = Vec::new();
        let mut materials = Vec::new();

        // Load materials first
        for mat in doc.materials()
        {
                let name = mat.name().unwrap_or("unnamed").to_string();

                let pbr = mat.pbr_metallic_roughness();

                println!("Processing material: {}", name);

                let base_color_texture = pbr.base_color_texture().map(|tex_info| {
                        let texture_index = tex_info.texture().index(); // This is the key!
                        let tex_coord = tex_info.tex_coord(); // UV set index

                        println!(
                                "  Material '{}' uses texture index: {}, UV set: {}",
                                name, texture_index, tex_coord
                        );

                        format!("{name}_baseColor.png")
                });

                // Also check if the material has any texture at all
                if base_color_texture.is_none()
                {
                        println!("Material '{}' has no base color texture", name);
                }

                log::info!("Material {}: Diffuse Texture: {:?}", name, base_color_texture);

                let normal_texture = mat
                        .normal_texture()
                        .and_then(|t| t.texture().source().name())
                        .map(|tex_name| base_path.join(tex_name).to_string_lossy().to_string());

                let metallic_roughness_texture = pbr
                        .metallic_roughness_texture()
                        .and_then(|t| t.texture().source().name())
                        .map(|tex_name| base_path.join(tex_name).to_string_lossy().to_string());

                let diffuse_texture = mat
                        .pbr_metallic_roughness()
                        .base_color_texture()
                        .and_then(|t| t.texture().source().name())
                        .map(|tex_name| base_path.join(tex_name).to_string_lossy().to_string());

                materials.push(MaterialData {
                        name,
                        diffuse_texture,
                        base_color_texture,
                        normal_texture,
                        metallic_roughness_texture,
                        base_color_factor: pbr.base_color_factor(),
                        metallic_factor: pbr.metallic_factor(),
                        roughness_factor: pbr.roughness_factor(),
                });
        }

        // Process all scenes and their node hierarchies
        for scene in doc.scenes()
        {
                for node in scene.nodes()
                {
                        process_node(&node, &buffers, &mut meshes, Matrix4::identity());
                }
        }

        Ok((meshes, materials))
}

fn process_node(
        node: &gltf::Node,
        buffers: &[gltf::buffer::Data],
        meshes: &mut Vec<MeshData>,
        parent_transform: Matrix4<f32>,
)
{
        // Calculate this node's transform
        let node_transform = parent_transform * get_node_transform(node);

        // Process mesh if this node has one
        if let Some(mesh) = node.mesh()
        {
                let mesh_name = mesh.name().unwrap_or("Unnamed").to_string();

                for (primitive_index, primitive) in mesh.primitives().enumerate()
                {
                        let reader = primitive.reader(|b| Some(&buffers[b.index()]));

                        let positions: Vec<[f32; 3]> = reader
                                .read_positions()
                                .map(|iter| iter.collect())
                                .unwrap_or_default();

                        if positions.is_empty()
                        {
                                continue;
                        }

                        let normals: Vec<[f32; 3]> = reader
                                .read_normals()
                                .map(|iter| iter.collect())
                                .unwrap_or_else(|| vec![[0.0, 0.0, 0.0]; positions.len()]);

                        let texcoords: Vec<[f32; 2]> = reader
                                .read_tex_coords(0)
                                .map(|tc| tc.into_f32().collect())
                                .unwrap_or_else(|| vec![[0.0, 0.0]; positions.len()]);

                        let indices: Vec<u32> = reader
                                .read_indices()
                                .map(|i| i.into_u32().collect())
                                .unwrap_or_else(|| (0..positions.len() as u32).collect());

                        let vertices: Vec<ModelVertex> = positions
                                .iter()
                                .enumerate()
                                .map(|(i, pos)| ModelVertex {
                                        position: *pos,
                                        normal: normals[i],
                                        tex_coords: texcoords[i],
                                })
                                .collect();

                        // Create unique name for each primitive
                        let primitive_name = if mesh.primitives().count() > 1
                        {
                                format!("{}_primitive_{}", mesh_name, primitive_index)
                        }
                        else
                        {
                                mesh_name.clone()
                        };

                        meshes.push(MeshData {
                                name: primitive_name,
                                vertices,
                                indices,
                                material_id: primitive.material().index(),
                                transform: node_transform, // Store the transform
                        });
                }
        }

        // Process child nodes recursively
        for child in node.children()
        {
                process_node(&child, buffers, meshes, node_transform);
        }
}

fn get_node_transform(node: &gltf::Node) -> Matrix4<f32>
{
        let (translation, rotation, scale) = node.transform().decomposed();

        // Convert to cgmath types
        let translation_vec = Vector3::new(translation[0], translation[1], translation[2]);
        let rotation_quat = Quaternion::new(rotation[3], rotation[0], rotation[1], rotation[2]);
        let scale_vec = Vector3::new(scale[0], scale[1], scale[2]);

        // Create transformation matrix
        Matrix4::from_translation(translation_vec)
                * Matrix4::from(rotation_quat)
                * Matrix4::from_nonuniform_scale(scale_vec[0], scale_vec[1], scale_vec[2])
}
