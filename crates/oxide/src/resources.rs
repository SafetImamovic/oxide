use crate::geometry::mesh::MeshData;
use crate::material::MaterialData;
use crate::model::{Model, ModelVertex};
use cgmath::{Matrix4, Quaternion, SquareMatrix, Vector3};
use std::path::{Path, PathBuf};

#[cfg(not(target_arch = "wasm32"))]
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
pub fn load_resources() -> PathBuf
{
        PathBuf::from("/resources/")
}

#[cfg(not(target_arch = "wasm32"))]
pub fn resource_path(
        file_name: &str,
        #[allow(unused_variables)] crate_name: Option<&str>,
) -> PathBuf
{
        load_resources().join(file_name)
}

#[cfg(target_arch = "wasm32")]
pub fn resource_path(
        file_name: &str,
        #[allow(unused_variables)] crate_name: Option<&str>,
) -> String
{
        if file_name.starts_with('/')
        {
                return file_name.to_string();
        }

        format!("/resources/{}", file_name)
}

pub fn resource_path_relative(
        base_file: &Path,
        file_name: &str,
) -> PathBuf
{
        let base_dir = base_file.parent().unwrap_or_else(|| Path::new("."));
        base_dir.join(file_name)
}

/// Main function that is responsible for loading in 3D Models.
pub async fn load_model(
        file_name: &str,
        crate_name: Option<&str>,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        material_bind_group_layout: &wgpu::BindGroupLayout,
        transform_bind_group_layout: &wgpu::BindGroupLayout,
) -> anyhow::Result<Model>
{
        #[cfg(not(target_arch = "wasm32"))]
        let path = resource_path(file_name, crate_name)
                .to_string_lossy()
                .to_string();

        #[cfg(target_arch = "wasm32")]
        let path = resource_path(file_name, crate_name);

        let (meshes, materials, images) = if file_name.ends_with(".obj")
        {
                anyhow::bail!("OBJ format not supported yet.");
        }
        else if file_name.ends_with(".glb")
        {
                load_gltf(&path, crate_name).await?
        }
        else
        {
                anyhow::bail!("Unsupported format: {}", file_name);
        };

        Ok(Model::from_data(
                meshes,
                materials,
                images,
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

pub async fn load_gltf(
        path: &str,
        crate_name: Option<&str>,
) -> anyhow::Result<(Vec<MeshData>, Vec<MaterialData>, Vec<gltf::image::Data>)>
{
        log::info!("Loading 3D model from: {:?}", path);

        let (doc, buffers, images) = if path.ends_with(".glb")
        {
                load_glb(path, crate_name).await?
        }
        else
        {
                anyhow::bail!("Unsupported format: {}", path);
        };

        println!("Found {} embedded images", images.len());

        let mut meshes = Vec::new();
        let mut materials = Vec::new();

        for mat in doc.materials()
        {
                let name = mat.name().unwrap_or("unnamed").to_string();
                let pbr = mat.pbr_metallic_roughness();

                println!("Processing material: {}", name);

                // Extract texture indices from the GLTF material
                let base_color_texture_index = pbr
                        .base_color_texture()
                        .map(|tex_info| tex_info.texture().index());

                let metallic_roughness_texture_index = pbr
                        .metallic_roughness_texture()
                        .map(|tex_info| tex_info.texture().index());

                let normal_texture_index = mat
                        .normal_texture()
                        .map(|tex_info| tex_info.texture().index());

                materials.push(MaterialData {
                        name: name.clone(),
                        base_color_texture: None, // This can probably be removed
                        base_color_factor: pbr.base_color_factor(),
                        metallic_factor: pbr.metallic_factor(),
                        roughness_factor: pbr.roughness_factor(),
                        base_color_texture_index, // Now this will have the actual index!
                        normal_texture_index,     // Now this will have the actual index!
                        diffuse_texture: None,    // This can probably be removed
                        normal_texture: None,     // This can probably be removed
                        metallic_roughness_texture: None, // This can probably be removed
                        metallic_roughness_texture_index, // Now this will have the actual index!
                });
        }

        for scene in doc.scenes()
        {
                for node in scene.nodes()
                {
                        process_node(&node, &buffers, &mut meshes, Matrix4::identity());
                }
        }

        Ok((meshes, materials, images))
}

async fn load_glb(
        path: &str,
        #[allow(unused_variables)] crate_name: Option<&str>,
) -> anyhow::Result<(gltf::Document, Vec<gltf::buffer::Data>, Vec<gltf::image::Data>)>
{
        #[cfg(target_arch = "wasm32")]
        {
                use wasm_bindgen::JsCast;
                use web_sys::Response;

                let window =
                        web_sys::window().ok_or_else(|| anyhow::anyhow!("No window available"))?;

                let full_path = resource_path(path, crate_name);
                log::info!("Fetching GLB from: {}", full_path);

                let resp_value =
                        wasm_bindgen_futures::JsFuture::from(window.fetch_with_str(&full_path))
                                .await
                                .map_err(|e| anyhow::anyhow!("Failed to fetch GLB: {:?}", e))?;

                let resp: Response = resp_value
                        .dyn_into()
                        .map_err(|e| anyhow::anyhow!("Failed to convert to Response: {:?}", e))?;

                if !resp.ok()
                {
                        return Err(anyhow::anyhow!("HTTP error: {}", resp.status()));
                }

                let array_buffer =
                        wasm_bindgen_futures::JsFuture::from(resp.array_buffer().map_err(|e| {
                                anyhow::anyhow!("Failed to get array buffer: {:?}", e)
                        })?)
                        .await
                        .map_err(|e| anyhow::anyhow!("Failed to await array buffer: {:?}", e))?;

                let bytes = js_sys::Uint8Array::new(&array_buffer).to_vec();

                // Import GLB - this should work since everything is embedded
                gltf::import_slice(&bytes)
                        .map_err(|e| anyhow::anyhow!("Failed to import GLB: {:?}", e))
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
                // Native loading - simple file read
                let bytes = std::fs::read(path)?;
                gltf::import_slice(&bytes)
                        .map_err(|e| anyhow::anyhow!("Failed to import GLB: {:?}", e))
        }
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
