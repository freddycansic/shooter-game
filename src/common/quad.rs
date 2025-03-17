use crate::texture::Texture2D;
use cgmath::{Point2, Vector2};
use glium::implement_vertex;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone, Serialize, Deserialize)]
pub struct Quad {
    pub position: Point2<f32>,
    pub size: Vector2<f32>,
    pub texture: Arc<Texture2D>,
}

#[derive(Copy, Clone)]
pub struct QuadVertex {
    pub position: [f32; 2],
    pub size: [f32; 2],
}
implement_vertex!(QuadVertex, position, size);

impl From<Quad> for QuadVertex {
    fn from(value: Quad) -> Self {
        QuadVertex {
            position: <[f32; 2]>::from(value.position),
            size: <[f32; 2]>::from(value.size),
        }
    }
}
