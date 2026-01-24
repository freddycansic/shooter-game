use itertools::Itertools;
use nalgebra::{Point3, Vector3};
use petgraph::{Direction, Graph, graph::NodeIndex};

use crate::geometry::Primitive;
use crate::{
    collision::{
        collidable::{RayHit, Intersectable},
        colliders::aabb::Aabb,
    },
    colors::Color,
    debug::DebugCuboid,
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

#[derive(Debug)]
pub struct Triangle([Vector3<f32>; 3]);

impl Intersectable for Triangle {
    /// (t, u, v) are barycentric coordinates
    /// they sum to 1, and are weights towards triangle vertices
    /// if any are negative, then the Euclidean point is outside the triangle
    fn intersect_ray(&self, ray: &Ray) -> Option<RayHit> {
        let v0v1 = self.0[1] - self.0[0];
        let v0v2 = self.0[2] - self.0[0];
        let pvec = ray.direction().cross(&v0v2);

        let det = v0v1.dot(&pvec);

        // If the determinant is negative, the triangle is back-facing.
        // Just abs to ignore this
        // If the determinant is close to 0, the ray misses the triangle.
        if det.abs() < f32::EPSILON {
            return None;
        }

        let inv_det = 1.0 / det;

        let tvec = (ray.origin - self.0[0]).to_homogeneous().xyz();
        let u = tvec.dot(&pvec) * inv_det;
        if u < 0.0 || u > 1.0 {
            return None;
        }

        let qvec = tvec.cross(&v0v1);
        let v = ray.direction().dot(&qvec) * inv_det;
        if v < 0.0 || u + v > 1.0 {
            return None;
        }

        let t = v0v2.dot(&qvec) * inv_det;

        debug_assert_ne!(t, f32::NAN);
        debug_assert_ne!(t, f32::NEG_INFINITY);
        debug_assert_ne!(t, f32::INFINITY);

        // Slightly inefficient to return redundant information
        // Can always remove it if this becomes a real problem
        Some(RayHit { tmin: t, tmax: t })
    }
}

pub type Centroid = Vector3<f32>;

pub struct TriangleWithCentroid {
    verts: Triangle,
    centroid: Centroid,
}

#[derive(Debug)]
enum BvhNode {
    Aabb(Aabb),
    Leaf { triangles: Vec<Triangle>, aabb: Aabb },
}

#[derive(Debug)]
pub struct Bvh {
    pub graph: Graph<BvhNode, Split>,
    pub root: NodeIndex,
}

impl Bvh {
    pub fn from_primitives(primitives: &[Primitive]) -> Self {
        let tris_with_centroids = Self::get_tris_with_centroids(primitives);
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
            min: Point3::new(f32::INFINITY, f32::INFINITY, f32::INFINITY),
            max: Point3::new(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY),
        };

        for triangle_centroid in triangles_centroids {
            for vert in triangle_centroid.verts.0 {
                aabb.min.x = aabb.min.x.min(vert[0]);
                aabb.min.y = aabb.min.y.min(vert[1]);
                aabb.min.z = aabb.min.z.min(vert[2]);

                aabb.max.x = aabb.max.x.max(vert[0]);
                aabb.max.y = aabb.max.y.max(vert[1]);
                aabb.max.z = aabb.max.z.max(vert[2]);
            }

            centroid_sum += triangle_centroid.centroid / 3.0;
        }

        let centroid = centroid_sum / triangles_centroids.len() as f32;

        BvhPass { aabb, centroid }
    }

    pub fn pass_triangles(triangles: &[Triangle]) -> Aabb {
        let mut aabb = Aabb {
            min: Point3::new(f32::INFINITY, f32::INFINITY, f32::INFINITY),
            max: Point3::new(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY),
        };

        for triangle in triangles {
            for vert in triangle.0 {
                aabb.min.x = aabb.min.x.min(vert[0]);
                aabb.min.y = aabb.min.y.min(vert[1]);
                aabb.min.z = aabb.min.z.min(vert[2]);

                aabb.max.x = aabb.max.x.max(vert[0]);
                aabb.max.y = aabb.max.y.max(vert[1]);
                aabb.max.z = aabb.max.z.max(vert[2]);
            }
        }

        aabb
    }

    pub fn get_tris_with_centroids(primitives: &[Primitive]) -> Vec<TriangleWithCentroid> {
        let mut triangles = Vec::new();

        for primitive in primitives {
            for chunk in primitive.indices.chunks(3) {
                let triangle = Triangle([
                    Vector3::from_row_slice(primitive.vertices[chunk[0] as usize].position.as_slice()),
                    Vector3::from_row_slice(primitive.vertices[chunk[1] as usize].position.as_slice()),
                    Vector3::from_row_slice(primitive.vertices[chunk[2] as usize].position.as_slice()),
                ]);

                let mut centroid = Centroid::zeros();

                for chunk_index in chunk {
                    let pos = primitive.vertices[*chunk_index as usize].position;
                    centroid += Vector3::new(pos[0], pos[1], pos[2]);
                }

                triangles.push(TriangleWithCentroid {
                    verts: triangle,
                    centroid,
                });
            }
        }

        triangles
    }

    fn intersect_ray_inner(&self, ray: &Ray, node: NodeIndex) -> Option<RayHit> {
        match &self.graph[node] {
            BvhNode::Aabb(aabb) => aabb.intersect_ray(ray).and_then(|_| {
                self.graph
                    .neighbors_directed(node, Direction::Outgoing)
                    .filter_map(|child| self.intersect_ray_inner(ray, child))
                    .min_by(|a, b| a.tmin.partial_cmp(&b.tmin).unwrap())
            }),
            BvhNode::Leaf { triangles, aabb } => aabb.intersect_ray(ray).and_then(|_| {
                triangles
                    .iter()
                    .filter_map(|tri| tri.intersect_ray(ray))
                    .min_by(|a, b| a.tmin.partial_cmp(&b.tmin).unwrap())
            }),
        }
    }
}

