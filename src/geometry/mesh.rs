use crate::geometry::vertex::Vertex;

#[derive(Debug)]
pub struct Mesh
{
        pub vertices: &'static [Vertex],
        pub indices: &'static [u16],
}
