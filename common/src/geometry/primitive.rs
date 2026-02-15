use glium::{IndexBuffer, VertexBuffer};

use super::geometry_vertex::GeometryVertex;

#[derive(Debug)]
pub struct Primitive {
    pub cpu: PrimitiveCpu,
    pub gpu: Option<PrimitiveGpu>,
}

#[derive(Debug)]
pub struct PrimitiveCpu {
    pub vertices: Vec<GeometryVertex>,
    pub indices: Vec<u32>,
}

#[derive(Debug)]
pub struct PrimitiveGpu {
    pub vertex_buffer: VertexBuffer<GeometryVertex>,
    pub index_buffer: IndexBuffer<u32>,
}
