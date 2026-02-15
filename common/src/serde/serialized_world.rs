use color_eyre::eyre::Result;
use fxhash::FxHashMap;
use glium::{glutin::surface::WindowSurface, Display};
use petgraph::prelude::NodeIndex;
use serde::{Deserialize, Serialize};

use crate::serde::SerializedRenderable;
use crate::systems::renderer::Renderable;
use crate::world::{PhysicsContext, QuadTree, SerializedQuadTree, World, WorldGraph};
use crate::{
    light::Light,
    resources::Resources,
    serde::{serialized_background::SerializedBackground, SerializeWithContext},
};

#[derive(Serialize, Deserialize)]
pub struct SerializedWorld {
    pub title: String,
    pub graph: WorldGraph,
    pub background: SerializedBackground,
    pub lights: Vec<Light>,
    // pub terrain: Option<Terrain>,
    pub quads: SerializedQuadTree,
    pub physics_context: PhysicsContext,
    pub renderables: FxHashMap<NodeIndex, SerializedRenderable>,
    pub player_spawn: Option<NodeIndex>,
}

impl SerializedWorld {
    pub fn from_world(value: &World, resources: &Resources) -> Self {
        let serialized_renderables = value
            .renderables
            .iter()
            .map(|(node, renderable)| (*node, renderable.serialize_with(resources)))
            .collect();

        Self {
            title: value.title.clone(),
            quads: value.quads.serialize_with(resources),
            graph: value.graph.clone(),
            background: SerializedBackground::from_background(&value.background, &resources),
            lights: value.lights.clone(),
            // terrain: value.terrain.clone(),
            // serialized_models,
            physics_context: value.physics_context.clone(),
            renderables: serialized_renderables,
            player_spawn: value.player_spawn,
        }
    }

    pub fn into_world(self, display: &Display<WindowSurface>, resources: &mut Resources) -> Result<World> {
        let renderables = self
            .renderables
            .into_iter()
            .map(|(node, serialized_renderable)| {
                (
                    node,
                    Renderable::deserialize_with(serialized_renderable, display, resources),
                )
            })
            .collect();

        Ok(World {
            title: self.title,
            quads: QuadTree::deserialize_with(self.quads, display, resources),
            graph: self.graph,
            background: self.background.into_background(display, resources),
            lights: self.lights,
            // terrain_bvh: None,
            // terrain: self.terrain,
            // resources,
            lines: vec![],
            physics_context: self.physics_context,
            renderables,
            player_spawn: self.player_spawn,
        })
    }
}
