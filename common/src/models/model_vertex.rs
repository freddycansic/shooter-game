use glium::implement_vertex;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct ModelVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub tex_coord: [f32; 2],
}

impl Default for ModelVertex {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0, 0.0],
            normal: [0.0, 0.0, 0.0],
            tex_coord: [0.0, 0.0],
        }
    }
}

implement_vertex!(ModelVertex, position, normal, tex_coord);
