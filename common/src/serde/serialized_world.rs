use color_eyre::eyre::Result;
use glium::{Display, glutin::surface::WindowSurface};
use serde::{Deserialize, Serialize};

use crate::{
    camera::FpsCamera,
    light::Light,
    resources::Resources,
    serde::{
        SerializeWithContext, serialized_background::SerializedBackground, serialized_graph::SerializedSceneGraph,
    },
};
use crate::scene::SerializedQuadTree;
use crate::world::World;

#[derive(Serialize, Deserialize)]
pub struct SerializedWorld {
    pub title: String,
    pub graph: SerializedSceneGraph,
    pub background: SerializedBackground,
    pub lights: Vec<Light>,
    // pub terrain: Option<Terrain>,
    pub quads: SerializedQuadTree,
}

impl SerializedWorld {
    pub fn from_world(value: &World, resources: &Resources) -> Self {
        Self {
            title: value.title.clone(),
            quads: value.quads.serialize_with(&resources),
            graph: SerializedSceneGraph::from_world_graph(&value.graph, &resources),
            background: SerializedBackground::from_background(&value.background, &resources),
            lights: value.lights.clone(),
            // terrain: value.terrain.clone(),
            // serialized_models,
        }
    }

    pub fn into_world(self, display: &Display<WindowSurface>) -> Result<World> {
        unimplemented!()
        // let mut resources = Resources::new();
        //
        // Ok(Scene {
        //     title: self.title,
        //     quads: QuadTree::deserialize_with(self.quads, display, &mut resources),
        //     graph: self.graph.into_world_graph(display, &mut resources),
        //     background: self.background.into_background(display, &mut resources),
        //     lights: self.lights,
        //     terrain_bvh: None,
        //     // terrain: self.terrain,
        //     resources,
        //     lines: vec![],
        // })
    }
}
