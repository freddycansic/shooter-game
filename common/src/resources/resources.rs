use std::path::PathBuf;

use color_eyre::eyre::Result;
use egui_glium::egui_winit::egui::epaint::textures;
use fxhash::{FxBuildHasher, FxHashMap, FxHasher};
use glium::{Display, glutin::surface::WindowSurface};
use gltf::json::Texture;

use crate::{resources::handle::TextureHandle, texture::Texture2D};

pub struct Resources {
    loaded_textures_handles: FxHashMap<PathBuf, TextureHandle>,
    textures: Vec<Texture2D>,
}

impl Resources {
    pub fn new() -> Self {
        let hasher = FxBuildHasher::default();

        Self {
            loaded_textures_handles: FxHashMap::with_hasher(hasher),
            textures: vec![],
        }
    }

    pub fn get_texture(&self, texture_handle: TextureHandle) -> Option<&Texture2D> {
        self.textures.get(texture_handle.0)
    }

    pub fn get_texture_handle(
        &mut self,
        path: PathBuf,
        display: &Display<WindowSurface>,
    ) -> Result<TextureHandle> {
        if let Some(handle) = self.loaded_textures_handles.get(&path) {
            return Ok(*handle);
        }

        self.textures.push(Texture2D::load(path.clone(), display)?);
        let handle = TextureHandle(self.textures.len());
        self.loaded_textures_handles.insert(path.clone(), handle);

        Ok(handle)
    }
}
