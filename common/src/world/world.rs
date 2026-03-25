use crate::collision::collidable::{RayHitNode, Sweep, SweepHitNode};
use crate::collision::colliders::sphere::Sphere;
use crate::ecs::archetype::{Archetype, Column};
use crate::ecs::component::StableComponentId;
use crate::ecs::entity::Entity;
use crate::engine::renderer::Background;
use crate::engine::resources::{GeometryHandle, Resources, TextureHandle};
use crate::light::Light;
use crate::line::Line;
use crate::maths::Ray;
use crate::serde::SerializedWorld;
use crate::world::graph::WorldGraph;
use crate::world::{PhysicsContext, QuadTree};
use common::ecs::component;
use common::ecs::owned_components::OwnedComponents;
use fxhash::FxHashMap;
use itertools::Itertools;
use petgraph::prelude::NodeIndex;
use rfd::FileDialog;

pub struct World {
    pub title: String,
    pub lines: Vec<Line>,
    pub quads: QuadTree,
    pub background: Background,
    pub graph: WorldGraph,
    pub lights: Vec<Light>,

    // Components
    pub geometries: FxHashMap<NodeIndex, GeometryHandle>,
    pub textures: FxHashMap<NodeIndex, TextureHandle>,
    pub player_spawn: Option<NodeIndex>,
    pub physics_context: PhysicsContext,

    pub archetypes: FxHashMap<u64, Archetype>,
}

impl World {
    pub fn spawn<T: OwnedComponents>(&mut self, components: T) -> Entity {
        self.find_exact_archetype(&T::ids()).spawn(components)
    }

    /// Finds the single archetype matching T exactly, creates it if it does not exist.
    pub fn find_exact_archetype(&mut self, ids: &[StableComponentId]) -> &mut Archetype {
        let archetype_id = component::archetype_id(ids);

        self.archetypes.entry(archetype_id).or_insert_with_key(|id| Archetype {
            id: *id,
            entities: vec![],
            columns: ids.iter().cloned().map(Column::new_empty).collect_vec(),
        })
    }

    /// Finds all archetypes which are a superset of T
    pub fn find_superset_archetypes(&mut self, ids: &[StableComponentId]) -> Vec<&mut Archetype> {
        let mut superset_archetypes = Vec::new();

        for archetype in self.archetypes.values_mut() {
            if archetype
                .columns
                .iter()
                .all(|column| ids.binary_search(&column.id).is_ok())
            {
                superset_archetypes.push(archetype);
            }
        }

        superset_archetypes
    }

    pub fn raycast(&self, ray: &Ray, resources: &Resources) -> Option<RayHitNode> {
        self.physics_context.raycast(ray, &self.graph, resources)
    }

    pub fn spherecast(&self, sphere: &Sweep<Sphere>, resources: &Resources) -> Option<SweepHitNode> {
        self.physics_context.spherecast(sphere, &self.graph, resources)
    }

    pub fn save_as(&self, resources: &Resources) {
        let serialized_world = SerializedWorld::from_world(self, resources);

        let serialized = serde_json::to_string(&serialized_world).unwrap();

        std::thread::spawn(move || {
            if let Some(save_path) = FileDialog::new().save_file() {
                std::fs::write(save_path, serialized).unwrap();
            }
        });
    }
}

impl Default for World {
    fn default() -> Self {
        Self {
            title: "Untitled".to_string(),
            background: Background::default(),
            quads: QuadTree::new(),
            lines: vec![],
            graph: WorldGraph::new(),
            lights: vec![],
            physics_context: PhysicsContext::new(),

            geometries: FxHashMap::default(),
            textures: FxHashMap::default(),
            player_spawn: None,
            archetypes: FxHashMap::default(),
        }
    }
}
