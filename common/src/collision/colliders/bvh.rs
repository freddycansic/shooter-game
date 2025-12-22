use itertools::Itertools;
use nalgebra::{Point3, Vector3};
use petgraph::{Direction, Graph, graph::NodeIndex};

use crate::{
    collision::{
        collidable::{Hit, Intersectable},
        colliders::aabb::Aabb,
    },
    colors::Color,
    debug::DebugCuboid,
    geometry::Geometry,
    maths::Ray,
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

pub struct BvhPass {
    pub aabb: Aabb,
    pub centroid: Vector3<f32>,
}

impl BvhPass {
    fn determine_split(&self) -> Split {
        let bounding_box_size = self.aabb.max - self.aabb.min;

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

pub type Triangle = [[f32; 3]; 3];

pub type Centroid = Vector3<f32>;

pub struct TriangleWithCentroid {
    verts: Triangle,
    centroid: Centroid,
}

enum BvhNode {
    Aabb(Aabb),
    Leaf { triangles: Vec<Triangle>, aabb: Aabb },
}

impl Intersectable for BvhNode {
    fn intersect_t(&self, ray: &Ray) -> Option<Hit> {
        match &self {
            Self::Aabb(aabb) => aabb.intersect_t(ray),
            Self::Leaf { .. } => unimplemented!(),
        }
    }
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
                BvhNode::Aabb(aabb) => Some(DebugCuboid {
                    min: aabb.min.to_homogeneous().xyz().cast::<f32>(),
                    max: aabb.max.to_homogeneous().xyz().cast::<f32>(),
                    color: Color::from_components((
                        fastrand::f32() * 100.0,
                        fastrand::f32() * 150.0,
                        fastrand::f32() * 360.0,
                    )),
                }),
                BvhNode::Leaf { .. } => None,
            })
            .collect_vec()
    }

    fn build(graph: &mut Graph<BvhNode, Split>, tris_with_centroids: Vec<TriangleWithCentroid>) -> NodeIndex {
        if tris_with_centroids.len() <= 2 {
            let leaf_tris = tris_with_centroids.into_iter().map(|tri| tri.verts).collect_vec();

            let leaf_aabb = Self::pass_triangles(&leaf_tris);

            return graph.add_node(BvhNode::Leaf {
                triangles: leaf_tris,
                aabb: leaf_aabb,
            });
        }

        let pass = Self::pass_triangles_with_centroids(&tris_with_centroids);
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
                .map(|tri| tri.verts)
                .collect_vec();

            let leaf_aabb = Self::pass_triangles(&leaf_tris);

            return graph.add_node(BvhNode::Leaf {
                triangles: leaf_tris,
                aabb: leaf_aabb,
            });
        }

        let left_id = Self::build(graph, left);
        let right_id = Self::build(graph, right);

        let left_aabb = match &graph[left_id] {
            BvhNode::Aabb(aabb) => aabb,
            BvhNode::Leaf { aabb, .. } => aabb,
        };

        let right_aabb = match &graph[right_id] {
            BvhNode::Aabb(aabb) => aabb,
            BvhNode::Leaf { aabb, .. } => aabb,
        };

        let combined = left_aabb.union(&right_aabb);

        let parent_id = graph.add_node(BvhNode::Aabb(combined));

        graph.add_edge(parent_id, left_id, split.clone());
        graph.add_edge(parent_id, right_id, split);

        parent_id
    }

    pub fn pass_triangles_with_centroids(triangles_centroids: &[TriangleWithCentroid]) -> BvhPass {
        let mut centroid_sum = Vector3::zeros();
        let mut aabb = Aabb {
            min: Point3::new(f64::INFINITY, f64::INFINITY, f64::INFINITY),
            max: Point3::new(f64::NEG_INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY),
        };

        for triangle_centroid in triangles_centroids {
            for vert in triangle_centroid.verts {
                aabb.min.x = aabb.min.x.min(vert[0] as f64);
                aabb.min.y = aabb.min.y.min(vert[1] as f64);
                aabb.min.z = aabb.min.z.min(vert[2] as f64);

                aabb.max.x = aabb.max.x.max(vert[0] as f64);
                aabb.max.y = aabb.max.y.max(vert[1] as f64);
                aabb.max.z = aabb.max.z.max(vert[2] as f64);
            }

            centroid_sum += triangle_centroid.centroid / 3.0;
        }

        let centroid = centroid_sum / triangles_centroids.len() as f32;

        BvhPass { aabb, centroid }
    }

    pub fn pass_triangles(triangles: &[Triangle]) -> Aabb {
        let mut aabb = Aabb {
            min: Point3::new(f64::INFINITY, f64::INFINITY, f64::INFINITY),
            max: Point3::new(f64::NEG_INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY),
        };

        for triangle in triangles {
            for vert in triangle {
                aabb.min.x = aabb.min.x.min(vert[0] as f64);
                aabb.min.y = aabb.min.y.min(vert[1] as f64);
                aabb.min.z = aabb.min.z.min(vert[2] as f64);

                aabb.max.x = aabb.max.x.max(vert[0] as f64);
                aabb.max.y = aabb.max.y.max(vert[1] as f64);
                aabb.max.z = aabb.max.z.max(vert[2] as f64);
            }
        }

        aabb
    }

    pub fn get_tris_with_centroids(geometry: &Geometry) -> Vec<TriangleWithCentroid> {
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

    fn intersect_t_inner(&self, ray: &Ray, node: NodeIndex) -> Option<Hit> {
        match &self.graph[node] {
            BvhNode::Aabb(aabb) => {
                if aabb.intersect_t(ray).is_some() {
                    for child in self.graph.neighbors_directed(node, Direction::Outgoing) {
                        if let Some(intersection) = self.intersect_t_inner(ray, child) {
                            return Some(intersection);
                        }
                    }
                }
            }
            _ => unimplemented!(),
        }

        None
    }
}

impl Intersectable for Bvh {
    fn intersect_t(&self, ray: &Ray) -> Option<Hit> {
        self.intersect_t_inner(ray, self.root)
    }
}
