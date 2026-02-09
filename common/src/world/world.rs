use crate::light::Light;
use crate::line::Line;
use crate::systems::renderer::{Background, Renderable};
use crate::world::graph::WorldGraph;
use crate::world::{PhysicsContext, QuadTree};
use fxhash::FxHashMap;
use petgraph::prelude::NodeIndex;

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
    pub fn new() -> Self {
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
