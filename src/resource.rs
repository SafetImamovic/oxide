use std::collections::HashMap;

use crate::geometry::mesh::Mesh;

#[derive(Debug, Default)]
pub struct Resources
{
        pub meshes: HashMap<String, Mesh>,
}

impl Resources
{
        pub fn new() -> Self
        {
                Self {
                        meshes: HashMap::new(),
                }
        }

        pub fn add_mesh(
                &mut self,
                name: &str,
                mesh: Mesh,
        )
        {
                self.meshes.insert(name.to_string(), mesh);
        }
}
