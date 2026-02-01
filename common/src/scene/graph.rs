use fxhash::{FxBuildHasher, FxHashMap};
use itertools::Itertools;
use petgraph::{
    graph::{EdgeIndex, NodeIndex},
    Direction,
};
use std::hash::{Hash, Hasher};
use nalgebra::Vector3;
use crate::collision::collidable::{Intersectable, RayHit, SweepHit};
use crate::{
    maths::Transform,
};

pub struct SceneNode {
    pub local_transform: Transform,
    world_transform: Transform,
    pub world_transform_dirty: bool,
    pub visible: bool,
}

impl SceneNode {
    pub fn world_transform(&self) -> &Transform {
        #[cfg(debug_assertions)]
        if self.world_transform_dirty {
            log::warn!("Obtaining dirty world transform.")
        }

        &self.world_transform
    }

    fn create_root() -> Self {
        Self {
            local_transform: Transform::identity(),
            world_transform: Transform::identity(),
            world_transform_dirty: false,
            visible: false,
        }
    }
}

impl Default for SceneNode {
    fn default() -> Self {
        Self {
            local_transform: Transform::identity(),
            world_transform: Transform::identity(),
            world_transform_dirty: false,
            visible: true,
        }
    }
}

pub struct SceneGraph {
    pub graph: petgraph::stable_graph::StableDiGraph<SceneNode, ()>,
    pub root: NodeIndex,
    pub selection: Vec<NodeIndex>,
}

impl SceneGraph {
    pub fn new() -> Self {
        let mut graph = petgraph::stable_graph::StableDiGraph::<SceneNode, ()>::new();
        let root = graph.add_node(SceneNode::create_root());

        Self {
            graph,
            root,
            selection: vec![],
        }
    }

    pub fn add_edge(&mut self, a: NodeIndex, b: NodeIndex) -> EdgeIndex {
        self.graph.add_edge(a, b, ())
    }

    pub fn add_node(&mut self, node: SceneNode) -> NodeIndex {
        self.graph.add_node(node)
    }

    pub fn add_root_node(&mut self, node: SceneNode) -> NodeIndex {
        let node_index = self.graph.add_node(node);

        self.graph.add_edge(self.root, node_index, ());

        node_index
    }

    pub fn calculate_world_matrices(&mut self) {
        self.calculate_world_matrices_inner(self.root);
    }

    fn calculate_world_matrices_inner(&mut self, parent: NodeIndex) {
        let children = self.graph.neighbors_directed(parent, Direction::Outgoing).collect_vec();

        for child in children.into_iter() {
            self.graph[child].world_transform = self.graph[parent]
                .world_transform
                .combine(&self.graph[child].local_transform);

            self.graph[child].world_transform_dirty = false;

            self.calculate_world_matrices_inner(child);
        }
    }
}
