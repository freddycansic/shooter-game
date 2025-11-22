use std::hash::{Hash, Hasher};

use fxhash::{FxBuildHasher, FxHashMap, FxHasher};
use itertools::Itertools;
use nalgebra::Transform3;
use petgraph::{Direction, graph::NodeIndex};

use crate::{
    renderer::Instance,
    resources::handle::{GeometryHandle, TextureHandle},
    transform::Transform,
};

pub struct SceneNode {
    local_transform: Transform,
    world_transform: Transform,
    world_transform_dirty: bool,
    pub visible: bool,
    pub selected: bool,
    pub ty: NodeType,
}

pub enum NodeType {
    Renderable(Renderable),
    Group,
}

impl SceneNode {
    pub fn new(ty: NodeType, local_transform: Transform) -> Self {
        Self {
            local_transform,
            world_transform: Transform::identity(),
            world_transform_dirty: true,
            visible: true,
            selected: false,
            ty,
        }
    }

    fn create_root() -> Self {
        Self {
            local_transform: Transform::identity(),
            world_transform: Transform::identity(),
            world_transform_dirty: false,
            visible: false,
            selected: false,
            ty: NodeType::Group,
        }
    }
}

pub struct Renderable {
    pub geometry_handle: GeometryHandle,
    pub texture_handle: TextureHandle,
}

#[derive(Clone)]
pub struct InstanceBatchKey {
    pub geometry_handle: GeometryHandle,
    pub texture_handle: TextureHandle,
    pub selected: bool,
}

pub struct InstanceBatch {
    pub key: InstanceBatchKey,
    pub instances: Vec<Instance>,
}

impl Hash for InstanceBatchKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.geometry_handle.hash(state);
        self.texture_handle.hash(state);
        self.selected.hash(state);
    }
}

impl PartialEq for InstanceBatchKey {
    fn eq(&self, other: &Self) -> bool {
        self.geometry_handle == other.geometry_handle
            && self.texture_handle == other.texture_handle
            && self.selected == other.selected
    }
}

impl Eq for InstanceBatchKey {}

pub struct RenderQueue {
    pub queue: Vec<InstanceBatch>,
}

pub struct SceneGraph {
    graph: petgraph::stable_graph::StableDiGraph<SceneNode, ()>,
    root: NodeIndex,
}

impl SceneGraph {
    pub fn new() -> Self {
        let mut graph = petgraph::stable_graph::StableDiGraph::<SceneNode, ()>::new();
        let root = graph.add_node(SceneNode::create_root());

        Self { graph, root }
    }

    pub fn add_root_node(&mut self, node: SceneNode) -> NodeIndex {
        let node_index = self.graph.add_node(node);

        self.graph.add_edge(self.root, node_index, ());

        node_index
    }

    pub fn build_render_queue(&mut self) -> RenderQueue {
        self.calculate_world_matrices();

        let batches = self.batch_instances();

        RenderQueue { queue: batches }
    }

    fn batch_instances(&self) -> Vec<InstanceBatch> {
        let mut batches =
            FxHashMap::<(GeometryHandle, TextureHandle, bool), InstanceBatch>::with_hasher(
                FxBuildHasher::new(),
            );

        let visible_nodes = self.graph.node_weights().filter(|node| node.visible);

        for (i, scene_node) in visible_nodes.enumerate() {
            // log::info!("Batching node {}", i);
            match &scene_node.ty {
                NodeType::Renderable(renderable) => {
                    let node_key = (
                        renderable.geometry_handle,
                        renderable.texture_handle,
                        scene_node.selected,
                    );
                    let batch = batches.entry(node_key).or_insert(InstanceBatch {
                        key: InstanceBatchKey {
                            texture_handle: renderable.texture_handle,
                            geometry_handle: renderable.geometry_handle,
                            selected: scene_node.selected,
                        },
                        instances: Vec::new(),
                    });

                    batch.instances.push(Instance {
                        transform: scene_node.world_transform.matrix(),
                    });
                }
                NodeType::Group => (),
            }
        }

        for (i, batch) in batches.iter().enumerate() {
            log::info!("Batch {} len {}", i, batch.1.instances.len());
        }

        batches.into_values().collect_vec()
    }

    fn calculate_world_matrices(&mut self) {
        self.calculate_world_matrices_inner(self.root);
    }

    fn calculate_world_matrices_inner(&mut self, parent: NodeIndex) {
        let children = self
            .graph
            .neighbors_directed(parent, Direction::Outgoing)
            .collect_vec();

        for child in children.into_iter() {
            self.graph[child].world_transform = self.graph[parent]
                .world_transform
                .combine(&self.graph[child].local_transform);

            self.graph[child].world_transform_dirty = false;

            self.calculate_world_matrices_inner(child);
        }
    }
}
