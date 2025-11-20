use glium::implement_vertex;
use nalgebra::Point3;
use serde::{Deserialize, Serialize};

use crate::colors::{Color, ColorExt};

#[derive(Clone, Serialize, Deserialize)]
pub struct Light {
    pub position: Point3<f32>,
    pub color: Color,
}

impl Default for Light {
    fn default() -> Self {
        Self {
            position: Point3::new(0.0, 0.0, 0.0),
            color: Color::from_named(palette::named::WHITE),
        }
    }
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
