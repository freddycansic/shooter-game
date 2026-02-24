use crate::collision::collidable::{RayHitNode, Sweep, SweepHitNode};
use crate::collision::colliders::sphere::Sphere;
use crate::ecs::archetype::Archetype;
use crate::ecs::component::Components;
use crate::ecs::entity::Entity;
use crate::engine::renderer::Background;
use crate::engine::resources::{GeometryHandle, Resources, TextureHandle};
use crate::light::Light;
use crate::line::Line;
use crate::maths::Ray;
use crate::serde::SerializedWorld;
use crate::world::graph::WorldGraph;
use crate::world::{PhysicsContext, QuadTree};
use fxhash::{FxHashMap, FxHasher};
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
    pub fn spawn<T: Components>(&mut self, components: T) -> Entity {
        self.find_archetype::<T>().spawn(components)
    }

    pub fn find_archetype<T: Components>(&mut self) -> &mut Archetype {
        let mut hasher = FxHasher::default();
        for id in T::ids() {
            hasher.
        }
        
        self.archetypes.entry()
        
        if let Some(index) = self
            .archetypes
            .iter()
            .position(|archetype| archetype.id == archetype_id)
        {
            return &mut self.archetypes[index];
        }

        self.archetypes.push(Archetype::new(archetype_id));
        self.archetypes.last_mut().unwrap()
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
            archetypes: vec![],
        }
    }
}
