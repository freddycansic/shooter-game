use color_eyre::eyre::Result;
use glium::{Display, glutin::surface::WindowSurface};
use serde::{Deserialize, Serialize};

use crate::{
    camera::FpsCamera,
    light::Light,
    resources::Resources,
    scene::{QuadTree, Scene, SerializedQuadTree},
    serde::{
        SerializeWithContext, serialized_background::SerializedBackground, serialized_graph::SerializedSceneGraph,
    },
};

#[derive(Serialize, Deserialize)]
pub struct SerializedScene {
    pub title: String,
    pub graph: SerializedSceneGraph,
    pub background: SerializedBackground,
    pub lights: Vec<Light>,
    // pub terrain: Option<Terrain>,
    pub quads: SerializedQuadTree,
}

impl SerializedScene {
    pub fn from_scene(value: &Scene) -> Self {
        Self {
            title: value.title.clone(),
            quads: value.quads.serialize_with(&value.resources),
            graph: SerializedSceneGraph::from_scene_graph(&value.graph, &value.resources),
            background: SerializedBackground::from_background(&value.background, &value.resources),
            lights: value.lights.clone(),
            // terrain: value.terrain.clone(),
            // serialized_models,
        }
    }

    pub fn into_scene(self, display: &Display<WindowSurface>) -> Result<Scene> {
        let mut resources = Resources::new();

        Ok(Scene {
            title: self.title,
            quads: QuadTree::deserialize_with(self.quads, display, &mut resources),
            graph: self.graph.into_scene_graph(display, &mut resources),
            background: self.background.into_background(display, &mut resources),
            lights: self.lights,
            terrain_bvh: None,
            // terrain: self.terrain,
            resources,
            lines: vec![],
        })
    }
}
