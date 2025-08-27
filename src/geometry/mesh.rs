use crate::geometry::vertex::Vertex;

#[derive(Debug)]
pub struct Mesh<'a>
{
        pub vertices: &'a [Vertex],
        pub indices: &'a [u16],
}
