use std::path::PathBuf;

use glium::{Display, glutin::surface::WindowSurface};
use petgraph::{graph::NodeIndex, prelude::StableDiGraph};
use serde::{Deserialize, Serialize};

use crate::{
    maths::Transform,
    resources::Resources,
    scene::graph::{NodeType, Renderable, SceneGraph, SceneNode},
};
use crate::components::component::Component;

#[derive(Serialize, Deserialize)]
struct SerializedSceneNode {
    local_transform: Transform,
    visible: bool,
    components: Vec<Component>,

    pub ty: SerializedNodeType,
}

impl SerializedSceneNode {
    fn from_scene_node(scene_node: &SceneNode, resources: &Resources) -> Self {
        let serialized_node_type = match &scene_node.ty {
            NodeType::Group => SerializedNodeType::Group,
            NodeType::Renderable(renderable) => {
                SerializedNodeType::SerializedRenderable(SerializedRenderable::from_renderable(renderable, resources))
            }
        };

        Self {
            local_transform: scene_node.local_transform.clone(),
            visible: scene_node.visible,
            ty: serialized_node_type,
            components: scene_node.components.clone(),
        }
    }

    fn into_scene_node(self, display: &Display<WindowSurface>, resources: &mut Resources) -> SceneNode {
        let node_type = match self.ty {
            SerializedNodeType::Group => NodeType::Group,
            SerializedNodeType::SerializedRenderable(serialized_renderable) => {
                NodeType::Renderable(serialized_renderable.into_renderable(resources, display))
            }
        };

        let mut node = SceneNode::new(node_type);
        node.local_transform = self.local_transform;
        node.visible = self.visible;
        node.components = self.components;

        node
    }
}

#[derive(Serialize, Deserialize)]
enum SerializedNodeType {
    Group,
    SerializedRenderable(SerializedRenderable),
}

#[derive(Serialize, Deserialize)]
struct SerializedRenderable {
    texture_path: PathBuf,
    geometry_path: PathBuf,
    mesh_index: usize,
}

impl SerializedRenderable {
    fn from_renderable(renderable: &Renderable, resources: &Resources) -> Self {
        let (geometry_path, mesh_index) = resources.get_geometry_path_and_index(renderable.geometry_handle);
        let texture_path = resources.get_texture_path(renderable.texture_handle);

        Self {
            geometry_path,
            texture_path,
            mesh_index,
        }
    }

    fn into_renderable(self, resources: &mut Resources, display: &Display<WindowSurface>) -> Renderable {
        let geometry_handle = resources.get_geometry_handles(&self.geometry_path, display).unwrap()[self.mesh_index];

        let texture_handle = resources.get_texture_handle(&self.texture_path, display).unwrap();

        Renderable {
            geometry_handle,
            texture_handle,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct SerializedSceneGraph {
    pub serialized_graph: StableDiGraph<SerializedSceneNode, ()>,
    pub root: NodeIndex,
}

impl SerializedSceneGraph {
    pub fn from_scene_graph(scene: &SceneGraph, resources: &Resources) -> Self {
        let serialized_graph = scene.graph.map(
            |_, scene_node| SerializedSceneNode::from_scene_node(scene_node, resources),
            |_, _| (),
        );

        Self {
            serialized_graph,
            root: scene.root,
        }
    }

    pub fn into_scene_graph(self, display: &Display<WindowSurface>, resources: &mut Resources) -> SceneGraph {
        let graph = self.serialized_graph.map_owned(
            |_, serialized_scene_node| serialized_scene_node.into_scene_node(display, resources),
            |_, _| (),
        );

        SceneGraph {
            graph,
            root: self.root,
            selection: vec![],
        }
    }
}
