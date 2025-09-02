use crate::geometry::mesh::Mesh;

#[derive(Debug, Default)]
pub struct Resources
{
        pub meshes: Vec<Mesh>,
}

impl Resources
{
        pub fn new() -> Self
        {
                Self {
                        meshes: Vec::new(),
                }
        }

        pub fn add_mesh(
                &mut self,
                mesh: Mesh,
        )
        {
                self.meshes.push(mesh);
        }

        pub fn upload_all(
                &mut self,
                device: &wgpu::Device,
        )
        {
                log::info!("Uploading all resources...");

                for mesh in self.meshes.iter_mut()
                {
                        if mesh.needs_upload()
                        {
                                mesh.upload(&device, wgpu::BufferUsages::COPY_DST);
                        }
                }
        }
}
