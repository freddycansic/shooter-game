use crate::models::{Primitive, model_vertex::ModelVertex};

pub struct SerializedPrimitive {
    pub vertices: Vec<ModelVertex>,
    pub indices: Vec<u32>,
}

impl From<Primitive> for SerializedPrimitive {
    fn from(value: Primitive) -> Self {
        Self {
            vertices: value.vertices,
            indices: value.indices,
        }
    }
}
