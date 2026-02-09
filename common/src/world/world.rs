use crate::light::Light;
use crate::line::Line;
use crate::quad::Quad;
use crate::systems::renderer::{Background, Renderable};
use crate::world::QuadTree;
use crate::world::graph::WorldGraph;

pub struct World {
    pub title: String,
    pub renderables: Vec<Renderable>,
    pub lines: Vec<Line>,
    pub quads: QuadTree,
    pub background: Background,
    pub graph: WorldGraph,
    pub lights: Vec<Light>,
}

impl World {
    pub fn new() -> Self {
        Self {
            title: "Untitled".to_string(),
            renderables: vec![],
            background: Background::default(),
            quads: QuadTree::new(),
            lines: vec![],
            graph: WorldGraph::new(),
            lights: vec![],
        }
    }
}
