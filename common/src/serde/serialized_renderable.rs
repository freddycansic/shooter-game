use std::path::PathBuf;

use glium::{Display, glutin::surface::WindowSurface};
use petgraph::graph::NodeIndex;
use serde::{Deserialize, Serialize};

use crate::serde::SerializeWithContext;
use crate::systems::renderer::Renderable;
use crate::resources::Resources;

#[derive(Serialize, Deserialize)]
pub struct SerializedRenderable {
    texture_path: PathBuf,
    geometry_path: PathBuf,
    mesh_index: usize,
    node: NodeIndex,
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
            node: self.node,
        }
    }

    fn deserialize_with(
        serialized: Self::Serialized,
        display: &Display<WindowSurface>,
        resources: &mut Resources,
    ) -> Self {
        let geometry_handle = resources
            .get_geometry_handles(&serialized.geometry_path, display)
            .unwrap()[serialized.mesh_index];

        let texture_handle = resources.get_texture_handle(&serialized.texture_path, display).unwrap();

        Renderable {
            geometry_handle,
            texture_handle,
            node: serialized.node,
        }
    }
}
