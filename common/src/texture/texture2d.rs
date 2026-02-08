use std::path::{Path, PathBuf};

use color_eyre::Result;
use glium::Display;
use glium::glutin::surface::WindowSurface;
use glium::texture::CompressedTexture2d;

use crate::texture::texture;

pub struct Texture2DResource {
    pub path: PathBuf,
    pub inner_texture: CompressedTexture2d,
}

impl Texture2DResource {
    pub fn load(path: &Path, display: &Display<WindowSurface>) -> Result<Self> {
        let raw_image = texture::load_raw_image(path)?;
        let opengl_texture = CompressedTexture2d::new(display, raw_image).unwrap();

        Ok(Texture2DResource {
            inner_texture: opengl_texture,
            path: path.to_path_buf(),
        })
    }

    pub fn default_diffuse(display: &Display<WindowSurface>) -> Result<Self> {
        Self::load(Path::new("assets/textures/uv-test.jpg"), display)
    }

    pub fn solid(width: u32, height: u32, display: &Display<WindowSurface>) -> Result<Self> {
        let value = 255 / 2;

        let opengl_texture = CompressedTexture2d::new(
            display,
            vec![vec![(value / 255, value / 255, value / 255); height as usize]; width as usize],
        )
        .unwrap();

        Ok(Texture2DResource {
            inner_texture: opengl_texture,
            path: PathBuf::new(),
        })
    }
}
