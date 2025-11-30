use color_eyre::owo_colors::colors::xterm::GrandisCaramel;
use itertools::Itertools;
use nalgebra::{Point3, UnitVector3, Vector3};
use petgraph::Graph;

use crate::{
    geometry::{Geometry, GeometryVertex},
    resources::Resources,
    scene::graph::Renderable,
};

enum Axis {
    X,
    Y,
    Z,
}

struct Split {
    axis: Axis,
    position: f32,
}

struct BvhPass {
    bounds: Bounds,
    centroid: Vector3<f32>,
}

struct Bounds {
    min: Point3<f32>,
    max: Point3<f32>,
}

enum BvhNode {
    Bounds(Bounds),
    Leaf(Vec<[GeometryVertex; 3]>),
}

pub struct Bvh(Graph<BvhNode, Split>);

impl Bvh {
    pub fn from_geometry(geometry: &Geometry, resources: &Resources) -> Self {
        let triangles = Self::get_triangles(geometry);

        let pass = Self::pass_triangles(&triangles);

        let bounding_box_size = pass.bounds.max - pass.bounds.min;

        let split = if bounding_box_size.x > bounding_box_size.y
            && bounding_box_size.x > bounding_box_size.z
        {
            Split {
                axis: Axis::X,
                position: pass.centroid.x,
            }
        } else if bounding_box_size.y > bounding_box_size.z {
            Split {
                axis: Axis::Y,
                position: pass.centroid.y,
            }
        } else {
            Split {
                axis: Axis::Z,
                position: pass.centroid.z,
            }
        };

        Self(Graph::new())
    }

    fn pass_triangles(triangles: &[[GeometryVertex; 3]]) -> BvhPass {
        let mut centroid_sum = Vector3::zeros();
        let mut bounds = Bounds {
            min: Point3::new(f32::INFINITY, f32::INFINITY, f32::INFINITY),
            max: Point3::new(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY),
        };

        for triangle in triangles {
            let mut tri_centroid = Vector3::zeros();

            for vert in triangle {
                bounds.min.x = bounds.min.x.min(vert.position[0]);
                bounds.min.y = bounds.min.y.min(vert.position[1]);
                bounds.min.z = bounds.min.z.min(vert.position[2]);

                bounds.max.x = bounds.max.x.max(vert.position[0]);
                bounds.max.y = bounds.max.y.max(vert.position[1]);
                bounds.max.z = bounds.max.z.max(vert.position[2]);

                tri_centroid += Vector3::new(vert.position[0], vert.position[1], vert.position[2]);
            }

            centroid_sum += tri_centroid / 3.0;
        }

        let centroid = centroid_sum / triangles.len() as f32;

        BvhPass { bounds, centroid }
    }

    fn get_triangles(geometry: &Geometry) -> Vec<[GeometryVertex; 3]> {
        let mut triangles = Vec::new();

        for primitive in &geometry.primitives {
            for chunk in primitive.indices.chunks(3) {
                let triangle = [
                    primitive.vertices[chunk[0] as usize],
                    primitive.vertices[chunk[1] as usize],
                    primitive.vertices[chunk[2] as usize],
                ];
                triangles.push(triangle);
            }
        }

        triangles
    }
}
