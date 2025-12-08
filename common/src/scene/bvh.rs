use color_eyre::owo_colors::colors::xterm::GrandisCaramel;
use itertools::Itertools;
use nalgebra::{Point3, UnitVector3, Vector3};
use petgraph::{Graph, graph::NodeIndex};

use crate::{
    colors::{Color, ColorExt},
    debug::DebugCuboid,
    geometry::{Geometry, GeometryVertex},
    resources::Resources,
    scene::graph::Renderable,
};

#[derive(Debug, Clone)]
enum Axis {
    X,
    Y,
    Z,
}

#[derive(Debug, Clone)]
struct Split {
    axis: Axis,
    position: f32,
}

struct BvhPass {
    bounds: Bounds,
    centroid: Vector3<f32>,
}

impl BvhPass {
    fn determine_split(&self) -> Split {
        let bounding_box_size = self.bounds.max - self.bounds.min;

        if bounding_box_size.x > bounding_box_size.y && bounding_box_size.x > bounding_box_size.z {
            Split {
                axis: Axis::X,
                position: self.centroid.x,
            }
        } else if bounding_box_size.y > bounding_box_size.z {
            Split {
                axis: Axis::Y,
                position: self.centroid.y,
            }
        } else {
            Split {
                axis: Axis::Z,
                position: self.centroid.z,
            }
        }
    }
}

struct Bounds {
    min: Point3<f32>,
    max: Point3<f32>,
}

type Triangle = [[f32; 3]; 3];

type Centroid = Vector3<f32>;

struct TriangleWithCentroid {
    verts: Triangle,
    centroid: Centroid,
}

enum BvhNode {
    Bounds(Bounds),
    Leaf(Vec<Triangle>),
}

pub struct Bvh {
    pub graph: Graph<BvhNode, Split>,
    pub root: NodeIndex,
}

impl Bvh {
    pub fn from_geometry(geometry: &Geometry) -> Self {
        let tris_with_centroids = Self::get_tris_with_centroids(geometry);
        let mut graph = Graph::<BvhNode, Split>::new();

        let root = Self::build(&mut graph, tris_with_centroids);

        Self { graph, root }
    }

    pub fn get_debug_cuboids(&self) -> Vec<DebugCuboid> {
        self.graph
            .node_indices()
            .into_iter()
            .filter_map(|node| match &self.graph[node] {
                BvhNode::Bounds(bounds) => Some(DebugCuboid {
                    min: bounds.min.to_homogeneous().xyz(),
                    max: bounds.max.to_homogeneous().xyz(),
                    color: Color::from_named(palette::named::WHITE),
                }),
                BvhNode::Leaf(_) => None,
            })
            .collect_vec()
    }

    fn build(
        graph: &mut Graph<BvhNode, Split>,
        tris_with_centroids: Vec<TriangleWithCentroid>,
    ) -> NodeIndex {
        if tris_with_centroids.len() <= 2 {
            let leaf_tris = tris_with_centroids.into_iter().map(|t| t.verts).collect();
            return graph.add_node(BvhNode::Leaf(leaf_tris));
        }

        let pass = Self::pass_triangles(&tris_with_centroids);
        let split = pass.determine_split();

        let mut left = Vec::new();
        let mut right = Vec::new();

        for triangle in tris_with_centroids {
            let position = match split.axis {
                Axis::X => triangle.centroid.x,
                Axis::Y => triangle.centroid.y,
                Axis::Z => triangle.centroid.z,
            };

            if position < split.position {
                left.push(triangle);
            } else {
                right.push(triangle);
            }
        }

        if left.is_empty() || right.is_empty() {
            let leaf_tris = left
                .into_iter()
                .chain(right.into_iter())
                .map(|t| t.verts)
                .collect();
            return graph.add_node(BvhNode::Leaf(leaf_tris));
        }

        let bounds_node = graph.add_node(BvhNode::Bounds(pass.bounds));

        let left_id = Self::build(graph, left);
        let right_id = Self::build(graph, right);

        graph.add_edge(bounds_node, left_id, split.clone());
        graph.add_edge(bounds_node, right_id, split);

        bounds_node
    }

    fn pass_triangles(triangles_centroids: &[TriangleWithCentroid]) -> BvhPass {
        let mut centroid_sum = Vector3::zeros();
        let mut bounds = Bounds {
            min: Point3::new(f32::INFINITY, f32::INFINITY, f32::INFINITY),
            max: Point3::new(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY),
        };

        for triangle_centroid in triangles_centroids {
            for vert in triangle_centroid.verts {
                bounds.min.x = bounds.min.x.min(vert[0]);
                bounds.min.y = bounds.min.y.min(vert[1]);
                bounds.min.z = bounds.min.z.min(vert[2]);

                bounds.max.x = bounds.max.x.max(vert[0]);
                bounds.max.y = bounds.max.y.max(vert[1]);
                bounds.max.z = bounds.max.z.max(vert[2]);
            }

            centroid_sum += triangle_centroid.centroid / 3.0;
        }

        let centroid = centroid_sum / triangles_centroids.len() as f32;

        BvhPass { bounds, centroid }
    }

    fn get_tris_with_centroids(geometry: &Geometry) -> Vec<TriangleWithCentroid> {
        let mut triangles = Vec::new();

        for primitive in &geometry.primitives {
            for chunk in primitive.indices.chunks(3) {
                let verts = [
                    primitive.vertices[chunk[0] as usize].position,
                    primitive.vertices[chunk[1] as usize].position,
                    primitive.vertices[chunk[2] as usize].position,
                ];

                let mut centroid = Centroid::zeros();

                for chunk_index in chunk {
                    let pos = primitive.vertices[*chunk_index as usize].position;
                    centroid += Vector3::new(pos[0], pos[1], pos[2]);
                }

                triangles.push(TriangleWithCentroid { verts, centroid });
            }
        }

        triangles
    }
}
