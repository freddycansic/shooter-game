use crate::colors::{Color, ColorExt};
use cgmath::Point3;
use glium::implement_vertex;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct Light {
    pub position: Point3<f32>,
    pub color: Color,
}

#[derive(Copy, Clone)]
pub struct ShaderLight {
    pub light_translation: [f32; 3],
    pub light_color: [f32; 3],
}
implement_vertex!(ShaderLight, light_translation, light_color);

impl From<Light> for ShaderLight {
    fn from(light: Light) -> Self {
        Self {
            light_translation: <[f32; 3]>::from(light.position),
            light_color: <[f32; 3]>::from(light.color.to_rgb_vector3()),
        }
    }
}
