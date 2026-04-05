use std::path::PathBuf;

use crate::engine::assets::{GeometryHandle, Assets, TextureHandle};
use crate::serde::SerializeWithContext;
use glium::{Display, glutin::surface::WindowSurface};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct SerializedGeometryHandle {
    path: PathBuf,
    mesh_index: usize,
}

impl SerializeWithContext for GeometryHandle {
    type Serialized = SerializedGeometryHandle;

    fn serialize_with(&self, resources: &Assets) -> Self::Serialized {
        let (path, mesh_index) = resources.get_geometry_path_and_index(self.clone());

        Self::Serialized { path, mesh_index }
    }

    fn deserialize_with(
        serialized: Self::Serialized,
        display: &Display<WindowSurface>,
        resources: &mut Assets,
    ) -> Self {
        resources.get_geometry_handles(&serialized.path, Some(display)).unwrap()[serialized.mesh_index]
    }
}

impl SerializeWithContext for TextureHandle {
    type Serialized = PathBuf;

    fn serialize_with(&self, resources: &Assets) -> Self::Serialized {
        resources.get_texture_path(self.clone())
    }

    fn deserialize_with(
        serialized: Self::Serialized,
        display: &Display<WindowSurface>,
        resources: &mut Assets,
    ) -> Self {
        resources.get_texture_handle(&serialized, display).unwrap()
    }
}
