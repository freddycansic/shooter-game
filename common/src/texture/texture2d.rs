use std::hash::{Hash, Hasher};
use std::path::PathBuf;

use color_eyre::Result;
use fxhash::FxHasher;
use glium::Display;
use glium::glutin::surface::WindowSurface;
use glium::texture::CompressedTexture2d;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::texture::texture;
use crate::texture::texture::TextureLoadError;

#[derive(Serialize, Deserialize, Debug)]
pub struct Texture2D {
    pub uuid: Uuid,
    pub path: PathBuf,
    #[serde(skip)]
    pub inner_texture: Option<CompressedTexture2d>,
}

impl Texture2D {
    pub fn load(path: PathBuf, display: &Display<WindowSurface>) -> Result<Self> {
        let raw_image = texture::load_raw_image(&path)?;
        let opengl_texture = CompressedTexture2d::new(display, raw_image)
            .map_err(TextureLoadError::CreateTextureError)?;

        Ok(Texture2D {
            inner_texture: Some(opengl_texture),
            path: path.clone(),
            uuid: Uuid::new_v4(),
        })
    }

    pub fn default_diffuse(display: &Display<WindowSurface>) -> Result<Self> {
        Self::load(PathBuf::from("assets/textures/uv-test.jpg"), display)
    }

    pub fn solid(width: u32, height: u32, display: &Display<WindowSurface>) -> Result<Self> {
        let value = 255 / 2;

        let opengl_texture = CompressedTexture2d::new(
            display,
            vec![vec![(value / 255, value / 255, value / 255); height as usize]; width as usize],
        )
        .map_err(TextureLoadError::CreateTextureError)?;

        Ok(Texture2D {
            inner_texture: Some(opengl_texture),
            path: PathBuf::new(),
            uuid: Uuid::new_v4(),
        })
    }
}

impl PartialEq<Self> for Texture2D {
    fn eq(&self, other: &Self) -> bool {
        self.uuid == other.uuid
    }
}

impl Eq for Texture2D {}

impl Hash for Texture2D {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let mut hasher = FxHasher::default();
        self.uuid.hash(&mut hasher);

        let result = hasher.finish();
        state.write_u64(result);
    }
}
