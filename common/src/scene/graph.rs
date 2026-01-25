use fxhash::{FxBuildHasher, FxHashMap};
use itertools::Itertools;
use petgraph::{
    Direction,
    graph::{EdgeIndex, NodeIndex},
};
use std::hash::{Hash, Hasher};

use crate::collision::collidable::{RayHit, Intersectable};
use crate::maths::Ray;
use crate::resources::Resources;
use crate::{
    maths::Transform,
    renderer::Instance,
    resources::{GeometryHandle, TextureHandle},
};

pub enum NodeType {
    Renderable(Renderable),
    Group,
}

pub struct SceneNode {
    pub local_transform: Transform,
    world_transform: Transform,
    pub world_transform_dirty: bool,
    pub visible: bool,

    pub ty: NodeType,
}

impl SceneNode {
    pub fn new_visible(ty: NodeType, local_transform: Transform) -> Self {
        Self::new(ty, local_transform, true)
    }

    pub fn new(ty: NodeType, local_transform: Transform, visible: bool) -> Self {
        Self {
            local_transform,
            world_transform: Transform::identity(),
            world_transform_dirty: true,
            visible,
            ty,
        }
    }

    pub fn intersect_ray(&self, ray: &Ray, resources: &mut Resources) -> Option<RayHit> {
        match &self.ty {
            NodeType::Renderable(renderable) => {
                let geometry = resources.get_geometry(renderable.geometry_handle);

                // geometry bvh is in local space, incoming ray is world space
                let local_ray = {
                    let world_inverse = self.world_transform.matrix().inverse();

                    let local_origin = world_inverse * ray.origin;
                    let local_direction = world_inverse.transform_vector(&ray.direction()).normalize();

                    Ray::new(local_origin, local_direction)
                };

                geometry.bvh.intersect_ray(&local_ray)
            }
            NodeType::Group => None,
        }
    }

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
            ty: NodeType::Group,
        }
    }
}

pub struct Renderable {
    pub geometry_handle: GeometryHandle,
    pub texture_handle: TextureHandle,
}

#[derive(Clone)]
pub struct GeometryBatchKey {
    pub geometry_handle: GeometryHandle,
    pub texture_handle: TextureHandle,
    pub selected: bool,
}

impl Hash for GeometryBatchKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.geometry_handle.hash(state);
        self.texture_handle.hash(state);
        self.selected.hash(state);
    }
}

impl PartialEq for GeometryBatchKey {
    fn eq(&self, other: &Self) -> bool {
        self.geometry_handle == other.geometry_handle
            && self.texture_handle == other.texture_handle
            && self.selected == other.selected
    }
}

impl Eq for GeometryBatchKey {}

pub type GeometryBatches = FxHashMap<GeometryBatchKey, Vec<Instance>>;

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

    pub fn batch_geometry(&mut self) -> GeometryBatches {
        self.calculate_world_matrices();

        let mut batches = GeometryBatches::with_hasher(FxBuildHasher::new());

        let visible_nodes = self
            .graph
            .node_weights()
            .zip(self.graph.node_indices())
            .filter(|(node, _)| node.visible);

        for (scene_node, index) in visible_nodes {
            // log::info!("Batching node {}", i);
            match &scene_node.ty {
                NodeType::Renderable(renderable) => {
                    let node_key = GeometryBatchKey {
                        geometry_handle: renderable.geometry_handle,
                        texture_handle: renderable.texture_handle,
                        selected: self.selection.contains(&index),
                    };

                    let batch = batches.entry(node_key).or_insert(vec![]);

                    let transform = scene_node.world_transform.raw_matrix();

                    batch.push(Instance { transform });

                    // dbg!(batch.last().unwrap().transform_x);
                    // dbg!(batch.last().unwrap().transform_x);
                    // dbg!(batch.last().unwrap().transform_x);
                    // dbg!(batch.last().unwrap().transform_x);
                }
                NodeType::Group => (),
            }
        }

        // for (i, batch) in batches.iter().enumerate() {
        //     log::info!("Batch {} len {}", i, batch.1.instances.len());
        // }

        batches
    }

    fn calculate_world_matrices(&mut self) {
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
