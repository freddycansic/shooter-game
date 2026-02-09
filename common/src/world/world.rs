use crate::light::Light;
use crate::line::Line;
use crate::systems::renderer::{Background, Renderable};
use crate::world::graph::WorldGraph;
use crate::world::{PhysicsContext, QuadTree};
use fxhash::FxHashMap;
use petgraph::prelude::NodeIndex;
use crate::collision::collidable::{RayHit, RayHitNode};
use crate::maths::Ray;
use crate::resources::Resources;

pub type Renderables = FxHashMap<NodeIndex, Renderable>;

pub struct World {
    pub title: String,
    pub renderables: Renderables,
    pub lines: Vec<Line>,
    pub quads: QuadTree,
    pub background: Background,
    pub graph: WorldGraph,
    pub lights: Vec<Light>,
    pub physics_context: PhysicsContext,
}

impl World {
    pub fn raycast(&self, ray: &Ray, resources: &Resources) -> Option<RayHitNode> {
        self.physics_context.raycast(ray, &self.graph, resources)
    }
}

impl Default for World {
    fn default() -> Self {
        Self {
            title: "Untitled".to_string(),
            renderables: Renderables::default(),
            background: Background::default(),
            quads: QuadTree::new(),
            lines: vec![],
            graph: WorldGraph::new(),
            lights: vec![],
            physics_context: PhysicsContext::new(),
        }
    }
}