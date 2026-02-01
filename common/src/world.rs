use crate::line::Line;
use crate::systems::renderer::{Background, Renderable};

pub struct World {
    pub renderables: Vec<Renderable>,
    pub lines: Vec<Line>,
    // pub quads: Vec<Quad>,
    pub background: Background,
}

impl World {
    pub fn new() -> Self {
        Self {
            renderables: vec![],
            background: Background::default(),
            lines: vec![],
        }
    }
}