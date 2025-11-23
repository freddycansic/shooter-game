use std::path::PathBuf;

use color_eyre::Result;
use glium::Display;
use glium::glutin::surface::WindowSurface;
use glium::texture::CompressedTexture2d;

use crate::texture::texture;

pub struct Texture2D {
    pub path: PathBuf,
    pub inner_texture: CompressedTexture2d,
}

impl Texture2D {
    pub fn load(path: PathBuf, display: &Display<WindowSurface>) -> Result<Self> {
        let raw_image = texture::load_raw_image(&path)?;
        let opengl_texture = CompressedTexture2d::new(display, raw_image).unwrap();

        Ok(Texture2D {
            inner_texture: opengl_texture,
            path: path.clone(),
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
        .unwrap();

        Ok(Texture2D {
            inner_texture: opengl_texture,
            path: PathBuf::new(),
        })
    }
}
