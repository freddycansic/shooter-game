use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

use color_eyre::eyre::Result;
use fxhash::{FxBuildHasher, FxHashMap, FxHasher};
use glium::{Display, glutin::surface::WindowSurface};

use crate::ecs::subsystem::Subsystem;
use crate::engine::assets::handle::{CubemapHandle, GeometryHandle, TextureHandle};
use crate::geometry::Geometry;
use crate::material::{Cubemap, Texture2DResource};
use crate::runtime::RuntimeContext;
use common::engine::scheduler::Scheduler;
use common::world::World;
use common_macros::Resource;
use itertools::Itertools;

#[derive(Resource)]
pub struct Assets {
    textures_handles: FxHashMap<PathBuf, TextureHandle>,
    textures: FxHashMap<TextureHandle, Texture2DResource>,
    default_texture: Option<TextureHandle>,

    geometry_handles: FxHashMap<PathBuf, Vec<GeometryHandle>>,
    geometry: FxHashMap<GeometryHandle, Geometry>,

    cubemap_handles: FxHashMap<PathBuf, CubemapHandle>,
    cubemaps: FxHashMap<CubemapHandle, Cubemap>,
}

impl Subsystem for Assets {
    fn register_resources(world: &mut World, context: Option<&RuntimeContext>) {
        let mut assets = Assets::new();
        assets.initialise_default_texture(context.unwrap().display).unwrap();

        world.register_resource(assets);
    }

    fn register_systems(_scheduler: &mut Scheduler) {}
}

// TODO for possible performance
// 2 tiered handles
// 1 is stable and derived from the path etc
// 2 is unstable, and derived at runtime
impl Assets {
    pub fn new() -> Self {
        let hasher = FxBuildHasher::default();

        Self {
            textures_handles: FxHashMap::with_hasher(hasher.clone()),
            textures: FxHashMap::with_hasher(hasher.clone()),
            default_texture: None,

            geometry_handles: FxHashMap::with_hasher(hasher.clone()),
            geometry: FxHashMap::with_hasher(hasher.clone()),

            cubemap_handles: FxHashMap::with_hasher(hasher.clone()),
            cubemaps: FxHashMap::with_hasher(hasher),
        }
    }

    pub fn initialise_default_texture(&mut self, display: &Display<WindowSurface>) -> Result<()> {
        let handle = self.get_texture_handle(&PathBuf::from("assets/textures/uv-test.jpg"), display)?;

        self.default_texture = Some(handle);

        Ok(())
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

        let mut hasher = FxHasher::default();
        path.canonicalize().unwrap().hash(&mut hasher);
        let handle = TextureHandle(hasher.finish());

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

    pub fn default_texture(&self) -> Option<TextureHandle> {
        self.default_texture.clone()
    }

    pub fn get_geometry(&self, geometry_handle: GeometryHandle) -> &Geometry {
        self.geometry
            .get(&geometry_handle)
            .expect(format!("GeometryHandle {} not loaded!", geometry_handle.0).as_str())
    }

    pub fn get_geometry_handles(
        &mut self,
        path: &Path,
        display: Option<&Display<WindowSurface>>,
    ) -> Result<Vec<GeometryHandle>> {
        if let Some(handles) = self.geometry_handles.get(path) {
            return Ok(handles.clone());
        }

        let geometries = Geometry::load(path, display)?;

        let handles = (0..geometries.len())
            .map(|index| {
                let mut hasher = FxHasher::default();
                path.canonicalize().unwrap().hash(&mut hasher);
                index.hash(&mut hasher);

                GeometryHandle(hasher.finish())
            })
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

        let mut hasher = FxHasher::default();
        path.canonicalize().unwrap().hash(&mut hasher);
        let handle = CubemapHandle(hasher.finish());

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
}
