use crate::collision::collidable::{RayHitNode, Sweep, SweepHitNode};
use crate::collision::colliders::sphere::Sphere;
use crate::engine::renderer::{Background, Renderable};
use crate::engine::resources::Resources;
use crate::light::Light;
use crate::line::Line;
use crate::maths::Ray;
use crate::serde::SerializedWorld;
use crate::world::graph::WorldGraph;
use crate::world::{PhysicsContext, QuadTree};
use fxhash::FxHashMap;
use petgraph::prelude::NodeIndex;
use rfd::FileDialog;

pub type Renderables = FxHashMap<NodeIndex, Renderable>;

pub struct World {
    pub title: String,
    pub lines: Vec<Line>,
    pub quads: QuadTree,
    pub background: Background,
    pub graph: WorldGraph,
    pub lights: Vec<Light>,

    // Components
    pub renderables: Renderables,
    pub player_spawn: Option<NodeIndex>,
    pub physics_context: PhysicsContext,
}

impl World {
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

            renderables: Renderables::default(),
            player_spawn: None,
        }
    }
}
