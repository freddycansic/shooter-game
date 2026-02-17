use crate::engine::resources::TextureHandle;
use glium::implement_vertex;
use nalgebra::{Point2, Vector2};

// #[derive(Clone, Serialize, Deserialize)]
pub struct Quad {
    pub position: Point2<f32>,
    pub size: Vector2<f32>,
    pub texture: TextureHandle,

    pub selected: bool,
    pub name: String,
}

impl Quad {
    pub fn new(position: Point2<f32>, size: Vector2<f32>, texture: TextureHandle) -> Self {
        Self {
            position,
            size,
            texture,
            selected: false,
            name: "Unnamed Quad".to_string(),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct QuadVertex {
    pub position: [f32; 2],
    pub size: [f32; 2],
    pub layer: i32,
}
implement_vertex!(QuadVertex, position, size, layer);
