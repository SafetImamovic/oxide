use crate::geometry::mesh::{Mesh, MeshData};
use crate::material::{MaterialData, MaterialProperties};
use crate::resources::create_transform_bind_group_layout;
use cgmath::{Deg, EuclideanSpace, Euler, InnerSpace, Quaternion, Rad, Rotation3, Vector3};
use std::ops::Range;
use std::time::Duration;
use wgpu::util::DeviceExt;
use wgpu::{BindGroupDescriptor, BindGroupEntry};

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

pub trait Transform
{
        fn calculate_transform(&self) -> cgmath::Matrix4<f32>;
}

#[derive(Debug)]
pub struct Model
{
        pub position: cgmath::Point3<f32>,
        pub rotation: Quaternion<f32>,
        pub euler_angles: [f32; 3],
        pub rotation_speeds: [f32; 3],
        pub is_spinning: bool,
        pub scale: Vector3<f32>,
        pub meshes: Vec<Mesh>,
        pub materials: Vec<crate::material::Material>,
}

impl Transform for Model
{
        fn calculate_transform(&self) -> cgmath::Matrix4<f32>
        {
                let translation = cgmath::Matrix4::from_translation(self.position.to_vec());
                let rotation = cgmath::Matrix4::from(self.rotation);
                let scale = cgmath::Matrix4::from_nonuniform_scale(
                        self.scale.x,
                        self.scale.y,
                        self.scale.z,
                );

                translation * rotation * scale
        }
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
                            // R8G8 format (2 bytes per pixel) - use the appropriate texture format
                            // Convert to RGBA if needed, or use a two-channel format
                            let mut rgba_data = Vec::with_capacity(image.pixels.len() * 2);
                            for chunk in image.pixels.chunks_exact(2) {
                                rgba_data.extend_from_slice(chunk);
                                rgba_data.push(0); // Add the blue channel
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
                    let padded_bytes_per_row = unpadded_bytes_per_row.div_ceil(align) * align;

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

            let material_bind_group = device.create_bind_group(&BindGroupDescriptor {
                layout: material_bind_group_layout,
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&base_color_texture.view),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&base_color_texture.sampler),
                    },
                    BindGroupEntry {
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

                log::info!("from_data Called!");

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

                                let transform_buffer = Self::create_transform_buffer(device, &m);
                                let transform_bind_group = Self::create_transform_bind_group(
                                        device,
                                        transform_bind_group_layout,
                                        &transform_buffer,
                                        &m,
                                );

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
                        position: cgmath::Point3::new(0.0, 0.0, 0.0),
                        rotation: Quaternion::new(1.0, 0.0, 0.0, 0.0),
                        euler_angles: [0.0, 0.0, 0.0],
                        rotation_speeds: [0.0, 0.0, 0.0],
                        is_spinning: false,
                        scale: Vector3::new(1.0, 1.0, 1.0),
                        meshes: gpu_meshes,
                        materials: gpu_materials,
                }
        }

        pub fn create_transform_buffer(
                device: &wgpu::Device,
                m: &MeshData,
        ) -> wgpu::Buffer
        {
                let transform_data: [[f32; 4]; 4] = m.transform.into();

                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Transform Buffer"),
                        contents: bytemuck::cast_slice(&transform_data),
                        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                })
        }

        pub fn create_model_transform_buffer(
                &self,
                device: &wgpu::Device,
        ) -> wgpu::Buffer
        {
                let transform_data: [[f32; 4]; 4] = self.calculate_transform().into();

                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Transform Buffer"),
                        contents: bytemuck::cast_slice(&transform_data),
                        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                })
        }

        pub fn create_model_transform_bind_group(
                &self,
                device: &wgpu::Device,
        ) -> wgpu::BindGroup
        {
                let layout = create_transform_bind_group_layout(device);

                device.create_bind_group(&BindGroupDescriptor {
                        label: Some("model_transform_bind_group"),
                        layout: &layout,
                        entries: &[BindGroupEntry {
                                binding: 0,
                                resource: self
                                        .create_model_transform_buffer(device)
                                        .as_entire_binding(),
                        }],
                })
        }

        pub fn create_transform_bind_group(
                device: &wgpu::Device,
                transform_bind_group_layout: &wgpu::BindGroupLayout,
                transform_buffer: &wgpu::Buffer,
                m: &MeshData,
        ) -> wgpu::BindGroup
        {
                device.create_bind_group(&BindGroupDescriptor {
                        layout: transform_bind_group_layout,
                        entries: &[BindGroupEntry {
                                binding: 0,
                                resource: transform_buffer.as_entire_binding(),
                        }],
                        label: Some(&format!("{} Transform Bind Group", m.name)),
                })
        }

        pub fn rotation_ui(
                &mut self,
                ui: &mut egui::Ui,
        )
        {
                ui.heading("Rotation Controls");

                // Euler angle editor (only affects initial orientation)
                ui.collapsing("Euler Angles (Initial)", |ui| {
                        ui.add(egui::DragValue::new(&mut self.euler_angles[0])
                                .speed(1.0)
                                .suffix("°"));
                        ui.add(egui::DragValue::new(&mut self.euler_angles[1])
                                .speed(1.0)
                                .suffix("°"));
                        ui.add(egui::DragValue::new(&mut self.euler_angles[2])
                                .speed(1.0)
                                .suffix("°"));

                        self.rotation = Quaternion::from(Euler::new(
                                Rad::from(Deg(self.euler_angles[0])),
                                Rad::from(Deg(self.euler_angles[1])),
                                Rad::from(Deg(self.euler_angles[2])),
                        ))
                        .normalize();
                });

                // Continuous rotation controls
                ui.collapsing("Continuous Rotation", |ui| {
                        ui.checkbox(&mut self.is_spinning, "Enable Spinning");

                        ui.label("Rotation Speeds (deg/sec):");

                        ui.add(egui::Slider::new(&mut self.rotation_speeds[0], -720.0..=720.0)
                                .text("X Speed"));
                        ui.add(egui::Slider::new(&mut self.rotation_speeds[1], -720.0..=720.0)
                                .text("Y Speed"));
                        ui.add(egui::Slider::new(&mut self.rotation_speeds[2], -720.0..=720.0)
                                .text("Z Speed"));

                        if ui.button("Reset Speeds").clicked()
                        {
                                self.rotation_speeds = [0.0, 0.0, 0.0];
                        }
                });

                // Current status
                ui.collapsing("Current Status", |ui| {
                        let euler = Euler::from(self.rotation);
                        ui.label(format!(
                                "Current Euler: X: {:.1}°, Y: {:.1}°, Z: {:.1}°",
                                euler.x.0.to_degrees(),
                                euler.y.0.to_degrees(),
                                euler.z.0.to_degrees()
                        ));

                        ui.label(format!(
                                "Rotation Speeds: X: {:.1}°/s, Y: {:.1}°/s, Z: {:.1}°/s",
                                self.rotation_speeds[0],
                                self.rotation_speeds[1],
                                self.rotation_speeds[2]
                        ));
                });
        }

        pub fn ui(
                &mut self,
                ui: &mut egui::Ui,
        )
        {
                egui::CollapsingHeader::new("Model")
                        .default_open(true)
                        .show(ui, |ui| {
                                ui.label("Position");
                                ui.add(egui::DragValue::new(&mut self.position.x));
                                ui.add(egui::DragValue::new(&mut self.position.y));
                                ui.add(egui::DragValue::new(&mut self.position.z));

                                self.rotation_ui(ui);

                                ui.label("Scale");
                                ui.add(egui::DragValue::new(&mut self.scale.x).speed(0.001));
                                ui.add(egui::DragValue::new(&mut self.scale.y).speed(0.001));
                                ui.add(egui::DragValue::new(&mut self.scale.z).speed(0.001));
                        });
        }

        pub fn update(
                &mut self,
                dt: &Duration,
        )
        {
                if !self.is_spinning
                {
                        return;
                }

                let delta_seconds = dt.as_secs_f32();

                // Apply continuous rotation
                let x_rot = Quaternion::from_axis_angle(
                        Vector3::unit_x(),
                        Rad::from(Deg(self.rotation_speeds[0] * delta_seconds)),
                );

                let y_rot = Quaternion::from_axis_angle(
                        Vector3::unit_y(),
                        Rad::from(Deg(self.rotation_speeds[1] * delta_seconds)),
                );

                let z_rot = Quaternion::from_axis_angle(
                        Vector3::unit_z(),
                        Rad::from(Deg(self.rotation_speeds[2] * delta_seconds)),
                );

                self.rotation = (z_rot * y_rot * x_rot * self.rotation).normalize();

                self.update_euler_from_quat();
        }

        pub fn toggle_spin(&mut self)
        {
                self.is_spinning = !self.is_spinning;
        }

        fn update_euler_from_quat(&mut self)
        {
                // Convert quaternion to Euler angles
                let euler = Euler::from(self.rotation);

                // Convert radians to degrees and store
                self.euler_angles = [
                        euler.x.0.to_degrees(),
                        euler.y.0.to_degrees(),
                        euler.z.0.to_degrees(),
                ];

                self.normalize_euler_angles();
        }

        fn normalize_euler_angles(&mut self)
        {
                for angle in &mut self.euler_angles
                {
                        // Normalize to -180 to 180 range
                        *angle %= 360.0;
                        if *angle > 180.0
                        {
                                *angle -= 360.0;
                        }
                        else if *angle < -180.0
                        {
                                *angle += 360.0;
                        }
                }
        }

        pub fn set_rotation_speed(
                &mut self,
                axis: usize,
                speed_deg_per_sec: f32,
        )
        {
                if axis < 3
                {
                        self.rotation_speeds[axis] = speed_deg_per_sec;
                }
        }

        // Get Euler angles from quaternion (for demonstration)
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
