use glium::{IndexBuffer, VertexBuffer};

use super::geometry_vertex::GeometryVertex;

#[derive(Debug)]
pub struct Primitive {
    pub vertex_buffer: VertexBuffer<GeometryVertex>,
    pub index_buffer: IndexBuffer<u32>,
    pub vertices: Vec<GeometryVertex>,
    pub indices: Vec<u32>,
}
