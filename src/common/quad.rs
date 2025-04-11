use crate::texture::Texture2D;
use crate::ui::selectable::Selectable;
use cgmath::{Point2, Vector2};
use glium::implement_vertex;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone, Serialize, Deserialize, Selectable)]
pub struct Quad {
    pub position: Point2<f32>,
    pub size: Vector2<f32>,
    pub texture: Arc<Texture2D>,
    // Higher layer = closer to camera
    pub layer: i32,

    pub selected: bool,
}

impl Quad {
    pub fn new(
        position: Point2<f32>,
        size: Vector2<f32>,
        texture: Arc<Texture2D>,
        layer: i32,
    ) -> Self {
        Self {
            position,
            size,
            texture,
            layer,
            selected: false,
        }
    }
}

#[derive(Copy, Clone)]
pub struct QuadVertex {
    pub position: [f32; 2],
    pub size: [f32; 2],
    pub layer: i32,
}
implement_vertex!(QuadVertex, position, size, layer);

impl From<Quad> for QuadVertex {
    fn from(value: Quad) -> Self {
        QuadVertex {
            position: <[f32; 2]>::from(value.position),
            size: <[f32; 2]>::from(value.size),
            layer: value.layer,
        }
    }
}
