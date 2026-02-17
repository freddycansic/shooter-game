use std::path::PathBuf;

use glium::{glutin::surface::WindowSurface, Display};
use serde::{Deserialize, Serialize};
use crate::engine::renderer::Renderable;
use crate::engine::resources::Resources;
use crate::serde::SerializeWithContext;

#[derive(Serialize, Deserialize)]
pub struct SerializedRenderable {
    texture_path: PathBuf,
    geometry_path: PathBuf,
    mesh_index: usize,
}

impl SerializeWithContext for Renderable {
    type Serialized = SerializedRenderable;

    fn serialize_with(&self, resources: &Resources) -> Self::Serialized {
        let (geometry_path, mesh_index) = resources.get_geometry_path_and_index(self.geometry_handle);
        let texture_path = resources.get_texture_path(self.texture_handle);

        Self::Serialized {
            geometry_path,
            texture_path,
            mesh_index,
        }
    }

    fn deserialize_with(
        serialized: Self::Serialized,
        display: &Display<WindowSurface>,
        resources: &mut Resources,
    ) -> Self {
        let geometry_handle = resources
            .get_geometry_handles(&serialized.geometry_path, Some(display))
            .unwrap()[serialized.mesh_index];

        let texture_handle = resources.get_texture_handle(&serialized.texture_path, display).unwrap();

        Renderable {
            geometry_handle,
            texture_handle,
        }
    }
}
