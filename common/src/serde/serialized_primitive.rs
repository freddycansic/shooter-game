use serde::{Deserialize, Serialize};

use crate::geometry::{GeometryVertex, Primitive};

#[derive(Serialize, Deserialize)]
pub struct SerializedPrimitive {
    pub vertices: Vec<GeometryVertex>,
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
