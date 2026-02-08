use std::path::{Path, PathBuf};

use color_eyre::eyre::Result;
use fxhash::{FxBuildHasher, FxHashMap};
use glium::{Display, glutin::surface::WindowSurface};

use itertools::Itertools;

use crate::{
    geometry::Geometry,
    resources::handle::{CubemapHandle, GeometryHandle, TextureHandle},
    texture::{Cubemap, Texture2DResource},
};

pub struct Resources {
    textures_handles: FxHashMap<PathBuf, TextureHandle>,
    textures: FxHashMap<TextureHandle, Texture2DResource>,

    geometry_handles: FxHashMap<PathBuf, Vec<GeometryHandle>>,
    geometry: FxHashMap<GeometryHandle, Geometry>,

    cubemap_handles: FxHashMap<PathBuf, CubemapHandle>,
    cubemaps: FxHashMap<CubemapHandle, Cubemap>,

    handle_count: usize,
}

impl Resources {
    pub fn new() -> Self {
        let hasher = FxBuildHasher::default();

        Self {
            textures_handles: FxHashMap::with_hasher(hasher.clone()),
            textures: FxHashMap::with_hasher(hasher.clone()),

            geometry_handles: FxHashMap::with_hasher(hasher.clone()),
            geometry: FxHashMap::with_hasher(hasher.clone()),

            cubemap_handles: FxHashMap::with_hasher(hasher.clone()),
            cubemaps: FxHashMap::with_hasher(hasher),

            handle_count: 0,
        }
    }

    pub fn get_texture(&self, texture_handle: TextureHandle) -> &Texture2DResource {
        self.textures
            .get(&texture_handle)
            .expect(format!("TextureHandle {} not loaded!", texture_handle.0).as_str())
    }

    pub fn get_texture_handle(&mut self, path: &Path, display: &Display<WindowSurface>) -> Result<TextureHandle> {
        if let Some(handle) = self.textures_handles.get(path) {
            return Ok(*handle);
        }

        log::info!("Loading texture {:?}...", path);

        let handle = TextureHandle(self.new_handle());

        self.textures.insert(handle, Texture2DResource::load(path, display)?);
        self.textures_handles.insert(path.to_path_buf(), handle);

        Ok(handle)
    }

    pub fn get_texture_path(&self, texture_handle: TextureHandle) -> PathBuf {
        self.textures_handles
            .iter()
            .find(|(_, handle)| **handle == texture_handle)
            .unwrap()
            .0
            .clone()
    }

    pub fn get_geometry(&self, geometry_handle: GeometryHandle) -> &Geometry {
        self.geometry
            .get(&geometry_handle)
            .expect(format!("GeometryHandle {} not loaded!", geometry_handle.0).as_str())
    }

    pub fn get_geometry_handles(
        &mut self,
        path: &Path,
        display: &Display<WindowSurface>,
    ) -> Result<Vec<GeometryHandle>> {
        if let Some(handles) = self.geometry_handles.get(path) {
            return Ok(handles.clone());
        }

        let geometries = Geometry::load(path.to_path_buf(), display)?;

        let handles = (0..geometries.len())
            .map(|_| GeometryHandle(self.new_handle()))
            .collect_vec();

        for (geometry, handle) in geometries.into_iter().zip(handles.clone()) {
            self.geometry.insert(handle, geometry);
        }

        self.geometry_handles.insert(path.to_path_buf(), handles.clone());

        Ok(handles)
    }

    pub fn get_geometry_path_and_index(&self, geometry_handle: GeometryHandle) -> (PathBuf, usize) {
        for (path, handles) in self.geometry_handles.iter() {
            if let Some(index) = handles.iter().position(|h| *h == geometry_handle) {
                return (path.clone(), index);
            }
        }

        panic!("Path not found for handle, something very bad has happened");
    }

    pub fn get_cubemap(&self, cubemap_handle: CubemapHandle) -> &Cubemap {
        self.cubemaps
            .get(&cubemap_handle)
            .expect(format!("CubemapHandle {} not loaded!", cubemap_handle.0).as_str())
    }

    pub fn get_cubemap_handle(&mut self, path: &PathBuf, display: &Display<WindowSurface>) -> Result<CubemapHandle> {
        if let Some(handle) = self.cubemap_handles.get(path) {
            return Ok(*handle);
        }

        log::info!("Loading cubemap {:?}...", path);

        let handle = CubemapHandle(self.new_handle());

        self.cubemaps.insert(handle, Cubemap::load(path.clone(), display)?);
        self.cubemap_handles.insert(path.clone(), handle);

        Ok(handle)
    }

    pub fn get_cubemap_path(&self, cubemap_handle: CubemapHandle) -> PathBuf {
        self.cubemap_handles
            .iter()
            .find(|(_, handle)| **handle == cubemap_handle)
            .unwrap()
            .0
            .clone()
    }

    fn new_handle(&mut self) -> usize {
        self.handle_count += 1;

        self.handle_count
    }
}
