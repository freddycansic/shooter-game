use serde::{Deserialize, Serialize};

use crate::models::{Primitive, model_vertex::ModelVertex};

#[derive(Serialize, Deserialize)]
pub struct SerializedPrimitive {
    pub vertices: Vec<ModelVertex>,
    pub indices: Vec<u32>,
}

impl From<&Primitive> for SerializedPrimitive {
    fn from(value: &Primitive) -> Self {
        Self {
            vertices: value.vertices.clone(),
            indices: value.indices.clone(),
        }
    }
}
