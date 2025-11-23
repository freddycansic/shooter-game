use std::sync::Arc;

use color_eyre::eyre::Result;
use fxhash::{FxBuildHasher, FxHashMap};
use glium::{
    Display, IndexBuffer, VertexBuffer, glutin::surface::WindowSurface, index::PrimitiveType,
};
use petgraph::prelude::StableDiGraph;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    camera::FpsCamera,
    geometry::Primitive,
    light::Light,
    quad::Quad,
    resources::resources::Resources,
    scene::{Scene, graph::SceneGraph, scene::Background},
    serde::serialized_graph::SerializedSceneGraph,
};

#[derive(Serialize, Deserialize)]
pub struct SerializedScene {
    pub title: String,
    pub camera: FpsCamera,
    pub graph: SerializedSceneGraph,
    pub background: Background,
    pub lights: Vec<Light>,
    // pub terrain: Option<Terrain>,
    // pub quads: StableDiGraph<Quad, ()>,
    // pub serialized_models: FxHashMap<Uuid, SerializedModel>,
    // pub serialized_materials: FxHashMap<Uuid, SerializedMa
}

impl SerializedScene {
    pub fn from_scene(value: &Scene) -> Self {
        Self {
            title: value.title.clone(),
            camera: value.camera.clone(),
            graph: SerializedSceneGraph::from_scene_graph(&value.graph, &value.resources),
            background: value.background.clone(),
            lights: value.lights.clone(),
            // terrain: value.terrain.clone(),
            // quads: value.quads.clone(),
            // serialized_models,
        }
    }

    pub fn into_scene(self, display: &Display<WindowSurface>) -> Result<Scene> {
        let mut resources = Resources::new();
        let graph = self.graph.into_scene_graph(display, &mut resources);

        Ok(Scene {
            title: self.title,
            camera: self.camera,
            graph,
            background: self.background,
            lights: self.lights,
            // terrain: self.terrain,
            // quads: self.quads,
            resources,
            lines: vec![],
        })
    }
}
