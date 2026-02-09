use color_eyre::eyre::Result;
use glium::{Display, glutin::surface::WindowSurface};
use serde::{Deserialize, Serialize};

use crate::world::{SerializedQuadTree, World, WorldGraph};
use crate::{
    light::Light,
    resources::Resources,
    serde::{SerializeWithContext, serialized_background::SerializedBackground},
};

#[derive(Serialize, Deserialize)]
pub struct SerializedWorld {
    pub title: String,
    pub graph: WorldGraph,
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
            graph: value.graph.clone(),
            background: SerializedBackground::from_background(&value.background, &resources),
            lights: value.lights.clone(),
            // terrain: value.terrain.clone(),
            // serialized_models,
        }
    }

    pub fn into_world(self, _display: &Display<WindowSurface>) -> Result<World> {
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
