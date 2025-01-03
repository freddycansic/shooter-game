use crate::texture::Texture2D;
use color_eyre::eyre::Result;
use glium::glutin::surface::WindowSurface;
use glium::Display;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Serialize, Deserialize, Clone, Eq, Hash, PartialEq)]
pub struct Material {
    pub diffuse: Arc<Texture2D>,
    pub specular: Arc<Texture2D>,
}

impl Material {
    pub fn default(display: &Display<WindowSurface>) -> Result<Self> {
        let default_diffuse = Texture2D::default_diffuse(display)?;
        let (width, height) = default_diffuse.inner_texture.as_ref().unwrap().dimensions();

        Ok(Self {
            diffuse: default_diffuse,
            specular: Texture2D::solid(width, height, display)?,
        })
    }
}
