use image::GenericImageView;

#[derive(Debug)]
pub struct Texture
{
        pub texture: wgpu::Texture,
        pub view: wgpu::TextureView,
        pub sampler: wgpu::Sampler,
}

impl Texture
{
        /// Create a 1x1 white texture to use as a fallback
        pub fn create_dummy(
                device: &wgpu::Device,
                queue: &wgpu::Queue,
        ) -> Self
        {
                // A single opaque white pixel (RGBA8)
                let rgba: [u8; 4] = [255, 255, 255, 255];

                // Describe texture
                let size = wgpu::Extent3d {
                        width: 1,
                        height: 1,
                        depth_or_array_layers: 1,
                };

                let texture = device.create_texture(&wgpu::TextureDescriptor {
                        label: Some("Dummy White Texture"),
                        size,
                        mip_level_count: 1,
                        sample_count: 1,
                        dimension: wgpu::TextureDimension::D2,
                        format: wgpu::TextureFormat::Rgba8UnormSrgb,
                        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                        view_formats: &[],
                });

                // Upload the pixel
                queue.write_texture(
                        texture.as_image_copy(),
                        &rgba,
                        wgpu::ImageDataLayout {
                                offset: 0,
                                bytes_per_row: Some(4), // 4 bytes per row (RGBA8)
                                rows_per_image: Some(1),
                        },
                        size,
                );

                let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
                let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
                        label: Some("Dummy Sampler"),
                        address_mode_u: wgpu::AddressMode::ClampToEdge,
                        address_mode_v: wgpu::AddressMode::ClampToEdge,
                        address_mode_w: wgpu::AddressMode::ClampToEdge,
                        mag_filter: wgpu::FilterMode::Linear,
                        min_filter: wgpu::FilterMode::Linear,
                        mipmap_filter: wgpu::FilterMode::Nearest,
                        ..Default::default()
                });

                Self {
                        texture,
                        view,
                        sampler,
                }
        }

        pub fn from_bytes(
                device: &wgpu::Device,
                queue: &wgpu::Queue,
                bytes: &[u8],
                label: &str,
        ) -> anyhow::Result<Self>
        {
                let img = image::load_from_memory(bytes)?;
                Self::from_image(device, queue, &img, label)
        }

        pub fn from_image(
                device: &wgpu::Device,
                queue: &wgpu::Queue,
                img: &image::DynamicImage,
                label: &str,
        ) -> anyhow::Result<Self>
        {
                let rgba = img.to_rgba8();
                let dims = img.dimensions();

                let size = wgpu::Extent3d {
                        width: dims.0,
                        height: dims.1,
                        depth_or_array_layers: 1,
                };

                let texture = Self::create_texture(&device, label, size);

                Self::write_texture_to_queue(&queue, &texture, dims, &rgba, size);

                let view = Self::create_view(&texture);

                let sampler = Self::create_sampler(&device);

                Ok(Self {
                        texture,
                        view,
                        sampler,
                })
        }

        fn create_texture(
                device: &wgpu::Device,
                label: &str,
                size: wgpu::Extent3d,
        ) -> wgpu::Texture
        {
                device.create_texture(&wgpu::TextureDescriptor {
                        label: Some(label),
                        size,
                        mip_level_count: 1,
                        sample_count: 1,
                        dimension: wgpu::TextureDimension::D2,
                        format: wgpu::TextureFormat::Rgba8UnormSrgb,
                        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                        view_formats: &[],
                })
        }

        fn write_texture_to_queue(
                queue: &wgpu::Queue,
                texture: &wgpu::Texture,
                dims: (u32, u32),
                rgba: &image::ImageBuffer<image::Rgba<u8>, Vec<u8>>,
                size: wgpu::Extent3d,
        )
        {
                queue.write_texture(
                        wgpu::TexelCopyTextureInfo {
                                aspect: wgpu::TextureAspect::All,
                                texture: &texture,
                                mip_level: 0,
                                origin: wgpu::Origin3d::ZERO,
                        },
                        &rgba,
                        wgpu::TexelCopyBufferLayout {
                                offset: 0,
                                bytes_per_row: Some(4 * dims.0),
                                rows_per_image: Some(dims.1),
                        },
                        size,
                );
        }

        fn create_view(texture: &wgpu::Texture) -> wgpu::TextureView
        {
                texture.create_view(&wgpu::TextureViewDescriptor::default())
        }

        fn create_sampler(device: &wgpu::Device) -> wgpu::Sampler
        {
                device.create_sampler(&wgpu::SamplerDescriptor {
                        address_mode_u: wgpu::AddressMode::Repeat,
                        address_mode_v: wgpu::AddressMode::Repeat,
                        address_mode_w: wgpu::AddressMode::Repeat,
                        mag_filter: wgpu::FilterMode::Linear,
                        min_filter: wgpu::FilterMode::Linear,
                        mipmap_filter: wgpu::FilterMode::Linear,
                        ..Default::default()
                })
        }

        pub fn new_texture_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout
        {
                device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                        entries: &[
                                wgpu::BindGroupLayoutEntry {
                                        binding: 0,
                                        visibility: wgpu::ShaderStages::FRAGMENT,
                                        ty: wgpu::BindingType::Texture {
                                                multisampled: true,
                                                view_dimension: wgpu::TextureViewDimension::D2,
                                                sample_type: wgpu::TextureSampleType::Float {
                                                        filterable: true,
                                                },
                                        },
                                        count: None,
                                },
                                wgpu::BindGroupLayoutEntry {
                                        binding: 1,
                                        visibility: wgpu::ShaderStages::FRAGMENT,
                                        // This should match the filterable field of the
                                        // corresponding Texture entry above.
                                        ty: wgpu::BindingType::Sampler(
                                                wgpu::SamplerBindingType::Filtering,
                                        ),
                                        count: None,
                                },
                        ],
                        label: Some("texture_bind_group_layout"),
                })
        }

        pub fn new_diffuse_bind_group(
                device: &wgpu::Device,
                texture_bind_group_layout: &wgpu::BindGroupLayout,
                diffuse_texture: &Texture,
        ) -> wgpu::BindGroup
        {
                device.create_bind_group(&wgpu::BindGroupDescriptor {
                        layout: &texture_bind_group_layout,
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
                        label: Some("diffuse_bind_group"),
                })
        }

        pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

        pub fn create_depth_texture(
                device: &wgpu::Device,
                config: &wgpu::SurfaceConfiguration,
                label: &str,
        ) -> Self
        {
                let size = wgpu::Extent3d {
                        width: config.width.max(1),
                        height: config.height.max(1),
                        depth_or_array_layers: 1,
                };

                let desc = wgpu::TextureDescriptor {
                        label: Some(label),
                        size,
                        mip_level_count: 1,
                        sample_count: 1,
                        dimension: wgpu::TextureDimension::D2,
                        format: Self::DEPTH_FORMAT,
                        usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                                | wgpu::TextureUsages::TEXTURE_BINDING,
                        view_formats: &[],
                };
                let texture = device.create_texture(&desc);

                let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
                let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
                        address_mode_u: wgpu::AddressMode::ClampToEdge,
                        address_mode_v: wgpu::AddressMode::ClampToEdge,
                        address_mode_w: wgpu::AddressMode::ClampToEdge,
                        mag_filter: wgpu::FilterMode::Linear,
                        min_filter: wgpu::FilterMode::Linear,
                        mipmap_filter: wgpu::FilterMode::Nearest,
                        compare: Some(wgpu::CompareFunction::LessEqual),
                        lod_min_clamp: 0.0,
                        lod_max_clamp: 100.0,
                        ..Default::default()
                });

                Self {
                        texture,
                        view,
                        sampler,
                }
        }
}
