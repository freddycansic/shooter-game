use color_eyre::eyre::Result;
use glium::{Display, glutin::surface::WindowSurface};
use serde::{Deserialize, Serialize};

use crate::{
    camera::FpsCamera,
    light::Light,
    resources::resources::Resources,
    scene::{Scene, scene::Background},
    serde::{serialized_background::SerializedBackground, serialized_graph::SerializedSceneGraph},
};

#[derive(Serialize, Deserialize)]
pub struct SerializedScene {
    pub title: String,
    pub camera: FpsCamera,
    pub graph: SerializedSceneGraph,
    pub background: SerializedBackground,
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
            background: SerializedBackground::from_background(&value.background, &value.resources),
            lights: value.lights.clone(),
            // terrain: value.terrain.clone(),
            // quads: value.quads.clone(),
            // serialized_models,
        }
    }

    pub fn into_scene(self, display: &Display<WindowSurface>) -> Result<Scene> {
        let mut resources = Resources::new();

        Ok(Scene {
            title: self.title,
            camera: self.camera,
            graph: self.graph.into_scene_graph(display, &mut resources),
            background: self.background.into_background(display, &mut resources),
            lights: self.lights,
            // terrain: self.terrain,
            // quads: self.quads,
            resources,
            lines: vec![],
        })
    }
}