impl Intersectable for Bvh {
    fn intersect_ray(&self, ray: &Ray) -> Option<RayHit> {
        self.intersect_ray_inner(ray, self.root)
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;

    use super::*;

    #[test]
    fn intersect_ray_triangle_perpendicular() {
        let ray = Ray::new(Point3::new(0.0, 0.0, 0.0), Vector3::new(0.0, 0.0, 1.0));

        let triangle = Triangle([
            Vector3::new(1.0, 0.0, 1.0),
            Vector3::new(-1.0, 1.0, 1.0),
            Vector3::new(-1.0, -1.0, 1.0),
        ]);

        let result = triangle.intersect_ray(&ray).unwrap();
        assert_relative_eq!(result.tmin, 1.0);
        assert_relative_eq!(result.tmax, 1.0);
    }

    #[test]
    fn intersect_ray_triangle_corner() {
        let ray = Ray::new(Point3::new(1.0, 0.0, 0.0), Vector3::new(0.0, 0.0, 1.0));

        let triangle = Triangle([
            Vector3::new(1.0, 0.0, 1.0),
            Vector3::new(-1.0, 1.0, 1.0),
            Vector3::new(-1.0, -1.0, 1.0),
        ]);

        let result = triangle.intersect_ray(&ray).unwrap();
        assert_relative_eq!(result.tmin, 1.0);
        assert_relative_eq!(result.tmax, 1.0);
    }

    #[test]
    fn intersect_ray_triangle_edge() {
        let v0 = Vector3::new(1.0, 0.0, 1.0);
        let v1 = Vector3::new(-1.0, 1.0, 1.0);
        let v2 = Vector3::new(-1.0, -1.0, 1.0);

        let v0v1 = v1 - v0;
        let midpoint = v0 + v0v1 / 2.0 - Vector3::new(0.0, 0.0, 1.0);

        let ray = Ray::new(midpoint.into(), Vector3::new(0.0, 0.0, 1.0));

        //   ^
        //  <->
        // <--->
        let triangle = Triangle([v0, v1, v2]);

        let result = triangle.intersect_ray(&ray).unwrap();
        assert_relative_eq!(result.tmin, 1.0);
        assert_relative_eq!(result.tmax, 1.0);
    }

    #[test]
    fn intersect_ray_triangle_diagonal() {
        let ray = Ray::new(Point3::new(-1.0, -1.0, -1.0), Vector3::new(1.0, 1.0, 1.0).normalize());

        let triangle = Triangle([
            Vector3::new(1.0, 0.0, 0.0),
            Vector3::new(-1.0, 1.0, 0.0),
            Vector3::new(-1.0, -1.0, 0.0),
        ]);

        let result = triangle.intersect_ray(&ray).unwrap();
        assert_relative_eq!(result.tmin, 3.0_f32.sqrt());
        assert_relative_eq!(result.tmax, 3.0_f32.sqrt());
    }
}
