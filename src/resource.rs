use std::collections::HashMap;

use crate::geometry::mesh::Mesh;

#[derive(Debug, Default)]
pub struct Resources<'a>
{
        pub meshes: HashMap<&'a str, Mesh<'a>>,
}

impl<'a> Resources<'a>
{
        pub fn new() -> Self
        {
                Self {
                        meshes: HashMap::new(),
                }
        }
}
