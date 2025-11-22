use std::path::PathBuf;

use color_eyre::eyre::Result;
use fxhash::{FxBuildHasher, FxHashMap};
use glium::{Display, glutin::surface::WindowSurface};

use itertools::Itertools;

use crate::{
    geometry::Geometry,
    resources::handle::{GeometryHandle, TextureHandle},
    texture::Texture2D,
};

pub struct Resources {
    textures_handles: FxHashMap<PathBuf, TextureHandle>,
    textures: Vec<Texture2D>,

    geometry_handles: FxHashMap<PathBuf, Vec<GeometryHandle>>,
    geometry: Vec<Vec<Geometry>>,
}

impl Resources {
    pub fn new() -> Self {
        let hasher = FxBuildHasher::default();

        Self {
            textures_handles: FxHashMap::with_hasher(hasher.clone()),
            textures: vec![],

            geometry_handles: FxHashMap::with_hasher(hasher),
            geometry: vec![],
        }
    }

    pub fn get_texture(&self, texture_handle: TextureHandle) -> &Texture2D {
        self.textures
            .get(texture_handle.0)
            .expect(format!("TextureHandle {} not loaded!", texture_handle.0).as_str())
    }

    pub fn get_texture_handle(
        &mut self,
        path: PathBuf,
        display: &Display<WindowSurface>,
    ) -> Result<TextureHandle> {
        if let Some(handle) = self.textures_handles.get(&path) {
            return Ok(*handle);
        }

        log::info!("Loading texture {:?}...", path);

        let handle = TextureHandle(self.textures.len());

        self.textures.push(Texture2D::load(path.clone(), display)?);
        self.textures_handles.insert(path.clone(), handle);

        Ok(handle)
    }

    pub fn get_geometry(&self, geometry_handle: GeometryHandle) -> &Geometry {
        let (scene_handle, mesh_handle) = geometry_handle.0;

        self.geometry
            .get(scene_handle)
            .and_then(|scene_geometry| scene_geometry.get(mesh_handle))
            .expect(
                format!(
                    "GeometryHandle ({}, {}) not loaded!",
                    geometry_handle.0.0, geometry_handle.0.1
                )
                .as_str(),
            )
    }

    pub fn get_geometry_handles(
        &mut self,
        path: PathBuf,
        display: &Display<WindowSurface>,
    ) -> Result<Vec<GeometryHandle>> {
        if let Some(handles) = self.geometry_handles.get(&path) {
            return Ok(handles.clone());
        }

        let geometries = Geometry::load(path.clone(), display)?;

        let scene_handle = self.geometry.len();
        let handles = (0..geometries.len())
            .map(|mesh_index| GeometryHandle((scene_handle, mesh_index)))
            .collect_vec();

        self.geometry.push(geometries);
        self.geometry_handles.insert(path.clone(), handles.clone());

        Ok(handles)
    }
}
