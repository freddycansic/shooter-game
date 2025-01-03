use crate::texture::texture;
use crate::texture::texture::TextureLoadError;
use color_eyre::Result;
use glium::glutin::surface::WindowSurface;
use glium::texture::CompressedTexture2d;
use glium::Display;
use memoize::memoize;
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub struct Texture2D {
    #[serde(with = "crate::serde::uuid")]
    pub uuid: Uuid,
    pub path: PathBuf,
    #[serde(skip)]
    pub inner_texture: Option<CompressedTexture2d>,
}

impl Texture2D {
    pub fn load(path: PathBuf, display: &Display<WindowSurface>) -> Result<Arc<Self>> {
        Ok(load(path, display)?)
    }

    pub fn default_diffuse(display: &Display<WindowSurface>) -> Result<Arc<Self>> {
        Self::load(PathBuf::from("assets/textures/uv-test.jpg"), display)
    }

    pub fn solid(width: u32, height: u32, display: &Display<WindowSurface>) -> Result<Arc<Self>> {
        Ok(solid_grey_texture(255 / 2, width, height, display)?)
    }
}

#[memoize(Ignore: display)]
fn solid_grey_texture(
    // This must be integral as f32 cannot implement Eq
    value: u8,
    width: u32,
    height: u32,
    display: &Display<WindowSurface>,
) -> Result<Arc<Texture2D>, TextureLoadError> {
    let opengl_texture = CompressedTexture2d::new(
        display,
        vec![vec![(value / 255, value / 255, value / 255); height as usize]; width as usize],
    )
    .map_err(TextureLoadError::CreateTextureError)?;

    Ok(Arc::new(Texture2D {
        inner_texture: Some(opengl_texture),
        path: PathBuf::new(),
        uuid: Uuid::new_v4(),
    }))
}

#[memoize(Ignore: display)]
fn load(
    path: PathBuf,
    display: &Display<WindowSurface>,
) -> Result<Arc<Texture2D>, TextureLoadError> {
    let raw_image = texture::load_raw_image(&path)?;
    let opengl_texture = CompressedTexture2d::new(display, raw_image)
        .map_err(TextureLoadError::CreateTextureError)?;

    Ok(Arc::new(Texture2D {
        inner_texture: Some(opengl_texture),
        path: path.clone(),
        uuid: Uuid::new_v4(),
    }))
}

impl PartialEq<Self> for Texture2D {
    fn eq(&self, other: &Self) -> bool {
        self.uuid == other.uuid
    }
}

impl Eq for Texture2D {}

impl Hash for Texture2D {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.uuid.hash(state)
    }
}
