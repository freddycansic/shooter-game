use crate::engine::assets::{Assets, GeometryHandle, TextureHandle};
use crate::serde::serialized_handles::SerializedGeometryHandle;
use crate::serde::SerializedArchetype;
use crate::world::{PhysicsContext, QuadTree, SerializedQuadTree, World, WorldGraph};
use crate::{
    light::Light,
    serde::{serialized_background::SerializedBackground, SerializeWithContext},
};
use color_eyre::eyre::Result;
use fxhash::FxHashMap;
use glium::{glutin::surface::WindowSurface, Display};
use itertools::Itertools;
use petgraph::prelude::NodeIndex;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize)]
pub struct SerializedWorld {
    pub title: String,
    pub graph: WorldGraph,
    pub background: SerializedBackground,
    pub lights: Vec<Light>,
    // pub terrain: Option<Terrain>,
    pub quads: SerializedQuadTree,
    pub physics_context: PhysicsContext,
    pub geometries: FxHashMap<NodeIndex, SerializedGeometryHandle>,
    pub textures: FxHashMap<NodeIndex, PathBuf>,
    pub player_spawn: Option<NodeIndex>,
    pub archetypes: Vec<SerializedArchetype>,
}

impl SerializedWorld {
    pub fn from_world(value: &World, resources: &Assets) -> Self {
        let serialized_geometries = value
            .geometries
            .iter()
            .map(|(node, geometry_handle)| (*node, geometry_handle.serialize_with(resources)))
            .collect();

        let serialized_textures = value
            .textures
            .iter()
            .map(|(node, texture_handle)| (*node, texture_handle.serialize_with(resources)))
            .collect();

        // let serialized_archetypes = value.archetypes.iter().map(|archetype| archetype.ser)
        unimplemented!();

        Self {
            title: value.title.clone(),
            quads: value.quads.serialize_with(resources),
            graph: value.graph.clone(),
            background: SerializedBackground::from_background(&value.background, &resources),
            lights: value.lights.clone(),
            // terrain: value.terrain.clone(),
            physics_context: value.physics_context.clone(),

            textures: serialized_textures,
            geometries: serialized_geometries,
            player_spawn: value.player_spawn,

            archetypes: vec![], // TODO
        }
    }

    pub fn into_world(self, display: &Display<WindowSurface>, resources: &mut Assets) -> Result<World> {
        let geometries = self
            .geometries
            .into_iter()
            .map(|(node, serialized_geometry_handle)| {
                (
                    node,
                    GeometryHandle::deserialize_with(serialized_geometry_handle, display, resources),
                )
            })
            .collect_vec();

        let textures = self
            .textures
            .into_iter()
            .map(|(node, serialized_texture_handle)| {
                (
                    node,
                    TextureHandle::deserialize_with(serialized_texture_handle, display, resources),
                )
            })
            .collect_vec();

        unimplemented!();

        //Ok(World {
        //    title: self.title,
        //    quads: QuadTree::deserialize_with(self.quads, display, resources),
        //    graph: self.graph,
        //    background: self.background.into_background(display, resources),
        //    lights: self.lights,
        //    // terrain_bvh: None,
        //    // terrain: self.terrain,
        //    // resources,
        //    lines: vec![],
        //    physics_context: self.physics_context,

        //    geometries,
        //    textures,
        //    player_spawn: self.player_spawn,
        //})
    }
}
