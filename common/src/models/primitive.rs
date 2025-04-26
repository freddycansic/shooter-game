use glium::{IndexBuffer, VertexBuffer};

use super::model_vertex::ModelVertex;

#[derive(Debug)]
pub struct Primitive {
    pub vertex_buffer: VertexBuffer<ModelVertex>,
    pub index_buffer: IndexBuffer<u32>,
    pub vertices: Vec<ModelVertex>,
    pub indices: Vec<u32>,
}
