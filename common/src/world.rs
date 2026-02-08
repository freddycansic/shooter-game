use crate::light::Light;
use crate::line::Line;
use crate::quad::Quad;
use crate::scene::graph::SceneGraph;
use crate::scene::QuadTree;
use crate::systems::renderer::{Background, Renderable};

pub struct World {
    pub title: String,
    pub renderables: Vec<Renderable>,
    pub lines: Vec<Line>,
    pub quads: QuadTree,
    pub background: Background,
    pub graph: SceneGraph,
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
            graph: SceneGraph::new(),
            lights: vec![],
        }
    }
}